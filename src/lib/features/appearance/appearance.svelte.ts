export type ThemeId =
  | "dark-indigo"
  | "midnight-ocean"
  | "forest-emerald"
  | "rose-twilight"
  | "amber-noir"
  | "nord-frost"
  | "warm-sepia"
  | "clean-light";

export interface AppearanceSettings {
  theme: ThemeId;
  fontFamily: string;
  ipaFontFamily: string;
  fontSize: number;
}

export const THEME_OPTIONS: Array<{
  id: ThemeId;
  name: string;
  bg: string;
  card: string;
  accent: string;
  isLight?: boolean;
}> = [
  { id: "dark-indigo", name: "Dark Indigo", bg: "#09090b", card: "#18181b", accent: "#6366f1" },
  { id: "midnight-ocean", name: "Midnight Ocean", bg: "#07111e", card: "#0f1d32", accent: "#38bdf8" },
  { id: "forest-emerald", name: "Forest Emerald", bg: "#07140e", card: "#0d2218", accent: "#34d399" },
  { id: "rose-twilight", name: "Rose Twilight", bg: "#160c18", card: "#241428", accent: "#f472b6" },
  { id: "amber-noir", name: "Amber Noir", bg: "#12100e", card: "#1d1915", accent: "#fbbf24" },
  { id: "nord-frost", name: "Nord Frost", bg: "#242933", card: "#2e3440", accent: "#88c0d0" },
  { id: "warm-sepia", name: "Warm Sepia", bg: "#f5efe6", card: "#ebe2d3", accent: "#b45309", isLight: true },
  { id: "clean-light", name: "Clean Light", bg: "#f8fafc", card: "#ffffff", accent: "#4f46e5", isLight: true },
];

export const GOOGLE_FONT_OPTIONS = [
  { value: '"Inter", "Noto Sans", sans-serif', label: "Inter (Hiện đại, rõ nét)" },
  { value: '"Noto Sans", sans-serif', label: "Noto Sans (Chuẩn đa ngôn ngữ & IPA)" },
  { value: '"Be Vietnam Pro", "Noto Sans", sans-serif', label: "Be Vietnam Pro (Tối ưu Tiếng Việt)" },
  { value: '"Plus Jakarta Sans", "Noto Sans", sans-serif', label: "Plus Jakarta Sans (Mềm mại)" },
  { value: '"Noto Serif", Georgia, serif', label: "Noto Serif (Kiểu sách báo)" },
];

export const IPA_FONT_OPTIONS = [
  { value: '"Noto Sans", "Inter", sans-serif', label: "Noto Sans (Chuẩn ký tự IPA)" },
  { value: '"Inter", "Noto Sans", sans-serif', label: "Inter" },
  { value: '"JetBrains Mono", "Noto Sans", monospace', label: "JetBrains Mono (Đơn cách)" },
];

const STORAGE_KEY = "vlearning_appearance";

const DEFAULT_SETTINGS: AppearanceSettings = {
  theme: "dark-indigo",
  fontFamily: GOOGLE_FONT_OPTIONS[0].value,
  ipaFontFamily: IPA_FONT_OPTIONS[0].value,
  fontSize: 16,
};

function loadInitial(): AppearanceSettings {
  if (typeof window === "undefined") return { ...DEFAULT_SETTINGS };
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULT_SETTINGS };
    return { ...DEFAULT_SETTINGS, ...JSON.parse(raw) };
  } catch {
    return { ...DEFAULT_SETTINGS };
  }
}

export const appearance = $state<AppearanceSettings>(loadInitial());

export function applyAppearance(s: AppearanceSettings): void {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  root.setAttribute("data-theme", s.theme);
  root.style.setProperty("--app-font-sans", s.fontFamily);
  root.style.setProperty("--app-font-ipa", s.ipaFontFamily);
  root.style.setProperty("--app-font-size", `${s.fontSize}px`);

  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(s));
  } catch {
    // ignore storage quota errors
  }
}

export function resetAppearance(): void {
  Object.assign(appearance, DEFAULT_SETTINGS);
  applyAppearance(appearance);
}
