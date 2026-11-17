"""Build pre-indexed offline SQLite dictionary packs for V-Learning.

Usage:
    python3 scripts/dict-builder/build_dict.py --sample
    python3 scripts/dict-builder/build_dict.py --profile modern --output-dir scripts/dict-builder/dist
"""

from __future__ import annotations

import argparse
import json
import sqlite3
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from schema import compress_zstd, finalize_dict_db, init_dict_db, write_manifest
from sources import (
    ATTRIBUTION_TEXT,
    DEFAULT_URLS,
    download_file,
    iter_gcide_xml_zip,
    iter_jsonl,
    iter_kaikki_vi_english,
    iter_skypediacode_db,
    parse_cmudict,
    parse_oewn_xml_gz,
    parse_varcon_variants,
    sample_fixtures,
)

POS_MAP = {
    "n": "noun",
    "noun": "noun",
    "v": "verb",
    "verb": "verb",
    "adj": "adj",
    "adjective": "adj",
    "a": "adj",
    "s": "adj",
    "adv": "adv",
    "adverb": "adv",
    "r": "adv",
    "prep": "prep",
    "preposition": "prep",
    "conj": "conj",
    "conjunction": "conj",
    "pron": "pron",
    "pronoun": "pron",
    "intj": "interj",
    "interj": "interj",
    "interjection": "interj",
    "phrase": "phrase",
    "idiom": "phrase",
}

ARCHAIC_TAGS = {"archaic", "obsolete", "dated", "historical"}


def normalize_pos(raw: str | None) -> str:
    """Normalize part-of-speech labels across heterogeneous sources."""
    if not raw:
        return "other"
    cleaned = raw.strip().lower().rstrip(".")
    return POS_MAP.get(cleaned, cleaned if cleaned in POS_MAP.values() else "other")


@dataclass
class SenseItem:
    pos: str
    definition_en: str = ""
    definition_vi: str = ""
    examples: list[str] = field(default_factory=list)
    tags: list[str] = field(default_factory=list)
    synonyms: list[str] = field(default_factory=list)
    antonyms: list[str] = field(default_factory=list)


@dataclass
class WordRecord:
    word: str
    ipa_us: str | None = None
    ipa_uk: str | None = None
    frequency_rank: int | None = None
    variants: set[str] = field(default_factory=set)
    sources: set[str] = field(default_factory=set)
    senses: list[SenseItem] = field(default_factory=list)
    forms: dict[str, tuple[str, str]] = field(default_factory=dict)  # form -> (pos, morph_tag)


class DictionaryAccumulator:
    """In-memory fusion container before flushing into SQLite."""

    def __init__(self, profile: str = "modern") -> None:
        self.profile = profile
        self.entries: dict[str, WordRecord] = {}

    def get_or_create(self, word: str) -> WordRecord | None:
        cleaned = word.strip()
        if not cleaned or len(cleaned) > 120:
            return None
        key = cleaned.lower()
        if key not in self.entries:
            self.entries[key] = WordRecord(word=cleaned)
        return self.entries[key]

    def ingest_skypediacode(self, rows: Any) -> None:
        for item in rows:
            rec = self.get_or_create(item["word"])
            if not rec:
                continue
            rec.sources.add("skypediacode")
            if item.get("ipa_us") and not rec.ipa_us:
                rec.ipa_us = item["ipa_us"]
            if item.get("ipa_uk") and not rec.ipa_uk:
                rec.ipa_uk = item["ipa_uk"]

            for s in item.get("senses", []):
                pos = normalize_pos(s.get("pos"))
                vi = (s.get("definition_vi") or "").strip()
                ex = (s.get("example") or "").strip()
                if not vi:
                    continue
                rec.senses.append(
                    SenseItem(
                        pos=pos,
                        definition_vi=vi,
                        examples=[ex] if ex else [],
                    )
                )

    def ingest_kaikki_en(self, rows: Any) -> None:
        for obj in rows:
            word = obj.get("word")
            if not isinstance(word, str):
                continue
            rec = self.get_or_create(word)
            if not rec:
                continue

            pos = normalize_pos(obj.get("pos"))
            # Extract IPA
            for snd in obj.get("sounds", []) or []:
                ipa = snd.get("ipa")
                if not ipa:
                    continue
                tags = {t.upper() for t in (snd.get("tags") or [])}
                if "UK" in tags or "RP" in tags:
                    rec.ipa_uk = rec.ipa_uk or ipa
                else:
                    rec.ipa_us = rec.ipa_us or ipa

            # Extract inflections -> word_forms
            for frm in obj.get("forms", []) or []:
                f_text = (frm.get("form") or "").strip()
                if not f_text or f_text.lower() == rec.word.lower():
                    continue
                if " " in f_text:
                    continue
                morph = ",".join(frm.get("tags") or [])
                rec.forms.setdefault(f_text.lower(), (pos, morph))

            added_sense = False
            for s in obj.get("senses", []) or []:
                tags = [str(t).lower() for t in (s.get("tags") or [])]
                if self.profile == "modern" and any(t in ARCHAIC_TAGS for t in tags):
                    continue
                glosses = [g.strip() for g in (s.get("glosses") or []) if isinstance(g, str)]
                if not glosses:
                    continue
                # Skip pure "inflection of ..." stubs since forms are indexed in word_forms
                if len(glosses) == 1 and "inflection of" in glosses[0].lower():
                    continue

                examples = [
                    ex["text"].strip()
                    for ex in (s.get("examples") or [])[:2]
                    if isinstance(ex, dict) and ex.get("text")
                ]
                syns = [
                    x["word"].strip()
                    for x in (s.get("synonyms") or [])[:8]
                    if isinstance(x, dict) and x.get("word")
                ]
                ants = [
                    x["word"].strip()
                    for x in (s.get("antonyms") or [])[:6]
                    if isinstance(x, dict) and x.get("word")
                ]
                rec.senses.append(
                    SenseItem(
                        pos=pos,
                        definition_en="; ".join(glosses[:2]),
                        examples=examples,
                        tags=tags[:6],
                        synonyms=syns,
                        antonyms=ants,
                    )
                )
                added_sense = True

            if added_sense:
                rec.sources.add("kaikki-en")

    def ingest_kaikki_vi(self, rows: Any) -> None:
        for obj in rows:
            word = obj.get("word")
            if not isinstance(word, str):
                continue
            rec = self.get_or_create(word)
            if not rec:
                continue

            pos = normalize_pos(obj.get("pos"))
            for frm in obj.get("forms", []) or []:
                f_text = (frm.get("form") or "").strip()
                if f_text and " " not in f_text and f_text.lower() != rec.word.lower():
                    rec.forms.setdefault(f_text.lower(), (pos, ",".join(frm.get("tags") or [])))

            added = False
            for s in obj.get("senses", []) or []:
                tags = [str(t).lower() for t in (s.get("tags") or [])]
                if self.profile == "modern" and any(t in ARCHAIC_TAGS for t in tags):
                    continue
                glosses = [g.strip() for g in (s.get("glosses") or []) if isinstance(g, str)]
                if not glosses:
                    continue
                vi_text = "; ".join(glosses[:2])
                if any(existing.definition_vi == vi_text for existing in rec.senses):
                    continue
                rec.senses.append(
                    SenseItem(
                        pos=pos,
                        definition_vi=vi_text,
                        tags=tags[:6],
                    )
                )
                added = True
            if added:
                rec.sources.add("kaikki-vi")

    def ingest_oewn(self, oewn_map: dict[str, list[dict[str, Any]]]) -> None:
        for key, senses in oewn_map.items():
            if not senses:
                continue
            rec = self.get_or_create(senses[0]["word"])
            if not rec:
                continue
            rec.sources.add("oewn")
            for s in senses[:6]:
                pos = normalize_pos(s.get("pos"))
                rec.senses.append(
                    SenseItem(
                        pos=pos,
                        definition_en=s.get("definition_en", ""),
                        examples=s.get("examples", []),
                        synonyms=s.get("synonyms", []),
                        antonyms=s.get("antonyms", []),
                    )
                )

    def ingest_gcide(self, rows: Any) -> None:
        if self.profile != "full":
            return
        for obj in rows:
            rec = self.get_or_create(obj["word"])
            if not rec:
                continue
            # Only fill entries not already covered by modern sources
            if rec.senses:
                continue
            rec.sources.add("gcide")
            rec.senses.append(
                SenseItem(
                    pos=normalize_pos(obj.get("pos")),
                    definition_en=obj["definition_en"],
                    tags=["archaic"],
                )
            )

    def apply_cmudict(self, cmu_map: dict[str, str]) -> None:
        for key, rec in self.entries.items():
            if not rec.ipa_us and key in cmu_map:
                rec.ipa_us = cmu_map[key]
                rec.sources.add("cmudict")

    def apply_variants(self, variants_map: dict[str, str]) -> None:
        """Register UK/US spelling variants into word_forms (e.g. colour -> color)."""
        for variant, canonical in variants_map.items():
            rec = self.entries.get(canonical.lower())
            if not rec:
                continue
            rec.variants.add(variant)
            rec.forms.setdefault(variant.lower(), ("other", "spelling_variant"))

    def flush_to_sqlite(self, conn: sqlite3.Connection) -> None:
        """Write accumulated entries, senses, word_forms, and FTS5 rows into SQLite."""
        for key in sorted(self.entries.keys()):
            rec = self.entries[key]
            if not rec.senses:
                continue

            vi_summary = " | ".join(
                dict.fromkeys(s.definition_vi for s in rec.senses if s.definition_vi)
            )[:500]
            en_summary = " | ".join(
                dict.fromkeys(s.definition_en for s in rec.senses if s.definition_en)
            )[:600]

            cur = conn.execute(
                """
                INSERT INTO entries (word, ipa_us, ipa_uk, frequency_rank, variants, meaning_vi, sources)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    rec.word,
                    rec.ipa_us,
                    rec.ipa_uk,
                    rec.frequency_rank,
                    json.dumps(sorted(rec.variants), ensure_ascii=False) if rec.variants else None,
                    vi_summary or None,
                    json.dumps(sorted(rec.sources), ensure_ascii=False),
                ),
            )
            entry_id = cur.lastrowid

            conn.executemany(
                """
                INSERT INTO senses (entry_id, pos, definition_en, definition_vi, examples, tags, synonyms, antonyms)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                """,
                [
                    (
                        entry_id,
                        s.pos,
                        s.definition_en or None,
                        s.definition_vi or None,
                        json.dumps(s.examples, ensure_ascii=False) if s.examples else None,
                        json.dumps(s.tags, ensure_ascii=False) if s.tags else None,
                        json.dumps(s.synonyms, ensure_ascii=False) if s.synonyms else None,
                        json.dumps(s.antonyms, ensure_ascii=False) if s.antonyms else None,
                    )
                    for s in rec.senses
                ],
            )

            if rec.forms:
                conn.executemany(
                    """
                    INSERT OR IGNORE INTO word_forms (form, entry_id, pos, morph_tag)
                    VALUES (?, ?, ?, ?)
                    """,
                    [(form, entry_id, pos, morph) for form, (pos, morph) in rec.forms.items()],
                )

            conn.execute(
                "INSERT INTO entries_fts(rowid, word, meaning_vi, definitions_en) VALUES (?, ?, ?, ?)",
                (entry_id, rec.word, vi_summary, en_summary),
            )


def build_pack(
    profile: str,
    output_dir: Path,
    cache_dir: Path,
    version: str,
    sample: bool = False,
    limit: int | None = None,
    compress: bool = True,
    kaikki_en_path: Path | None = None,
    with_kaikki_en: bool = False,
    with_oewn: bool = True,
    with_kaikki_vi: bool = True,
    with_gcide: bool = False,
) -> Path:
    """Run the end-to-end dictionary build pipeline."""
    acc = DictionaryAccumulator(profile=profile)

    if sample:
        fx = sample_fixtures()
        acc.ingest_skypediacode(fx["skypediacode"])
        acc.ingest_kaikki_en(fx["kaikki_en"])
        acc.apply_cmudict(fx["cmudict"])
        acc.apply_variants(fx["variants"])
    else:
        skype_db = download_file(DEFAULT_URLS["skypediacode_db"], cache_dir / "dictionary_en_vi.db")
        acc.ingest_skypediacode(iter_skypediacode_db(skype_db, limit=limit))

        if with_kaikki_vi:
            vi_path = download_file(
                DEFAULT_URLS["kaikki_vi"], cache_dir / "vi-wiktextract.jsonl.gz"
            )
            acc.ingest_kaikki_vi(iter_kaikki_vi_english(vi_path, limit=limit))

        if with_oewn:
            oewn_path = download_file(DEFAULT_URLS["oewn_xml_gz"], cache_dir / "oewn.xml.gz")
            acc.ingest_oewn(parse_oewn_xml_gz(oewn_path, limit=limit))

        if with_kaikki_en and not (kaikki_en_path and kaikki_en_path.exists()):
            kaikki_en_path = download_file(
                DEFAULT_URLS["kaikki_en"], cache_dir / "kaikki-en.jsonl"
            )
        if kaikki_en_path and kaikki_en_path.exists():
            acc.ingest_kaikki_en(iter_jsonl(kaikki_en_path, limit=limit))

        if profile == "full" and with_gcide:
            gcide_path = download_file(DEFAULT_URLS["gcide_xml_zip"], cache_dir / "gcide_xml.zip")
            acc.ingest_gcide(iter_gcide_xml_zip(gcide_path, limit=limit))

        cmu_path = download_file(DEFAULT_URLS["cmudict"], cache_dir / "cmudict.dict")
        acc.apply_cmudict(parse_cmudict(cmu_path))

        varcon_path = download_file(DEFAULT_URLS["scowl_varcon"], cache_dir / "varcon.txt")
        acc.apply_variants(parse_varcon_variants(varcon_path))

    pack_id = f"dict-{'sample' if sample else profile}"
    db_path = output_dir / f"{pack_id}.db"
    conn = init_dict_db(db_path)
    acc.flush_to_sqlite(conn)
    stats = finalize_dict_db(conn, profile=profile, attribution=ATTRIBUTION_TEXT)

    zst_path: Path | None = None
    if compress:
        try:
            zst_path = compress_zstd(db_path, output_dir / f"{pack_id}.db.zst")
        except RuntimeError as exc:
            print(f"[warn] Skipping zstd compression: {exc}")

    manifest_path = write_manifest(
        output_dir / f"{pack_id}.manifest.json",
        pack_id=pack_id,
        version=version,
        stats=stats,
        db_path=db_path,
        zst_path=zst_path,
        attribution=ATTRIBUTION_TEXT,
    )
    print(f"[done] Built {db_path} ({stats['wordCount']} words, {stats['senseCount']} senses, {stats['formCount']} forms)")
    print(f"[done] Manifest: {manifest_path}")
    return db_path


def main() -> None:
    parser = argparse.ArgumentParser(description="V-Learning Offline Dictionary Pack Builder")
    parser.add_argument("--profile", choices=["modern", "full"], default="modern")
    parser.add_argument("--version", default="v1.0.0")
    parser.add_argument("--output-dir", type=Path, default=Path("scripts/dict-builder/dist"))
    parser.add_argument("--cache-dir", type=Path, default=Path("scripts/dict-builder/cache"))
    parser.add_argument("--kaikki-en", type=Path, default=None, help="Optional local Kaikki English JSONL(.gz)")
    parser.add_argument("--with-kaikki-en", action="store_true", help="Download and ingest Kaikki English (~2GB)")
    parser.add_argument("--skip-oewn", action="store_true", help="Skip Open English WordNet")
    parser.add_argument("--skip-kaikki-vi", action="store_true", help="Skip Kaikki Vietnamese")
    parser.add_argument("--with-gcide", action="store_true", help="Include GCIDE in full profile")
    parser.add_argument("--limit", type=int, default=None, help="Limit records per source for quick testing")
    parser.add_argument("--sample", action="store_true", help="Build offline sample pack without downloading")
    parser.add_argument("--no-compress", action="store_true", help="Skip .db.zst compression")
    args = parser.parse_args()

    build_pack(
        profile=args.profile,
        output_dir=args.output_dir,
        cache_dir=args.cache_dir,
        version=args.version,
        sample=args.sample,
        limit=args.limit,
        compress=not args.no_compress,
        kaikki_en_path=args.kaikki_en,
        with_kaikki_en=args.with_kaikki_en,
        with_oewn=not args.skip_oewn,
        with_kaikki_vi=not args.skip_kaikki_vi,
        with_gcide=args.with_gcide,
    )


if __name__ == "__main__":
    main()
