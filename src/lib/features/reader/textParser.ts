export interface TextToken {
  text: string;
  isWord: boolean;
  sentence: string;
}

export function joinPdfLines(lines: string[]): string {
  let result = "";
  for (const raw of lines) {
    const line = raw.trim();
    if (!line) continue;
    if (result.endsWith("-") && /^[a-z]/.test(line)) {
      result = result.slice(0, -1) + line;
    } else if (result) {
      result += " " + line;
    } else {
      result = line;
    }
  }
  return result.replace(/\s+/g, " ").trim();
}

export function stripSubtitleMetadata(raw: string): string {
  return raw
    .replace(/\r\n/g, "\n")
    .split("\n")
    .filter((line) => {
      const t = line.trim();
      if (!t || t === "WEBVTT") return false;
      if (/^\d+$/.test(t)) return false;
      if (/^\d{2}:\d{2}[:.]\d{2}/.test(t) || t.includes("-->")) return false;
      return true;
    })
    .map((line) => line.replace(/<[^>]+>/g, "").trim())
    .filter(Boolean)
    .join(" ");
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
