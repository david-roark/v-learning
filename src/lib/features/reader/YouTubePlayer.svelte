<script lang="ts">
  import { onDestroy, onMount } from "svelte";
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

  let iframeEl = $state<HTMLIFrameElement | null>(null);
  let transcriptScrollEl = $state<HTMLDivElement | null>(null);
  let currentTime = $state(0);
  let autoFollow = $state(true);
  let rowEls: Array<HTMLDivElement | null> = [];

  const embedSrc = $derived(
    `https://www.youtube-nocookie.com/embed/${videoId}?enablejsapi=1&rel=0`,
  );

  const activeIndex = $derived.by(() => {
    if (transcript.length === 0) return -1;
    let found = -1;
    for (let i = 0; i < transcript.length; i++) {
      if (transcript[i].startSec <= currentTime + 0.15) {
        found = i;
      } else {
        break;
      }
    }
    return found;
  });

  $effect(() => {
    if (!autoFollow || activeIndex < 0) return;
    const row = rowEls[activeIndex];
    const container = transcriptScrollEl;
    if (!row || !container) return;
    const targetTop = row.offsetTop - container.clientHeight / 2 + row.clientHeight / 2;
    container.scrollTo({ top: Math.max(0, targetTop), behavior: "smooth" });
  });

  function formatTime(sec: number): string {
    const m = Math.floor(sec / 60);
    const s = Math.floor(sec % 60);
    return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  }

  function sendCommand(func: string, args: unknown[] = []) {
    iframeEl?.contentWindow?.postMessage(JSON.stringify({ event: "command", func, args }), "*");
  }

  function subscribePlayerEvents() {
    iframeEl?.contentWindow?.postMessage(JSON.stringify({ event: "listening", id: 1 }), "*");
    sendCommand("addEventListener", ["onStateChange"]);
  }

  function jumpTo(sec: number) {
    currentTime = sec;
    sendCommand("seekTo", [sec, true]);
    sendCommand("playVideo");
  }

  function handleWindowMessage(ev: MessageEvent) {
    if (typeof ev.data !== "string") return;
    if (!ev.origin.includes("youtube.com") && !ev.origin.includes("youtube-nocookie.com")) return;
    try {
      const msg = JSON.parse(ev.data) as {
        event?: string;
        info?: { currentTime?: number };
      };
      if (msg.event === "infoDelivery" && typeof msg.info?.currentTime === "number") {
        currentTime = msg.info.currentTime;
      }
    } catch {
      // Ignore non-JSON player messages
    }
  }

  let pollTimer: ReturnType<typeof setInterval> | null = null;

  onMount(() => {
    window.addEventListener("message", handleWindowMessage);
    // Re-send listening handshake briefly after load in case YouTube widget initializes asynchronously
    pollTimer = setInterval(() => {
      if (currentTime === 0) {
        subscribePlayerEvents();
      }
    }, 1500);
  });

  onDestroy(() => {
    window.removeEventListener("message", handleWindowMessage);
    if (pollTimer) clearInterval(pollTimer);
  });
</script>

<div class="flex h-[calc(100vh-13.5rem)] flex-col gap-3">
  <div
    class="aspect-video max-h-[45%] w-full overflow-hidden rounded-2xl border border-zinc-800 bg-black"
  >
    <iframe
      bind:this={iframeEl}
      src={embedSrc}
      onload={subscribePlayerEvents}
      title="YouTube video player"
      allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
      allowfullscreen
      class="h-full w-full"
    ></iframe>
  </div>

  <div class="flex min-h-0 flex-1 flex-col rounded-2xl border border-zinc-800 bg-zinc-900/70">
    {#if transcript.length > 0}
      <div
        class="flex items-center justify-between border-b border-zinc-800/80 px-4 py-2 text-[11px] text-zinc-400"
      >
        <span>
          Phụ đề ({transcript.length} dòng) ·
          <span class="font-mono text-indigo-300">{formatTime(currentTime)}</span>
        </span>
        <label class="inline-flex cursor-pointer items-center gap-1.5 select-none">
          <input
            type="checkbox"
            bind:checked={autoFollow}
            class="rounded border-zinc-600 bg-zinc-900 text-indigo-500"
          />
          Tự cuộn theo video
        </label>
      </div>
    {/if}

    <div bind:this={transcriptScrollEl} class="relative flex-1 overflow-y-auto p-4">
      {#if transcript.length === 0}
        <p class="py-8 text-center text-xs text-zinc-400">
          Video này không có phụ đề tiếng Anh tự động (hoặc YouTube đang chặn tải transcript). Bạn
          có thể mở file <code class="rounded bg-zinc-800 px-1">.srt/.vtt</code> để khai thác câu.
        </p>
      {:else}
        <div class="space-y-1.5">
          {#each transcript as line, idx (idx)}
            {@const tokens = tokenizeWithSentences(line.text)}
            {@const isActive = idx === activeIndex}
            <div
              bind:this={rowEls[idx]}
              class="flex items-start gap-2.5 rounded-xl border-l-2 px-2.5 py-1.5 transition {isActive
                ? 'border-indigo-400 bg-indigo-500/15'
                : 'border-transparent hover:bg-zinc-800/40'}"
            >
              <button
                type="button"
                onclick={() => jumpTo(line.startSec)}
                title="Tua tới đoạn này"
                class="mt-0.5 shrink-0 rounded-md px-1.5 py-0.5 font-mono text-[11px] transition {isActive
                  ? 'bg-indigo-600 text-white'
                  : 'bg-zinc-800 text-indigo-300 hover:bg-indigo-600 hover:text-white'}"
              >
                {formatTime(line.startSec)}
              </button>
              <div
                class="text-sm leading-relaxed {isActive
                  ? 'font-medium text-white'
                  : 'text-zinc-300'}"
              >
                {#each tokens as tok, tIdx (tIdx)}
                  {#if tok.isWord}
                    <button
                      type="button"
                      onclick={() => onWordClick(tok)}
                      class="select-text rounded px-0.5 transition hover:bg-indigo-500/25 hover:text-indigo-200 {selectedWord ===
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
</div>
