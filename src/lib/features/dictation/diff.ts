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

const CONTRACTIONS: Record<string, string[]> = {
  "don't": ["do", "not"],
  "doesn't": ["does", "not"],
  "didn't": ["did", "not"],
  "can't": ["cannot"],
  "won't": ["will", "not"],
  "wouldn't": ["would", "not"],
  "shouldn't": ["should", "not"],
  "couldn't": ["could", "not"],
  "isn't": ["is", "not"],
  "aren't": ["are", "not"],
  "wasn't": ["was", "not"],
  "weren't": ["were", "not"],
  "haven't": ["have", "not"],
  "hasn't": ["has", "not"],
  "hadn't": ["had", "not"],
  "i'm": ["i", "am"],
  "you're": ["you", "are"],
  "we're": ["we", "are"],
  "they're": ["they", "are"],
  "it's": ["it", "is"],
  "that's": ["that", "is"],
  "there's": ["there", "is"],
  "i've": ["i", "have"],
  "i'll": ["i", "will"],
  "i'd": ["i", "would"],
};

function normalizeWords(input: string): string[] {
  const raw = input
    .toLowerCase()
    .replace(/[.,!?;:"()[\]{}]/g, " ")
    .split(/\s+/)
    .filter(Boolean);

  const expanded: string[] = [];
  for (const w of raw) {
    const clean = w.replace(/^'+|'+$/g, "");
    if (!clean) continue;
    if (CONTRACTIONS[clean]) {
      expanded.push(...CONTRACTIONS[clean]);
    } else {
      expanded.push(clean);
    }
  }
  return expanded;
}

export function computeDictationDiff(expected: string, actual: string): DictationResult {
  const exp = normalizeWords(expected);
  const act = normalizeWords(actual);

  if (exp.length === 0) {
    return { tokens: [], accuracy: 100, isExact: true };
  }

  const n = exp.length;
  const m = act.length;
  const dp: number[][] = Array.from({ length: n + 1 }, () => Array(m + 1).fill(0));

  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      if (exp[i] === act[j]) {
        dp[i][j] = 1 + dp[i + 1][j + 1];
      } else {
        dp[i][j] = Math.max(dp[i + 1][j], dp[i][j + 1]);
      }
    }
  }

  const tokens: DiffToken[] = [];
  let i = 0;
  let j = 0;
  let correctCount = 0;

  while (i < n || j < m) {
    if (i < n && j < m && exp[i] === act[j]) {
      tokens.push({ text: act[j], status: "correct" });
      correctCount += 1;
      i += 1;
      j += 1;
    } else if (i < n && j < m && dp[i + 1][j] === dp[i][j + 1]) {
      tokens.push({ text: act[j], expected: exp[i], status: "wrong" });
      i += 1;
      j += 1;
    } else if (i < n && (j === m || dp[i + 1][j] >= dp[i][j + 1])) {
      tokens.push({ text: exp[i], status: "missing" });
      i += 1;
    } else if (j < m) {
      tokens.push({ text: act[j], status: "extra" });
      j += 1;
    }
  }

  const accuracy = Math.round((correctCount / exp.length) * 100);
  return {
    tokens,
    accuracy,
    isExact: correctCount === exp.length && act.length === exp.length,
  };
}
