#!/usr/bin/env python3
"""
Build BM25 index from Wikipedia JSONL corpus.

Usage:
    python3 scripts/build_bm25_index.py \
        --input data/corpus/wikipedia_v1.0.jsonl \
        --output data/corpus/wikipedia_v1.0.index.db
"""

import argparse
import json
import sqlite3

SQL_SCHEMA = """
CREATE TABLE IF NOT EXISTS articles (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    abstract TEXT NOT NULL,
    url TEXT NOT NULL,
    categories TEXT NOT NULL,
    entities TEXT NOT NULL,
    last_modified TEXT NOT NULL,
    word_count INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_title ON articles(title);
CREATE INDEX IF NOT EXISTS idx_entities ON articles(entities);

CREATE VIRTUAL TABLE IF NOT EXISTS articles_fts USING fts5(
    id UNINDEXED,
    title,
    abstract,
    content=articles,
    content_rowid=rowid
);

CREATE TRIGGER IF NOT EXISTS articles_ai AFTER INSERT ON articles BEGIN
  INSERT INTO articles_fts(rowid, id, title, abstract)
  VALUES (new.rowid, new.id, new.title, new.abstract);
END;

CREATE TRIGGER IF NOT EXISTS articles_ad AFTER DELETE ON articles BEGIN
  DELETE FROM articles_fts WHERE rowid = old.rowid;
END;

CREATE TRIGGER IF NOT EXISTS articles_au AFTER UPDATE ON articles BEGIN
  DELETE FROM articles_fts WHERE rowid = old.rowid;
  INSERT INTO articles_fts(rowid, id, title, abstract)
  VALUES (new.rowid, new.id, new.title, new.abstract);
END;
"""


def build_index(jsonl_file: str, db_file: str, batch_size: int = 10000):
    """
    Build SQLite BM25 index from JSONL corpus.

    Args:
        jsonl_file: Path to Wikipedia JSONL corpus
        db_file: Path to output SQLite database
        batch_size: Number of records to insert per transaction
    """
    print(f"Building index from {jsonl_file}...")
    print(f"Output: {db_file}")

    # Connect to database
    conn = sqlite3.connect(db_file)
    cursor = conn.cursor()

    # Create schema
    print("Creating schema...")
    cursor.executescript(SQL_SCHEMA)
    conn.commit()

    # Insert records
    print("Inserting records...")
    batch = []
    total_count = 0

    with open(jsonl_file, encoding="utf-8") as f:
        for line in f:
            record = json.loads(line)

            # Convert lists to JSON strings for storage
            categories_json = json.dumps(record["categories"])
            entities_json = json.dumps(record["entities"])

            batch.append(
                (
                    record["id"],
                    record["title"],
                    record["abstract"],
                    record["url"],
                    categories_json,
                    entities_json,
                    record["last_modified"],
                    record["word_count"],
                )
            )

            if len(batch) >= batch_size:
                cursor.executemany(
                    "INSERT OR REPLACE INTO articles VALUES (?, ?, ?, ?, ?, ?, ?, ?)", batch
                )
                conn.commit()
                total_count += len(batch)
                print(f"Inserted {total_count:,} articles...")
                batch = []

    # Insert remaining batch
    if batch:
        cursor.executemany("INSERT OR REPLACE INTO articles VALUES (?, ?, ?, ?, ?, ?, ?, ?)", batch)
        conn.commit()
        total_count += len(batch)

    print(f"Inserted {total_count:,} articles total.")

    # Optimize database
    print("Optimizing database...")
    cursor.execute("VACUUM")
    cursor.execute("ANALYZE")
    conn.commit()

    # Close connection
    conn.close()

    print(f"\nCompleted! Index created: {db_file}")
    print(f"Total articles: {total_count:,}")


def main():
    parser = argparse.ArgumentParser(description="Build BM25 index from Wikipedia JSONL")
    parser.add_argument("--input", required=True, help="Path to JSONL corpus")
    parser.add_argument("--output", required=True, help="Path to output SQLite database")
    parser.add_argument("--batch-size", type=int, default=10000, help="Batch size for inserts")

    args = parser.parse_args()

    build_index(args.input, args.output, args.batch_size)


if __name__ == "__main__":
    main()
