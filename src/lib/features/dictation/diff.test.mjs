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

test("computeDictationDiff highlights wrong, missing, and extra words", () => {
  const res = computeDictationDiff("we need a pragmatic approach", "we need an approach");
  assert.equal(res.isExact, false);
  assert.equal(res.tokens[2].status, "wrong");
  assert.equal(res.tokens[2].expected, "a");
  assert.equal(res.tokens[4].status, "missing");
  assert.equal(res.tokens[4].text, "approach");
});
