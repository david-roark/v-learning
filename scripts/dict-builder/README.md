# V-Learning Dictionary Builder (`scripts/dict-builder`)

Offline ETL pipeline to fuse open, license-clean English & Vietnamese dictionary datasets into a pre-indexed, FTS5-enabled SQLite pack (`dict.db` / `dict.db.zst`) for **V-Learning**.

## Architecture

- **Separate Pack DB (`dict.db`)**: Distributed separately from the user's `vlearning.db` (SRS cards/review history). Users download a `.db.zst` pack from GitHub Releases and query it immediately with zero local re-indexing.
- **Word Forms (`word_forms`)**: Maps inflections (`ran → run`, `geese → goose`) and UK/US variants (`colour → color`) to canonical entries so clicking any inflected word in the Reader resolves cleanly.
- **Profiles**:
  - `modern`: Prioritizes contemporary English, skipping `archaic` / `obsolete` / `dated` senses.
  - `full`: Includes historical, archaic, slang, and specialized senses.

## Data Sources & Attribution (`CC BY-SA 4.0` / Open)

- **Skypediacode English-Vietnamese Dictionary** (CC BY-SA 4.0; upstream MinhQND & OVDP)
- **Kaikki.org** (English & Vietnamese Wiktionary via `wiktextract`)
- **Open English WordNet (OEWN)**
- **CMU Pronouncing Dictionary (CMUdict)**
- **SCOWL / ESDB Wordlist**

## Quick Usage

```bash
# 1. Run unit tests
python3 -m unittest scripts/dict-builder/test_builder.py

# 2. Smoke-test offline sample pack (no downloads required)
python3 scripts/dict-builder/build_dict.py --sample --no-compress

# 3. Build real modern pack (install requirements first)
pip install -r scripts/dict-builder/requirements.txt
python3 scripts/dict-builder/build_dict.py --profile modern --version v1.0.0
```
