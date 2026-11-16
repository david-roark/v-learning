export interface TextToken {
  text: string;
  isWord: boolean;
  sentence: string;
}

export function tokenizeWithSentences(rawText: string): TextToken[] {
  const tokens: TextToken[] = [];
  const sentences = rawText.split(/(?<=[.!?])\s+|\n+/);

  for (let sIdx = 0; sIdx < sentences.length; sIdx++) {
    const sentence = sentences[sIdx].trim();
    if (!sentence) continue;

    const parts = sentence.split(/([A-Za-z][A-Za-z'-]*)/g);
    for (const part of parts) {
      if (!part) continue;
      const isWord = /^[A-Za-z][A-Za-z'-]*$/.test(part);
      tokens.push({ text: part, isWord, sentence });
    }

    if (sIdx < sentences.length - 1) {
      tokens.push({ text: " ", isWord: false, sentence });
    }
  }

  return tokens;
}
