import assert from "node:assert/strict";
import test from "node:test";
import {
  buildClozeQuestion,
  buildMatchRound,
  buildQuizQuestion,
  scrambleLetters,
} from "./gameEngine.ts";

function makeNote(id, word, meaning, ipa = null, exampleSentence = null) {
  return {
    id,
    deckId: "d1",
    noteType: "vocab",
    word,
    ipa,
    meaning,
    exampleSentence,
    audioPath: null,
    createdAt: "2026-01-01",
  };
}

test("buildMatchRound creates paired shuffled columns without duplicates", () => {
  const notes = [
    makeNote("1", "ubiquitous", "khắp nơi"),
    makeNote("2", "pragmatic", "thực tế"),
    makeNote("3", "resilient", "kiên cường"),
    makeNote("4", "eloquent", "hùng hồn"),
  ];
  const round = buildMatchRound(notes, 3);
  assert.equal(round.left.length, 3);
  assert.equal(round.right.length, 3);
  const leftIds = new Set(round.left.map((c) => c.id));
  const rightIds = new Set(round.right.map((c) => c.id));
  assert.deepEqual(leftIds, rightIds);
});

test("buildQuizQuestion builds 4 unique options including exact target", () => {
  const notes = [
    makeNote("1", "ubiquitous", "khắp nơi", "/juːˈbɪk.wɪ.təs/"),
    makeNote("2", "pragmatic", "thực tế"),
    makeNote("3", "resilient", "kiên cường"),
    makeNote("4", "eloquent", "hùng hồn"),
    makeNote("5", "ephemeral", "ngắn ngủi"),
  ];
  const q = buildQuizQuestion(notes[0], notes, "en-to-vi");
  assert.equal(q.prompt, "ubiquitous");
  assert.equal(q.options.length, 4);
  assert.equal(q.options.filter((o) => o.isCorrect).length, 1);
  assert.equal(new Set(q.options.map((o) => o.label)).size, 4);
});

test("buildClozeQuestion masks target word in example sentence", () => {
  const notes = [
    makeNote("1", "resilient", "kiên cường", null, "Children are remarkably resilient."),
    makeNote("2", "pragmatic", "thực tế"),
    makeNote("3", "eloquent", "hùng hồn"),
    makeNote("4", "ephemeral", "ngắn ngủi"),
  ];
  const cloze = buildClozeQuestion(notes[0], notes);
  assert.ok(cloze);
  assert.equal(cloze.maskedSentence, "Children are remarkably _____.");
  assert.equal(cloze.options.length, 4);
  assert.equal(cloze.options.find((o) => o.isCorrect)?.label, "resilient");
});

test("scrambleLetters returns scrambled permutation of same characters", () => {
  const letters = scrambleLetters("pragmatic");
  assert.equal(letters.length, "pragmatic".length);
  assert.notEqual(letters.join(""), "pragmatic");
  assert.deepEqual([...letters].sort(), [..."pragmatic"].sort());
});
