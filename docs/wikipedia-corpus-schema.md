# Wikipedia Corpus Schema

**Version**: 1.0  
**Date**: 2026-05-01  
**Format**: JSONL (JSON Lines) compressed with gzip

---

## File Structure

```
data/corpus/
├── wikipedia_v1.0.jsonl.gz          # Main corpus (6M articles, ~2GB)
├── wikipedia_v1.0.index.db          # BM25 index (SQLite, ~500MB)
├── wikipedia_v1.0.metadata.json     # Corpus metadata
└── wikipedia_v1.0.audit.json        # Leak audit report
```

---

## JSONL Record Schema

Each line in the JSONL file is a complete JSON object:

```json
{
  "id": "12345",
  "title": "Albert Einstein",
  "abstract": "Albert Einstein was a German-born theoretical physicist...",
  "url": "https://en.wikipedia.org/wiki/Albert_Einstein",
  "categories": ["1879 births", "1955 deaths", "Theoretical physicists", "Nobel laureates in Physics"],
  "entities": ["Albert Einstein", "Germany", "Physics", "Nobel Prize"],
  "last_modified": "2026-04-30T12:00:00Z",
  "word_count": 342
}
```

### Field Specifications

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Wikipedia article ID (unique) |
| `title` | string | Yes | Article title |
| `abstract` | string | Yes | First paragraph (abstract) of article |
| `url` | string | Yes | Canonical Wikipedia URL |
| `categories` | array[string] | Yes | Wikipedia categories |
| `entities` | array[string] | Yes | Extracted named entities (people, places, concepts) |
| `last_modified` | string (ISO 8601) | Yes | Last modification timestamp |
| `word_count` | integer | Yes | Word count of abstract |

---

## Metadata File Format

```json
{
  "version": "1.0",
  "created": "2026-05-01T13:00:00Z",
  "source": "enwiki-20260430-abstract.xml",
  "article_count": 6234567,
  "total_size_bytes": 2147483648,
  "compression": "gzip",
  "entity_extraction": "spaCy en_core_web_sm 3.7.0",
  "processing_date": "2026-05-01",
  "checksums": {
    "corpus": "sha256:abc123...",
    "index": "sha256:def456..."
  }
}
```

---

## BM25 Index Schema (SQLite)

```sql
CREATE TABLE articles (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    abstract TEXT NOT NULL,
    url TEXT NOT NULL,
    categories TEXT NOT NULL,  -- JSON array
    entities TEXT NOT NULL,    -- JSON array
    last_modified TEXT NOT NULL,
    word_count INTEGER NOT NULL
);

CREATE INDEX idx_title ON articles(title);
CREATE INDEX idx_entities ON articles(entities);

-- BM25 full-text search on abstract
CREATE VIRTUAL TABLE articles_fts USING fts5(
    id UNINDEXED,
    title,
    abstract,
    content=articles,
    content_rowid=rowid
);

-- Triggers to keep FTS in sync
CREATE TRIGGER articles_ai AFTER INSERT ON articles BEGIN
  INSERT INTO articles_fts(rowid, id, title, abstract)
  VALUES (new.rowid, new.id, new.title, new.abstract);
END;

CREATE TRIGGER articles_ad AFTER DELETE ON articles BEGIN
  DELETE FROM articles_fts WHERE rowid = old.rowid;
END;

CREATE TRIGGER articles_au AFTER UPDATE ON articles BEGIN
  DELETE FROM articles_fts WHERE rowid = old.rowid;
  INSERT INTO articles_fts(rowid, id, title, abstract)
  VALUES (new.rowid, new.id, new.title, new.abstract);
END;
```

---

## Versioning Strategy

### Version Naming

Format: `wikipedia_vMAJOR.MINOR.jsonl.gz`

- **MAJOR**: Incremented when schema changes (breaking)
- **MINOR**: Incremented for corpus updates (non-breaking)

Examples:
- `v1.0` - Initial release (2026-05-01)
- `v1.1` - Weekly update (2026-05-08)
- `v2.0` - Schema change (if needed)

### Reproducibility

All verification results include corpus version:

```json
{
  "verdict": {...},
  "corpus_version": "1.0",
  "corpus_timestamp": "2026-05-01T13:00:00Z"
}
```

Auditors can replay with exact same corpus version.

---

## Leak Audit Specification

### Audit Report Format

```json
{
  "audit_date": "2026-05-01T14:00:00Z",
  "corpus_version": "1.0",
  "benchmarks_checked": [
    {
      "name": "TruthfulQA",
      "total_questions": 817,
      "overlapping_articles": 12,
      "overlap_percentage": 1.47,
      "status": "PASS"
    },
    {
      "name": "HaluEval QA",
      "total_questions": 5000,
      "overlapping_articles": 143,
      "overlap_percentage": 2.86,
      "status": "PASS"
    }
  ],
  "overall_status": "PASS",
  "max_overlap_percentage": 2.86,
  "threshold": 5.0
}
```

### Audit Rules

1. **Overlap Detection**: Compare corpus article IDs against benchmark dataset sources
2. **Threshold**: Fail if overlap >5% with any benchmark
3. **Exemptions**: General knowledge articles (e.g., "Earth", "Water") are exempt
4. **CI Integration**: Run on every corpus update, fail build if FAIL

### Audit Script

Location: `benchmarks/audit_corpus_leak.py`

Usage:
```bash
python3 benchmarks/audit_corpus_leak.py \
  --corpus data/corpus/wikipedia_v1.0.jsonl.gz \
  --benchmarks truthfulqa,halueval_qa,ragtruth \
  --threshold 5.0 \
  --output data/corpus/wikipedia_v1.0.audit.json
```

---

## Processing Pipeline

### 1. Download

Source: https://dumps.wikimedia.org/enwiki/latest/enwiki-latest-abstract.xml

```bash
wget https://dumps.wikimedia.org/enwiki/latest/enwiki-latest-abstract.xml.gz
gunzip enwiki-latest-abstract.xml.gz
```

### 2. Parse XML to JSONL

Script: `scripts/process_wikipedia_corpus.py`

```bash
python3 scripts/process_wikipedia_corpus.py \
  --input enwiki-latest-abstract.xml \
  --output data/corpus/wikipedia_v1.0.jsonl \
  --spacy-model en_core_web_sm
```

Steps:
- Parse XML (xml.etree.ElementTree)
- Extract ID, title, abstract, URL
- Extract categories from XML
- Named entity extraction with spaCy
- Word count calculation
- Write JSONL line-by-line

### 3. Build BM25 Index

Script: `scripts/build_bm25_index.py`

```bash
python3 scripts/build_bm25_index.py \
  --input data/corpus/wikipedia_v1.0.jsonl \
  --output data/corpus/wikipedia_v1.0.index.db
```

Steps:
- Create SQLite database
- Create tables and FTS5 index
- Bulk insert from JSONL (batch_size=10000)
- Create indices
- Vacuum and analyze

### 4. Compress

```bash
gzip data/corpus/wikipedia_v1.0.jsonl
# Result: wikipedia_v1.0.jsonl.gz (~2GB from ~8GB raw)
```

### 5. Generate Metadata

```bash
python3 scripts/generate_corpus_metadata.py \
  --corpus data/corpus/wikipedia_v1.0.jsonl.gz \
  --index data/corpus/wikipedia_v1.0.index.db \
  --output data/corpus/wikipedia_v1.0.metadata.json
```

### 6. Run Leak Audit

```bash
python3 benchmarks/audit_corpus_leak.py \
  --corpus data/corpus/wikipedia_v1.0.jsonl.gz \
  --benchmarks truthfulqa,halueval_qa,ragtruth \
  --threshold 5.0 \
  --output data/corpus/wikipedia_v1.0.audit.json
```

---

## Usage in Rust

### Loading Corpus

```rust
use pure_reason_core::wikipedia_corpus::WikipediaCorpus;

// Lazy loading (doesn't load until first query)
let corpus = WikipediaCorpus::new("data/corpus/wikipedia_v1.0.jsonl.gz")?;

// Get version
let version = corpus.version(); // "1.0"

// Query
let results = corpus.query("Albert Einstein", limit=10)?;
for article in results {
    println!("{}: {}", article.title, article.abstract);
}

// Check entity presence
if corpus.contains_entity("Albert Einstein")? {
    println!("Entity found in corpus");
}
```

### Caching Strategy

- **LRU Cache**: Max 1000 queries cached in memory
- **Cache Key**: Query string (lowercased, trimmed)
- **Cache Value**: Vec<Article> (top 10 results)
- **Eviction**: Least recently used when cache full

---

## Storage Requirements

| Component | Size | Description |
|-----------|------|-------------|
| Raw XML | ~15GB | Downloaded Wikipedia dump |
| Processed JSONL | ~8GB | Uncompressed corpus |
| Compressed JSONL | ~2GB | Gzipped corpus |
| BM25 Index | ~500MB | SQLite FTS5 index |
| **Total** | **~2.5GB** | Production deployment size |

---

## Update Cadence

### Weekly Updates (Recommended)

- Download latest Wikipedia dump every Monday
- Process and build index
- Run leak audit
- Deploy if audit passes
- Version: increment MINOR (e.g., 1.0 → 1.1)

### CI Integration

```yaml
# .github/workflows/corpus-update.yml
name: Weekly Corpus Update
on:
  schedule:
    - cron: '0 0 * * 1'  # Every Monday at midnight
jobs:
  update:
    runs-on: ubuntu-latest
    steps:
      - name: Download Wikipedia dump
      - name: Process corpus
      - name: Build index
      - name: Run leak audit
      - name: Deploy if audit passes
      - name: Commit version bump
```

---

## FAQ

### Q: Why not use Wikipedia API?
**A**: API has rate limits and requires internet access. Local corpus is faster, offline-capable, and reproducible.

### Q: Why only abstracts, not full articles?
**A**: Abstracts contain core facts (80/20 rule). Full articles are 100GB+, impractical for local deployment.

### Q: How to handle corpus updates in production?
**A**: Blue-green deployment. Keep v1.0 and v1.1 side-by-side, switch after validation.

### Q: What if Wikipedia removes an article?
**A**: Corpus is immutable once published. New versions may exclude removed articles.

### Q: How to add non-Wikipedia sources?
**A**: Extend schema with `source` field. Keep separate indices per source for provenance.

---

**END OF SCHEMA**
