<script lang="ts">
  import { ExternalLink, Info, Scale, X } from "@lucide/svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { onMount } from "svelte";

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  const REPO_URL = "https://github.com/david-roark/v-learning";
  const LICENSE_URL = "https://github.com/david-roark/v-learning/blob/main/LICENSE";

  let appVersion = $state("0.2.0");

  async function handleOpenExternal(url: string) {
    try {
      await openUrl(url);
    } catch {
      window.open(url, "_blank", "noopener,noreferrer");
    }
  }

  onMount(async () => {
    try {
      appVersion = await getVersion();
    } catch {
      // Fallback to static version outside Tauri runtime
    }
  });
</script>

<svelte:window
  onkeydown={(e) => {
    if (open && e.key === "Escape") {
      onClose();
    }
  }}
/>

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-xs"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) onClose();
    }}
  >
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="about-modal-title"
      class="w-full max-w-lg rounded-2xl border border-zinc-800 bg-zinc-900 p-6 shadow-2xl"
    >
      <div class="mb-4 flex items-start justify-between gap-3">
        <div class="flex items-center gap-3">
          <img
            src="/logo.png"
            alt="V-Learning"
            class="h-12 w-12 rounded-2xl shadow-md shadow-indigo-600/20"
          />
          <div>
            <div class="flex items-center gap-2">
              <h2 id="about-modal-title" class="text-lg font-bold text-white">V-Learning</h2>
              <span class="rounded-full bg-indigo-500/15 px-2 py-0.5 font-mono text-[11px] font-medium text-indigo-300">
                v{appVersion}
              </span>
            </div>
            <p class="text-xs text-zinc-400">Học tiếng Anh & Từ vựng Offline-first</p>
          </div>
        </div>
        <button
          type="button"
          onclick={onClose}
          title="Đóng"
          class="rounded-lg p-1.5 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200"
        >
          <X class="h-4 w-4" />
        </button>
      </div>

      <div class="space-y-3.5 text-xs">
        <p class="leading-relaxed text-zinc-300">
          Ứng dụng học từ vựng và đọc hiểu tiếng Anh đa nền tảng với thuật toán lặp lại ngắt quãng
          <strong class="text-white">FSRS</strong>, từ điển offline FTS5, đọc báo/EPUB/PDF/YouTube,
          Dictation, Shadowing, AI Practice và Mini-game.
        </p>

        <div class="rounded-xl border border-zinc-800 bg-zinc-950/60 p-3.5 space-y-2">
          <div class="flex items-center gap-2 font-semibold text-zinc-200">
            <Info class="h-3.5 w-3.5 text-indigo-400" />
            Công nghệ
          </div>
          <p class="text-zinc-400">
            Tauri v2 · Rust · SQLite (FTS5) · Svelte 5 · SvelteKit · Tailwind CSS v4 · pdf.js
          </p>
        </div>

        <div class="rounded-xl border border-zinc-800 bg-zinc-950/60 p-3.5 space-y-2">
          <div class="flex items-center justify-between gap-2">
            <span class="flex items-center gap-2 font-semibold text-zinc-200">
              <Scale class="h-3.5 w-3.5 text-emerald-400" />
              Giấy phép (License)
            </span>
            <button
              type="button"
              onclick={() => handleOpenExternal(LICENSE_URL)}
              class="inline-flex items-center gap-1 text-[11px] text-indigo-400 hover:underline"
            >
              Xem LICENSE
              <ExternalLink class="h-3 w-3" />
            </button>
          </div>
          <p class="leading-relaxed text-zinc-400">
            <strong class="text-zinc-200">V-Learning Personal & Non-Commercial License</strong> —
            Miễn phí cho mục đích cá nhân, học tập và phi thương mại. Dữ liệu từ điển offline tuân
            thủ giấy phép nguồn mở tương ứng (Skypediacode & Kaikki.org CC BY-SA 4.0, Open English
            WordNet CC BY 4.0, CMUdict, SCOWL, GCIDE).
          </p>
        </div>

        <div class="flex flex-wrap items-center justify-between gap-2 pt-1">
          <button
            type="button"
            onclick={() => handleOpenExternal(REPO_URL)}
            class="inline-flex items-center gap-1.5 rounded-xl border border-zinc-700 bg-zinc-800/70 px-3.5 py-2 text-xs font-medium text-zinc-200 transition hover:border-indigo-500 hover:text-white"
          >
            <ExternalLink class="h-3.5 w-3.5" />
            GitHub: david-roark/v-learning
          </button>

          <button
            type="button"
            onclick={onClose}
            class="rounded-xl bg-indigo-600 px-4 py-2 text-xs font-semibold text-white hover:bg-indigo-500"
          >
            Đóng
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
