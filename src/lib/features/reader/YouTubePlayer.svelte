<script lang="ts">
  import { tokenizeWithSentences, type TextToken } from "./textParser";

  export interface TranscriptLine {
    startSec: number;
    text: string;
  }

  interface Props {
    videoId: string;
    transcript: TranscriptLine[];
    selectedWord?: string;
    onWordClick: (tok: TextToken) => void;
  }

  let { videoId, transcript, selectedWord = "", onWordClick }: Props = $props();

  let seekSec = $state(0);
  let autoplay = $state(0);

  const embedSrc = $derived(
    `https://www.youtube-nocookie.com/embed/${videoId}?start=${seekSec}&autoplay=${autoplay}`,
  );

  function formatTime(sec: number): string {
    const m = Math.floor(sec / 60);
    const s = Math.floor(sec % 60);
    return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  }

  function jumpTo(sec: number) {
    seekSec = Math.floor(sec);
    autoplay = 1;
  }
</script>

<div class="flex h-[calc(100vh-13.5rem)] flex-col gap-3">
  <div
    class="aspect-video max-h-[45%] w-full overflow-hidden rounded-2xl border border-zinc-800 bg-black"
  >
    <iframe
      src={embedSrc}
      title="YouTube video player"
      allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
      allowfullscreen
      class="h-full w-full"
    ></iframe>
  </div>

  <div class="flex-1 overflow-y-auto rounded-2xl border border-zinc-800 bg-zinc-900/70 p-4">
    {#if transcript.length === 0}
      <p class="py-8 text-center text-xs text-zinc-400">
        Video này không có phụ đề tiếng Anh tự động (hoặc YouTube đang chặn tải transcript). Bạn có
        thể mở file <code class="rounded bg-zinc-800 px-1">.srt/.vtt</code> để khai thác câu.
      </p>
    {:else}
      <div class="space-y-2">
        {#each transcript as line}
          {@const tokens = tokenizeWithSentences(line.text)}
          <div class="flex items-start gap-2.5 rounded-xl px-2 py-1.5 hover:bg-zinc-800/40">
            <button
              type="button"
              onclick={() => jumpTo(line.startSec)}
              title="Tua tới đoạn này"
              class="mt-0.5 shrink-0 rounded-md bg-zinc-800 px-1.5 py-0.5 font-mono text-[11px] text-indigo-300 hover:bg-indigo-600 hover:text-white"
            >
              {formatTime(line.startSec)}
            </button>
            <div class="text-sm leading-relaxed text-zinc-200">
              {#each tokens as tok}
                {#if tok.isWord}
                  <button
                    type="button"
                    onclick={() => onWordClick(tok)}
                    class="rounded px-0.5 transition hover:bg-indigo-500/25 hover:text-indigo-200 {selectedWord ===
                    tok.text
                      ? 'bg-indigo-600/30 text-indigo-200'
                      : ''}"
                  >
                    {tok.text}
                  </button>
                {:else}
                  <span>{tok.text}</span>
                {/if}
              {/each}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
