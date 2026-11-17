"""SQLite schema, FTS5 indexing, zstd compression, and manifest generation."""

from __future__ import annotations

import hashlib
import json
import sqlite3
from datetime import datetime, timezone
from pathlib import Path

SCHEMA_VERSION = "1"

SCHEMA_SQL = """
CREATE TABLE IF NOT EXISTS meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    word TEXT NOT NULL UNIQUE COLLATE NOCASE,
    ipa_us TEXT,
    ipa_uk TEXT,
    frequency_rank INTEGER,
    variants TEXT,
    meaning_vi TEXT,
    sources TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS senses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id INTEGER NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    pos TEXT NOT NULL,
    definition_en TEXT,
    definition_vi TEXT,
    examples TEXT,
    tags TEXT,
    synonyms TEXT,
    antonyms TEXT
);

CREATE INDEX IF NOT EXISTS idx_senses_entry_id ON senses(entry_id);

CREATE TABLE IF NOT EXISTS word_forms (
    form TEXT NOT NULL COLLATE NOCASE,
    entry_id INTEGER NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    pos TEXT,
    morph_tag TEXT,
    PRIMARY KEY (form, entry_id)
);

CREATE INDEX IF NOT EXISTS idx_word_forms_form ON word_forms(form COLLATE NOCASE);

CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
    word,
    meaning_vi,
    definitions_en,
    content='',
    tokenize='unicode61'
);
"""


def init_dict_db(db_path: Path) -> sqlite3.Connection:
    """Create a fresh SQLite dictionary pack at db_path."""
    if db_path.exists():
        db_path.unlink()
    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(str(db_path))
    conn.execute("PRAGMA journal_mode = OFF;")
    conn.execute("PRAGMA synchronous = OFF;")
    conn.execute("PRAGMA temp_store = MEMORY;")
    conn.execute("PRAGMA foreign_keys = ON;")
    conn.executescript(SCHEMA_SQL)
    return conn


def finalize_dict_db(
    conn: sqlite3.Connection,
    profile: str,
    attribution: str,
) -> dict[str, int | str]:
    """Populate meta table, optimize FTS5, and VACUUM for compact distribution."""
    word_count = conn.execute("SELECT COUNT(*) FROM entries").fetchone()[0]
    sense_count = conn.execute("SELECT COUNT(*) FROM senses").fetchone()[0]
    form_count = conn.execute("SELECT COUNT(*) FROM word_forms").fetchone()[0]
    built_at = datetime.now(timezone.utc).isoformat()

    meta_rows = [
        ("schema_version", SCHEMA_VERSION),
        ("profile", profile),
        ("built_at", built_at),
        ("attribution", attribution),
        ("word_count", str(word_count)),
        ("sense_count", str(sense_count)),
        ("form_count", str(form_count)),
    ]
    conn.executemany(
        "INSERT OR REPLACE INTO meta (key, value) VALUES (?, ?)",
        meta_rows,
    )
    conn.execute("INSERT INTO entries_fts(entries_fts) VALUES('optimize');")
    conn.commit()
    conn.execute("VACUUM;")
    conn.close()

    return {
        "schemaVersion": SCHEMA_VERSION,
        "profile": profile,
        "builtAt": built_at,
        "wordCount": word_count,
        "senseCount": sense_count,
        "formCount": form_count,
    }


def sha256_file(path: Path) -> str:
    """Compute hex SHA-256 checksum of a file."""
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def compress_zstd(src_db: Path, dst_zst: Path, level: int = 15) -> Path:
    """Compress src_db to dst_zst using zstandard (if installed)."""
    try:
        import zstandard as zstd  # type: ignore[import-not-found]
    except ImportError as exc:
        raise RuntimeError(
            "zstandard package is required for .db.zst compression. "
            "Install via: pip install -r scripts/dict-builder/requirements.txt"
        ) from exc

    cctx = zstd.ZstdCompressor(level=level)
    with src_db.open("rb") as fin, dst_zst.open("wb") as fout:
        cctx.copy_stream(fin, fout)
    return dst_zst


def write_manifest(
    manifest_path: Path,
    pack_id: str,
    version: str,
    stats: dict[str, int | str],
    db_path: Path,
    zst_path: Path | None,
    attribution: str,
) -> Path:
    """Write manifest.json describing the built pack for GitHub Releases."""
    payload = {
        "id": pack_id,
        "version": version,
        "schemaVersion": stats["schemaVersion"],
        "profile": stats["profile"],
        "builtAt": stats["builtAt"],
        "wordCount": stats["wordCount"],
        "senseCount": stats["senseCount"],
        "formCount": stats["formCount"],
        "uncompressedBytes": db_path.stat().st_size,
        "uncompressedSha256": sha256_file(db_path),
        "compressedBytes": zst_path.stat().st_size if zst_path else None,
        "compressedSha256": sha256_file(zst_path) if zst_path else None,
        "fileName": zst_path.name if zst_path else db_path.name,
        "attribution": attribution,
    }
    manifest_path.write_text(
        json.dumps(payload, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    return manifest_path
