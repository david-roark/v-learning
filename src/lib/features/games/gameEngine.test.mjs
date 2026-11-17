import assert from "node:assert/strict";
import test from "node:test";
import { buildMatchRound, buildQuizQuestion } from "./gameEngine.ts";

function makeNote(id, word, meaning, ipa = null) {
  return {
    id,
    deckId: "d1",
    noteType: "vocab",
    word,
    ipa,
    meaning,
    exampleSentence: null,
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
