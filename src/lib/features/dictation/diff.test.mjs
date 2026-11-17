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

test("computeDictationDiff handles empty input and extra/wrong words", () => {
  const emptyInput = computeDictationDiff("hello world", "   ");
  assert.equal(emptyInput.isExact, false);
  assert.equal(emptyInput.accuracy, 0);
  assert.equal(emptyInput.tokens.length, 2);
  assert.ok(emptyInput.tokens.every((t) => t.status === "missing"));

  const wrongAndExtra = computeDictationDiff("she is resilient", "she are very resilient");
  assert.equal(wrongAndExtra.isExact, false);
  assert.ok(wrongAndExtra.tokens.some((t) => t.status === "wrong" && t.expected === "is"));
  assert.ok(wrongAndExtra.tokens.some((t) => t.status === "extra" && t.text === "very"));

  const contractions = computeDictationDiff("it is what I am not", "it's what I'm not");
  assert.equal(contractions.isExact, true);
  assert.equal(contractions.accuracy, 100);
});
