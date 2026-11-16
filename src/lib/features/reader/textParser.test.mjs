import assert from "node:assert/strict";
import test from "node:test";
import { stripSubtitleMetadata, tokenizeWithSentences } from "./textParser.ts";

test("tokenizeWithSentences extracts words and maps each word to its sentence", () => {
  const sample = "Learning English is pragmatic. Consistency makes you resilient!";
  const tokens = tokenizeWithSentences(sample);

  const pragmatic = tokens.find((t) => t.text === "pragmatic");
  assert.ok(pragmatic);
  assert.equal(pragmatic.isWord, true);
  assert.equal(pragmatic.sentence, "Learning English is pragmatic.");

  const resilient = tokens.find((t) => t.text === "resilient");
  assert.ok(resilient);
  assert.equal(resilient.sentence, "Consistency makes you resilient!");
});

test("stripSubtitleMetadata cleans srt and vtt timestamps and tags", () => {
  const srt = `1
00:00:01,000 --> 00:00:03,500
<i>Hello world.</i>

2
00:00:04,000 --> 00:00:06,000
This is pragmatic.`;
  assert.equal(stripSubtitleMetadata(srt), "Hello world. This is pragmatic.");
});
