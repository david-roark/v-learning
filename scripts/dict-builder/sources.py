"""Source downloaders (with local cache) and streaming parsers for V-Learning dict-builder."""

from __future__ import annotations

import gzip
import io
import json
import sqlite3
import xml.etree.ElementTree as ET
import zipfile
from collections.abc import Iterator
from pathlib import Path
from typing import Any

from arpabet import arpabet_to_ipa

DEFAULT_URLS = {
    "skypediacode_db": (
        "https://github.com/skypediacode/english-vietnamese-dictionary/"
        "raw/main/dictionary_en_vi.db"
    ),
    "cmudict": "https://raw.githubusercontent.com/cmusphinx/cmudict/master/cmudict.dict",
    "kaikki_en": "https://kaikki.org/dictionary/English/kaikki.org-dictionary-English.jsonl",
    "kaikki_vi": "https://kaikki.org/viwiktionary/raw-wiktextract-data.jsonl.gz",
    "oewn_xml_gz": "https://en-word.net/static/english-wordnet-2025.xml.gz",
    "scowl_varcon": "https://raw.githubusercontent.com/en-wl/wordlist/master/varcon/varcon.txt",
    "gcide_xml_zip": "https://www.ibiblio.org/webster/gcide_xml-0.53.zip",
}

ATTRIBUTION_TEXT = (
    "Data sources (CC BY-SA 4.0 / Open): "
    "Skypediacode English-Vietnamese Dictionary (MinhQND & OVDP), "
    "Kaikki.org (English & Vietnamese Wiktionary via wiktextract), "
    "Open English WordNet (OEWN), "
    "CMU Pronouncing Dictionary (CMUdict), "
    "SCOWL / varcon Wordlist, "
    "GCIDE (GNU Collaborative International Dictionary of English)."
)


def download_file(url: str, dest: Path, force: bool = False) -> Path:
    """Download a file to cache if not already present."""
    if dest.exists() and not force:
        return dest

    try:
        import requests  # type: ignore[import-not-found]
    except ImportError as exc:
        raise RuntimeError(
            "requests package is required for downloading sources. "
            "Install via: pip install -r scripts/dict-builder/requirements.txt"
        ) from exc

    dest.parent.mkdir(parents=True, exist_ok=True)
    tmp = dest.with_suffix(dest.suffix + ".part")
    print(f"[download] {url} -> {dest}")
    with requests.get(url, stream=True, timeout=120) as resp:
        resp.raise_for_status()
        with tmp.open("wb") as f:
            for chunk in resp.iter_content(chunk_size=1024 * 256):
                if chunk:
                    f.write(chunk)
    tmp.replace(dest)
    return dest


def iter_jsonl(path: Path, limit: int | None = None) -> Iterator[dict[str, Any]]:
    """Stream JSONL or JSONL.gz records."""
    opener = gzip.open if path.suffix == ".gz" else Path.open
    count = 0
    with opener(path, "rt", encoding="utf-8", errors="replace") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                yield json.loads(line)
            except json.JSONDecodeError:
                continue
            count += 1
            if limit and count >= limit:
                break


def iter_kaikki_vi_english(path: Path, limit: int | None = None) -> Iterator[dict[str, Any]]:
    """Stream English headwords (with Vietnamese glosses) from viwiktionary JSONL(.gz)."""
    emitted = 0
    for obj in iter_jsonl(path):
        lang_code = (obj.get("lang_code") or "").lower()
        if lang_code != "en":
            continue
        yield obj
        emitted += 1
        if limit and emitted >= limit:
            break


def parse_cmudict(path: Path) -> dict[str, str]:
    """Parse cmudict.dict into {word_lower: '/ipa/'}."""
    ipa_map: dict[str, str] = {}
    with path.open("r", encoding="utf-8", errors="replace") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith(";;;"):
                continue
            parts = line.split(maxsplit=1)
            if len(parts) != 2:
                continue
            raw_word, phonemes = parts
            word = raw_word.split("(")[0].lower()
            if word in ipa_map:
                continue
            phonemes = phonemes.split("#")[0].strip()
            ipa = arpabet_to_ipa(phonemes)
            if ipa:
                ipa_map[word] = ipa
    return ipa_map


def parse_varcon_variants(path: Path) -> dict[str, str]:
    """Parse SCOWL varcon.txt into {variant_lower: primary_us_spelling}."""
    variants: dict[str, str] = {}
    with path.open("r", encoding="utf-8", errors="replace") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            # varcon lines look like: "A: color / B: colour / C: colour"
            segments = [seg.strip() for seg in line.split("/") if ":" in seg]
            if len(segments) < 2:
                continue
            us_word = None
            all_words: list[str] = []
            for seg in segments:
                tags_part, word_part = seg.split(":", 1)
                word = word_part.split("|")[0].strip().split()[0]
                if not word:
                    continue
                all_words.append(word)
                if "A" in tags_part and us_word is None:
                    us_word = word
            canonical = us_word or (all_words[0] if all_words else None)
            if not canonical:
                continue
            for w in all_words:
                if w.lower() != canonical.lower():
                    variants.setdefault(w.lower(), canonical)
    return variants


def parse_oewn_xml_gz(path: Path, limit: int | None = None) -> dict[str, list[dict[str, Any]]]:
    """Parse Open English WordNet WN-LMF .xml(.gz) into {word_lower: [sense_dicts]}."""
    opener = gzip.open if path.suffix == ".gz" else Path.open
    synsets: dict[str, dict[str, Any]] = {}
    synset_words: dict[str, list[str]] = {}
    entry_senses: list[tuple[str, str, str, list[str]]] = []  # (word, pos, synset_id, ant_sense_ids)
    sense_to_word: dict[str, str] = {}

    with opener(path, "rb") as fh:
        context = ET.iterparse(fh, events=("end",))
        for _, elem in context:
            tag = elem.tag.rsplit("}", 1)[-1]
            if tag == "Synset":
                sid = elem.attrib.get("id", "")
                pos = elem.attrib.get("partOfSpeech", "other")
                defs = [
                    (d.text or "").strip()
                    for d in elem.findall(".//{*}Definition")
                    if (d.text or "").strip()
                ]
                exs = [
                    (e.text or "").strip()
                    for e in elem.findall(".//{*}Example")
                    if (e.text or "").strip()
                ]
                if sid:
                    synsets[sid] = {"pos": pos, "defs": defs, "exs": exs}
                elem.clear()
            elif tag == "LexicalEntry":
                lemma_el = elem.find("{*}Lemma")
                if lemma_el is not None:
                    word = (lemma_el.attrib.get("writtenForm") or "").strip()
                    pos = lemma_el.attrib.get("partOfSpeech", "other")
                    if word:
                        for sense_el in elem.findall("{*}Sense"):
                            sense_id = sense_el.attrib.get("id", "")
                            syn_id = sense_el.attrib.get("synset", "")
                            if sense_id:
                                sense_to_word[sense_id] = word
                            if syn_id:
                                synset_words.setdefault(syn_id, []).append(word)
                                ant_ids = [
                                    rel.attrib.get("target", "")
                                    for rel in sense_el.findall("{*}SenseRelation")
                                    if rel.attrib.get("relType") == "antonym"
                                    and rel.attrib.get("target")
                                ]
                                entry_senses.append((word, pos, syn_id, ant_ids))
                elem.clear()
                if limit and len(entry_senses) >= limit * 2:
                    break

    result: dict[str, list[dict[str, Any]]] = {}
    for word, pos, syn_id, ant_ids in entry_senses:
        syn_data = synsets.get(syn_id)
        if not syn_data or not syn_data["defs"]:
            continue
        syns = [w for w in synset_words.get(syn_id, []) if w.lower() != word.lower()][:8]
        ants = [
            sense_to_word[aid]
            for aid in ant_ids
            if aid in sense_to_word and sense_to_word[aid].lower() != word.lower()
        ][:6]
        result.setdefault(word.lower(), []).append(
            {
                "word": word,
                "pos": pos,
                "definition_en": "; ".join(syn_data["defs"][:2]),
                "examples": syn_data["exs"][:2],
                "synonyms": syns,
                "antonyms": ants,
            }
        )
    return result


def iter_gcide_xml_zip(path: Path, limit: int | None = None) -> Iterator[dict[str, Any]]:
    """Stream entries from GCIDE XML zip (for archaic/historical vocabulary in full profile)."""
    emitted = 0
    with zipfile.ZipFile(path, "r") as zf:
        xml_names = sorted(n for n in zf.namelist() if n.lower().endswith(".xml"))
        for name in xml_names:
            with zf.open(name) as raw:
                data = raw.read()
            try:
                context = ET.iterparse(io.BytesIO(data), events=("end",))
                for _, elem in context:
                    if elem.tag.rsplit("}", 1)[-1] != "p":
                        continue
                    ent_el = elem.find(".//{*}ent")
                    def_el = elem.find(".//{*}def")
                    if ent_el is not None and def_el is not None:
                        word = "".join(ent_el.itertext()).strip()
                        defn = " ".join("".join(def_el.itertext()).split())
                        pos_el = elem.find(".//{*}pos")
                        pos = "".join(pos_el.itertext()).strip() if pos_el is not None else "other"
                        if word and defn:
                            yield {"word": word, "pos": pos, "definition_en": defn}
                            emitted += 1
                            if limit and emitted >= limit:
                                return
                    elem.clear()
            except ET.ParseError:
                continue


def iter_skypediacode_db(db_path: Path, limit: int | None = None) -> Iterator[dict[str, Any]]:
    """Read normalized entries from Skypediacode SQLite database."""
    conn = sqlite3.connect(str(db_path))
    conn.row_factory = sqlite3.Row

    query = """
        SELECT
            w.id AS word_id,
            w.word AS word,
            d.definition AS definition,
            d.pos AS pos,
            wd.example AS example
        FROM words w
        JOIN word_definitions wd ON wd.word_id = w.id
        JOIN definitions d ON d.id = wd.definition_id
        ORDER BY w.id
    """
    if limit:
        query += f" LIMIT {int(limit) * 3}"

    pron_map: dict[int, tuple[str | None, str | None]] = {}
    for row in conn.execute("SELECT word_id, ipa, region FROM pronunciations"):
        wid = int(row["word_id"])
        ipa = (row["ipa"] or "").strip()
        region = (row["region"] or "").upper()
        if not ipa:
            continue
        us, uk = pron_map.get(wid, (None, None))
        if "UK" in region or "GB" in region:
            uk = uk or ipa
        else:
            us = us or ipa
        pron_map[wid] = (us, uk)

    current_id: int | None = None
    current_word = ""
    senses: list[dict[str, Any]] = []
    emitted = 0

    for row in conn.execute(query):
        wid = int(row["word_id"])
        if current_id is not None and wid != current_id:
            us, uk = pron_map.get(current_id, (None, None))
            yield {
                "word": current_word,
                "ipa_us": us,
                "ipa_uk": uk,
                "senses": senses,
            }
            emitted += 1
            if limit and emitted >= limit:
                conn.close()
                return
            senses = []

        current_id = wid
        current_word = (row["word"] or "").strip()
        senses.append(
            {
                "pos": (row["pos"] or "other").strip(),
                "definition_vi": (row["definition"] or "").strip(),
                "example": (row["example"] or "").strip(),
            }
        )

    if current_id is not None and current_word:
        us, uk = pron_map.get(current_id, (None, None))
        yield {
            "word": current_word,
            "ipa_us": us,
            "ipa_uk": uk,
            "senses": senses,
        }

    conn.close()


def sample_fixtures() -> dict[str, Any]:
    """Self-contained sample fixtures for offline smoke testing (--sample)."""
    return {
        "skypediacode": [
            {
                "word": "run",
                "ipa_us": "/rʌn/",
                "ipa_uk": "/rʌn/",
                "senses": [
                    {
                        "pos": "verb",
                        "definition_vi": "chạy; vận hành, điều hành",
                        "example": "She runs every morning before work.",
                    },
                    {
                        "pos": "noun",
                        "definition_vi": "cuộc chạy; lượt chạy",
                        "example": "He went for a quick run in the park.",
                    },
                ],
            },
            {
                "word": "ubiquitous",
                "ipa_us": "/juːˈbɪk.wɪ.təs/",
                "ipa_uk": "/juːˈbɪk.wɪ.təs/",
                "senses": [
                    {
                        "pos": "adjective",
                        "definition_vi": "có mặt ở khắp mọi nơi, phổ biến rộng rãi",
                        "example": "Smartphones have become ubiquitous in modern life.",
                    }
                ],
            },
            {
                "word": "color",
                "ipa_us": "/ˈkʌl.ɚ/",
                "ipa_uk": "/ˈkʌl.ə/",
                "senses": [
                    {
                        "pos": "noun",
                        "definition_vi": "màu sắc",
                        "example": "Blue is my favorite color.",
                    }
                ],
            },
        ],
        "kaikki_en": [
            {
                "word": "run",
                "pos": "verb",
                "sounds": [{"ipa": "/rʌn/", "tags": ["US"]}],
                "forms": [
                    {"form": "runs", "tags": ["present", "singular", "third-person"]},
                    {"form": "running", "tags": ["participle", "present"]},
                    {"form": "ran", "tags": ["past"]},
                ],
                "senses": [
                    {
                        "glosses": ["To move swiftly on foot."],
                        "examples": [{"text": "I can run five kilometers."}],
                        "synonyms": [{"word": "sprint"}, {"word": "jog"}],
                        "antonyms": [{"word": "walk"}],
                        "tags": [],
                    }
                ],
            },
            {
                "word": "ubiquitous",
                "pos": "adj",
                "senses": [
                    {
                        "glosses": ["Being everywhere at once; omnipresent."],
                        "examples": [{"text": "Coffee shops are ubiquitous in the city."}],
                        "synonyms": [{"word": "omnipresent"}, {"word": "pervasive"}],
                        "antonyms": [{"word": "rare"}],
                        "tags": ["formal"],
                    }
                ],
            },
            {
                "word": "ye",
                "pos": "pron",
                "senses": [
                    {
                        "glosses": ["You (plural or formal singular)."],
                        "tags": ["archaic"],
                    }
                ],
            },
        ],
        "cmudict": {
            "pragmatic": "/pɹæɡˈmætɪk/",
            "resilient": "/ɹɪˈzɪljənt/",
        },
        "variants": {
            "colour": "color",
            "favourite": "favorite",
            "analyse": "analyze",
        },
    }
