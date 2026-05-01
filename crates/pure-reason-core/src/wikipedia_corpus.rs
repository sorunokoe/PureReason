//! # Wikipedia Corpus — External Knowledge Base (TRIZ P40)
//!
//! 6M Wikipedia article abstracts with BM25 full-text search.
//! Replaces the 107-prior world_priors atlas with comprehensive, versioned external knowledge.
//!
//! ## Key Features
//! - **Lazy loading**: Corpus loaded on first query (not at initialization)
//! - **BM25 search**: Fast full-text search via SQLite FTS5
//! - **Entity detection**: Check if entities exist in corpus
//! - **LRU cache**: 1000 most recent queries cached in memory
//! - **Versioned**: Corpus version included in verification results
//! - **Leak-audited**: CI fails if overlap >5% with benchmarks
//!
//! ## Usage
//!
//! ```rust
//! use pure_reason_core::wikipedia_corpus::WikipediaCorpus;
//!
//! let corpus = WikipediaCorpus::new("data/corpus/wikipedia_v1.0.jsonl.gz")?;
//!
//! // Query for articles
//! let results = corpus.query("Albert Einstein", 10)?;
//! for article in results {
//!     println!("{}: {}", article.title, article.abstract_text);
//! }
//!
//! // Check entity presence
//! if corpus.contains_entity("Albert Einstein")? {
//!     println!("Entity found");
//! }
//!
//! // Get version for reproducibility
//! println!("Corpus version: {}", corpus.version());
//! ```

use crate::error::{PureReasonError, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// A Wikipedia article record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    /// Wikipedia article ID
    pub id: String,
    /// Article title
    pub title: String,
    /// Article abstract (first paragraph)
    pub abstract_text: String,
    /// Canonical Wikipedia URL
    pub url: String,
    /// Wikipedia categories
    pub categories: Vec<String>,
    /// Extracted named entities
    pub entities: Vec<String>,
    /// Last modification timestamp
    pub last_modified: String,
    /// Word count of abstract
    pub word_count: usize,
}

/// LRU cache for query results.
struct QueryCache {
    cache: HashMap<String, Vec<Article>>,
    order: Vec<String>,
    capacity: usize,
}

impl QueryCache {
    fn new(capacity: usize) -> Self {
        Self {
            cache: HashMap::new(),
            order: Vec::new(),
            capacity,
        }
    }

    fn get(&mut self, key: &str) -> Option<&Vec<Article>> {
        if let Some(articles) = self.cache.get(key) {
            // Move to end (most recently used)
            if let Some(pos) = self.order.iter().position(|k| k == key) {
                self.order.remove(pos);
            }
            self.order.push(key.to_string());
            Some(articles)
        } else {
            None
        }
    }

    fn insert(&mut self, key: String, articles: Vec<Article>) {
        // Evict oldest if at capacity
        if self.cache.len() >= self.capacity && !self.cache.contains_key(&key) {
            if let Some(oldest) = self.order.first().cloned() {
                self.cache.remove(&oldest);
                self.order.remove(0);
            }
        }

        self.cache.insert(key.clone(), articles);
        self.order.push(key);
    }
}

/// Wikipedia corpus with BM25 search and caching.
pub struct WikipediaCorpus {
    /// Path to SQLite index database
    db_path: PathBuf,
    /// Corpus version (e.g., "1.0")
    version: String,
    /// SQLite connection (lazy-loaded)
    connection: Arc<Mutex<Option<Connection>>>,
    /// Query cache (LRU, max 1000 entries)
    cache: Arc<Mutex<QueryCache>>,
}

impl WikipediaCorpus {
    /// Create a new corpus instance (lazy loading).
    ///
    /// The corpus is not loaded until the first query.
    ///
    /// # Arguments
    /// * `db_path` - Path to SQLite index database (e.g., "data/corpus/wikipedia_v1.0.index.db")
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();

        // Extract version from filename (e.g., "wikipedia_v1.0.index.db" → "1.0")
        let version = db_path
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.strip_prefix("wikipedia_v"))
            .and_then(|s| s.strip_suffix(".index"))
            .unwrap_or("unknown")
            .to_string();

        Ok(Self {
            db_path,
            version,
            connection: Arc::new(Mutex::new(None)),
            cache: Arc::new(Mutex::new(QueryCache::new(1000))),
        })
    }

    /// Get corpus version for reproducibility.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Lazy-load database connection.
    fn get_connection(&self) -> Result<()> {
        let mut conn_guard = self.connection.lock().unwrap();

        if conn_guard.is_none() {
            let conn = Connection::open(&self.db_path).map_err(|e| {
                PureReasonError::Storage(format!("Failed to open corpus database: {}", e))
            })?;

            *conn_guard = Some(conn);
        }

        Ok(())
    }

    /// Query for articles matching the search text.
    ///
    /// Uses BM25 full-text search on title and abstract.
    ///
    /// # Arguments
    /// * `query_text` - Search query
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    /// Vector of articles ranked by relevance (BM25 score)
    pub fn query(&self, query_text: &str, limit: usize) -> Result<Vec<Article>> {
        let cache_key = format!("{}:{}", query_text.to_lowercase().trim(), limit);

        // Check cache first
        {
            let mut cache_guard = self.cache.lock().unwrap();
            if let Some(cached) = cache_guard.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        // Ensure connection is initialized
        self.get_connection()?;

        let conn_guard = self.connection.lock().unwrap();
        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| PureReasonError::Storage("Connection not initialized".to_string()))?;

        // BM25 query via FTS5
        let mut stmt = conn
            .prepare(
                "SELECT a.id, a.title, a.abstract, a.url, a.categories, a.entities, a.last_modified, a.word_count
                 FROM articles_fts fts
                 JOIN articles a ON a.rowid = fts.rowid
                 WHERE articles_fts MATCH ?1
                 ORDER BY rank
                 LIMIT ?2",
            )
            .map_err(|e| PureReasonError::Storage(format!("Failed to prepare query: {}", e)))?;

        let articles: Vec<Article> = stmt
            .query_map(params![query_text, limit], |row| {
                let id: String = row.get(0)?;
                let title: String = row.get(1)?;
                let abstract_text: String = row.get(2)?;
                let url: String = row.get(3)?;
                let categories_json: String = row.get(4)?;
                let entities_json: String = row.get(5)?;
                let last_modified: String = row.get(6)?;
                let word_count: i64 = row.get(7)?;

                let categories: Vec<String> =
                    serde_json::from_str(&categories_json).unwrap_or_default();
                let entities: Vec<String> =
                    serde_json::from_str(&entities_json).unwrap_or_default();

                Ok(Article {
                    id,
                    title,
                    abstract_text,
                    url,
                    categories,
                    entities,
                    last_modified,
                    word_count: word_count as usize,
                })
            })
            .map_err(|e| PureReasonError::Storage(format!("Query execution failed: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| PureReasonError::Storage(format!("Row parsing failed: {}", e)))?;

        // Cache results
        {
            let mut cache_guard = self.cache.lock().unwrap();
            cache_guard.insert(cache_key, articles.clone());
        }

        Ok(articles)
    }

    /// Check if an entity exists in the corpus.
    ///
    /// Useful for novelty detection: flag claims that introduce entities
    /// not present in the knowledge base.
    ///
    /// # Arguments
    /// * `entity` - Entity name to search for
    ///
    /// # Returns
    /// `true` if entity found in any article, `false` otherwise
    pub fn contains_entity(&self, entity: &str) -> Result<bool> {
        self.get_connection()?;

        let conn_guard = self.connection.lock().unwrap();
        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| PureReasonError::Storage("Connection not initialized".to_string()))?;

        // Search in entities JSON field
        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM articles WHERE entities LIKE ?1 LIMIT 1")
            .map_err(|e| PureReasonError::Storage(format!("Failed to prepare query: {}", e)))?;

        let pattern = format!("%\"{}%", entity);
        let count: i64 = stmt
            .query_row(params![pattern], |row| row.get(0))
            .map_err(|e| PureReasonError::Storage(format!("Query execution failed: {}", e)))?;

        Ok(count > 0)
    }

    /// Get article by ID.
    pub fn get_by_id(&self, article_id: &str) -> Result<Option<Article>> {
        self.get_connection()?;

        let conn_guard = self.connection.lock().unwrap();
        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| PureReasonError::Storage("Connection not initialized".to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, title, abstract, url, categories, entities, last_modified, word_count
                 FROM articles WHERE id = ?1",
            )
            .map_err(|e| PureReasonError::Storage(format!("Failed to prepare query: {}", e)))?;

        let result = stmt.query_row(params![article_id], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let abstract_text: String = row.get(2)?;
            let url: String = row.get(3)?;
            let categories_json: String = row.get(4)?;
            let entities_json: String = row.get(5)?;
            let last_modified: String = row.get(6)?;
            let word_count: i64 = row.get(7)?;

            let categories: Vec<String> =
                serde_json::from_str(&categories_json).unwrap_or_default();
            let entities: Vec<String> = serde_json::from_str(&entities_json).unwrap_or_default();

            Ok(Article {
                id,
                title,
                abstract_text,
                url,
                categories,
                entities,
                last_modified,
                word_count: word_count as usize,
            })
        });

        match result {
            Ok(article) => Ok(Some(article)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(PureReasonError::Storage(format!(
                "Query execution failed: {}",
                e
            ))),
        }
    }

    /// Get corpus statistics.
    pub fn stats(&self) -> Result<CorpusStats> {
        self.get_connection()?;

        let conn_guard = self.connection.lock().unwrap();
        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| PureReasonError::Storage("Connection not initialized".to_string()))?;

        let total_articles: i64 = conn
            .query_row("SELECT COUNT(*) FROM articles", [], |row| row.get(0))
            .map_err(|e| PureReasonError::Storage(format!("Failed to get stats: {}", e)))?;

        let total_words: i64 = conn
            .query_row("SELECT SUM(word_count) FROM articles", [], |row| row.get(0))
            .map_err(|e| PureReasonError::Storage(format!("Failed to get stats: {}", e)))?;

        Ok(CorpusStats {
            total_articles: total_articles as usize,
            total_words: total_words as usize,
            version: self.version.clone(),
        })
    }
}

/// Corpus statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusStats {
    pub total_articles: usize,
    pub total_words: usize,
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_corpus_version_extraction() {
        let corpus = WikipediaCorpus::new("data/corpus/wikipedia_v1.0.index.db").unwrap();
        assert_eq!(corpus.version(), "1.0");

        let corpus2 = WikipediaCorpus::new("data/corpus/wikipedia_v2.3.index.db").unwrap();
        assert_eq!(corpus2.version(), "2.3");
    }

    #[test]
    fn test_query_cache() {
        let mut cache = QueryCache::new(3);

        let articles1 = vec![Article {
            id: "1".to_string(),
            title: "Test".to_string(),
            abstract_text: "Test abstract".to_string(),
            url: "http://example.com".to_string(),
            categories: vec![],
            entities: vec![],
            last_modified: "2026-05-01T00:00:00Z".to_string(),
            word_count: 2,
        }];

        // Insert 3 entries
        cache.insert("query1".to_string(), articles1.clone());
        cache.insert("query2".to_string(), articles1.clone());
        cache.insert("query3".to_string(), articles1.clone());

        assert_eq!(cache.cache.len(), 3);

        // Insert 4th entry should evict oldest (query1)
        cache.insert("query4".to_string(), articles1.clone());

        assert_eq!(cache.cache.len(), 3);
        assert!(cache.cache.get("query1").is_none());
        assert!(cache.cache.get("query4").is_some());
    }

    #[test]
    fn test_article_serialization() {
        let article = Article {
            id: "123".to_string(),
            title: "Test Article".to_string(),
            abstract_text: "This is a test.".to_string(),
            url: "https://test.com".to_string(),
            categories: vec!["Science".to_string()],
            entities: vec!["Test".to_string()],
            last_modified: "2026-01-01".to_string(),
            word_count: 10,
        };

        let json = serde_json::to_string(&article).unwrap();
        let deserialized: Article = serde_json::from_str(&json).unwrap();

        assert_eq!(article.id, deserialized.id);
        assert_eq!(article.title, deserialized.title);
    }

    #[test]
    fn test_corpus_initialization() {
        // Test with non-existent path (should not panic, lazy loading)
        let corpus = WikipediaCorpus::new("nonexistent.db").unwrap();
        assert_eq!(corpus.version(), "unknown");
    }

    #[test]
    fn test_mock_db_query() {
        // Create a temporary SQLite database
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path();

        // Create schema
        let conn = Connection::open(db_path).unwrap();
        conn.execute(
            "CREATE VIRTUAL TABLE articles_fts USING fts5(
                title, abstract, content='articles', content_rowid='rowid'
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE articles (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                abstract TEXT NOT NULL,
                url TEXT,
                categories TEXT,
                entities TEXT,
                last_modified TEXT,
                word_count INTEGER
            )",
            [],
        )
        .unwrap();

        // Insert test data
        conn.execute(
            "INSERT INTO articles VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                "1",
                "Albert Einstein",
                "Albert Einstein was a theoretical physicist.",
                "https://en.wikipedia.org/wiki/Albert_Einstein",
                "[\"Physics\"]",
                "[\"Albert Einstein\"]",
                "2026-01-01",
                10
            ],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO articles_fts (rowid, title, abstract) VALUES (1, ?, ?)",
            params![
                "Albert Einstein",
                "Albert Einstein was a theoretical physicist."
            ],
        )
        .unwrap();

        drop(conn);

        // Test corpus with real DB
        let corpus = WikipediaCorpus::new(db_path.to_str().unwrap()).unwrap();
        let results = corpus.query("Einstein", 10).unwrap();

        assert!(!results.is_empty(), "Should find Einstein article");
        assert_eq!(results[0].title, "Albert Einstein");
    }

    #[test]
    fn test_entity_detection() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path();

        // Create minimal schema
        let conn = Connection::open(db_path).unwrap();
        conn.execute(
            "CREATE TABLE articles (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                abstract TEXT NOT NULL,
                url TEXT,
                categories TEXT,
                entities TEXT,
                last_modified TEXT,
                word_count INTEGER
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO articles VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                "1",
                "Python (programming language)",
                "Python is a high-level programming language.",
                "https://en.wikipedia.org/wiki/Python_(programming_language)",
                "[\"Programming\"]",
                "[\"Python\", \"Programming language\"]",
                "2026-01-01",
                15
            ],
        )
        .unwrap();

        drop(conn);

        let corpus = WikipediaCorpus::new(db_path.to_str().unwrap()).unwrap();

        // Should find entity by title match
        assert!(corpus.contains_entity("Python").unwrap_or(false));
    }
}
