import assert from "node:assert/strict";
import test from "node:test";
import { computeDictationDiff } from "./diff.ts";

test("computeDictationDiff handles exact match ignoring punctuation and case", () => {
  const res = computeDictationDiff(
    "Smartphones have become ubiquitous!",
    "smartphones have become ubiquitous",
  );
  assert.equal(res.isExact, true);
  assert.equal(res.accuracy, 100);
  assert.ok(res.tokens.every((t) => t.status === "correct"));
});

test("computeDictationDiff aligns missing word without shifting remaining words", () => {
  const res = computeDictationDiff("we need a pragmatic approach", "we a pragmatic approach");
  assert.equal(res.isExact, false);
  assert.equal(res.accuracy, 80);
  assert.equal(res.tokens[0].status, "correct");
  assert.equal(res.tokens[1].status, "missing");
  assert.equal(res.tokens[1].text, "need");
  assert.equal(res.tokens[2].status, "correct");
  assert.equal(res.tokens[3].status, "correct");
  assert.equal(res.tokens[4].status, "correct");
});

test("computeDictationDiff treats contractions as equivalent", () => {
  const res = computeDictationDiff("I do not know", "I don't know");
  assert.equal(res.isExact, true);
  assert.equal(res.accuracy, 100);
});
