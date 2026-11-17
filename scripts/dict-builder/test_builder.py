"""Unit tests for V-Learning dictionary builder pipeline."""

from __future__ import annotations

import gzip
import json
import sqlite3
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from arpabet import arpabet_to_ipa
from build_dict import DictionaryAccumulator, build_pack, normalize_pos
from sources import (
    iter_gcide_xml_zip,
    iter_kaikki_vi_english,
    parse_oewn_xml_gz,
    parse_varcon_variants,
)


class DictBuilderTests(unittest.TestCase):
    def test_arpabet_to_ipa_handles_stress_and_schwa(self) -> None:
        self.assertEqual(arpabet_to_ipa("R AH1 N"), "/ɹˈʌn/")
        self.assertEqual(arpabet_to_ipa("AH0 B AW1 T"), "/əbˈaʊt/")

    def test_normalize_pos_unifies_variants(self) -> None:
        self.assertEqual(normalize_pos("n."), "noun")
        self.assertEqual(normalize_pos("Adjective"), "adj")
        self.assertEqual(normalize_pos("INTJ"), "interj")

    def test_sample_build_creates_valid_sqlite_with_word_forms_and_fts(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            out_dir = Path(tmp) / "dist"
            cache_dir = Path(tmp) / "cache"
            db_path = build_pack(
                profile="modern",
                output_dir=out_dir,
                cache_dir=cache_dir,
                version="v1.0.0-test",
                sample=True,
                compress=False,
            )
            self.assertTrue(db_path.exists())
            self.assertTrue((out_dir / "dict-sample.manifest.json").exists())

            conn = sqlite3.connect(str(db_path))
            # Check lemmatization via word_forms: 'ran' -> 'run', 'colour' -> 'color'
            ran_word = conn.execute(
                """
                SELECT e.word FROM word_forms wf
                JOIN entries e ON e.id = wf.entry_id
                WHERE wf.form = 'ran'
                """
            ).fetchone()
            self.assertEqual(ran_word[0], "run")

            colour_word = conn.execute(
                """
                SELECT e.word FROM word_forms wf
                JOIN entries e ON e.id = wf.entry_id
                WHERE wf.form = 'colour'
                """
            ).fetchone()
            self.assertEqual(colour_word[0], "color")

            # 'ye' is tagged archaic and should be excluded in 'modern' profile
            ye_row = conn.execute("SELECT id FROM entries WHERE word = 'ye'").fetchone()
            self.assertIsNone(ye_row)

            # FTS5 search works
            fts_hit = conn.execute(
                "SELECT e.word FROM entries_fts f JOIN entries e ON e.id = f.rowid WHERE entries_fts MATCH 'omnipresent'"
            ).fetchone()
            self.assertEqual(fts_hit[0], "ubiquitous")
            conn.close()

    def test_varcon_and_oewn_xml_and_kaikki_vi_fusion(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            varcon = tmp_dir / "varcon.txt"
            varcon.write_text("A: color / B: colour / C: colour\n", encoding="utf-8")
            variants = parse_varcon_variants(varcon)
            self.assertEqual(variants.get("colour"), "color")

            oewn_xml = tmp_dir / "oewn.xml"
            oewn_xml.write_text(
                """<LexicalResource>
                  <Lexicon id="oewn">
                    <LexicalEntry id="e1">
                      <Lemma writtenForm="fast" partOfSpeech="a"/>
                      <Sense id="s1" synset="syn1">
                        <SenseRelation target="s2" relType="antonym"/>
                      </Sense>
                    </LexicalEntry>
                    <LexicalEntry id="e2">
                      <Lemma writtenForm="quick" partOfSpeech="a"/>
                      <Sense id="s3" synset="syn1"/>
                    </LexicalEntry>
                    <LexicalEntry id="e3">
                      <Lemma writtenForm="slow" partOfSpeech="a"/>
                      <Sense id="s2" synset="syn2"/>
                    </LexicalEntry>
                    <Synset id="syn1" partOfSpeech="a">
                      <Definition>Acting or moving quickly.</Definition>
                      <Example>A fast car.</Example>
                    </Synset>
                    <Synset id="syn2" partOfSpeech="a">
                      <Definition>Taking a long time.</Definition>
                    </Synset>
                  </Lexicon>
                </LexicalResource>""",
                encoding="utf-8",
            )
            oewn_map = parse_oewn_xml_gz(oewn_xml)
            self.assertIn("fast", oewn_map)
            self.assertIn("quick", oewn_map["fast"][0]["synonyms"])
            self.assertIn("slow", oewn_map["fast"][0]["antonyms"])

            acc = DictionaryAccumulator(profile="modern")
            acc.ingest_oewn(oewn_map)
            acc.ingest_kaikki_vi(
                [
                    {
                        "word": "fast",
                        "pos": "adj",
                        "senses": [{"glosses": ["nhanh, mau lẹ"], "tags": []}],
                    }
                ]
            )
            self.assertIn("oewn", acc.entries["fast"].sources)
            self.assertIn("kaikki-vi", acc.entries["fast"].sources)
            self.assertEqual(len(acc.entries["fast"].senses), 2)

    def test_kaikki_vi_filters_non_english_and_gcide_respects_profile(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            vi_jsonl = tmp_dir / "vi.jsonl.gz"
            with gzip.open(vi_jsonl, "wt", encoding="utf-8") as f:
                f.write(
                    json.dumps(
                        {
                            "word": "chiến tranh",
                            "lang_code": "vi",
                            "lang": "Tiếng Việt",
                            "pos": "noun",
                            "senses": [{"glosses": ["Xung đột vũ trang"]}],
                        }
                    )
                    + "\n"
                )
                f.write(
                    json.dumps(
                        {
                            "word": "give up",
                            "lang_code": "en",
                            "lang": "Tiếng Anh",
                            "pos": "verb",
                            "senses": [{"glosses": ["từ bỏ"]}],
                        }
                    )
                    + "\n"
                )

            en_entries = list(iter_kaikki_vi_english(vi_jsonl))
            self.assertEqual([e["word"] for e in en_entries], ["give up"])

            gcide_zip = tmp_dir / "gcide.zip"
            with zipfile.ZipFile(gcide_zip, "w") as zf:
                zf.writestr(
                    "gcide_w.xml",
                    "<gcide><p><ent>whilom</ent><pos>adv.</pos><def>Formerly; once.</def></p></gcide>",
                )

            gcide_items = list(iter_gcide_xml_zip(gcide_zip))
            self.assertEqual(len(gcide_items), 1)
            self.assertEqual(gcide_items[0]["word"], "whilom")

            modern_acc = DictionaryAccumulator(profile="modern")
            modern_acc.ingest_gcide(gcide_items)
            self.assertNotIn("whilom", modern_acc.entries)

            full_acc = DictionaryAccumulator(profile="full")
            full_acc.ingest_gcide(gcide_items)
            self.assertIn("whilom", full_acc.entries)
            self.assertEqual(full_acc.entries["whilom"].senses[0].tags, ["archaic"])


if __name__ == "__main__":
    unittest.main()
