export type DiffStatus = "correct" | "wrong" | "missing" | "extra";

export interface DiffToken {
  text: string;
  expected?: string;
  status: DiffStatus;
}

export interface DictationResult {
  tokens: DiffToken[];
  accuracy: number;
  isExact: boolean;
}

function normalizeWords(input: string): string[] {
  return input
    .toLowerCase()
    .replace(/[.,!?;:"'()[\]{}]/g, " ")
    .split(/\s+/)
    .filter(Boolean);
}

export function computeDictationDiff(expected: string, actual: string): DictationResult {
  const expWords = normalizeWords(expected);
  const actWords = normalizeWords(actual);

  if (expWords.length === 0) {
    return { tokens: [], accuracy: 100, isExact: true };
  }

  const tokens: DiffToken[] = [];
  const maxLen = Math.max(expWords.length, actWords.length);
  let correctCount = 0;

  for (let i = 0; i < maxLen; i++) {
    const exp = expWords[i];
    const act = actWords[i];

    if (exp && act) {
      if (exp === act) {
        tokens.push({ text: act, status: "correct" });
        correctCount += 1;
      } else {
        tokens.push({ text: act, expected: exp, status: "wrong" });
      }
    } else if (exp && !act) {
      tokens.push({ text: exp, status: "missing" });
    } else if (!exp && act) {
      tokens.push({ text: act, status: "extra" });
    }
  }

  const accuracy = Math.round((correctCount / expWords.length) * 100);
  return {
    tokens,
    accuracy,
    isExact: correctCount === expWords.length && actWords.length === expWords.length,
  };
}
