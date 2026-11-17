<script lang="ts">
  import { Mic, Play, Repeat, Square } from "@lucide/svelte";
  import { onDestroy } from "svelte";

  interface Props {
    onPlaySample: () => void;
  }

  let { onPlaySample }: Props = $props();

  let recording = $state(false);
  let recordedUrl = $state<string | null>(null);
  let micError = $state("");

  let mediaRecorder: MediaRecorder | null = null;
  let chunks: Blob[] = [];
  let compareTimer: ReturnType<typeof setTimeout> | null = null;
  let destroyed = false;

  function cleanupUrl() {
    if (recordedUrl) {
      URL.revokeObjectURL(recordedUrl);
      recordedUrl = null;
    }
  }

  async function toggleRecord() {
    if (recording) {
      mediaRecorder?.stop();
      recording = false;
      return;
    }

    micError = "";
    cleanupUrl();

    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      chunks = [];
      mediaRecorder = new MediaRecorder(stream);

      mediaRecorder.ondataavailable = (ev) => {
        if (ev.data.size > 0) chunks.push(ev.data);
      };

      mediaRecorder.onstop = () => {
        stream.getTracks().forEach((t) => t.stop());
        if (destroyed) return;
        const blob = new Blob(chunks, { type: "audio/webm" });
        recordedUrl = URL.createObjectURL(blob);
      };

      mediaRecorder.start();
      recording = true;
    } catch {
      micError = "Không thể truy cập micro. Hãy kiểm tra quyền ghi âm.";
    }
  }

  function playRecorded() {
    if (!recordedUrl) return;
    const audio = new Audio(recordedUrl);
    void audio.play();
  }

  function compareAB() {
    if (compareTimer) clearTimeout(compareTimer);
    onPlaySample();
    if (recordedUrl) {
      compareTimer = setTimeout(() => {
        compareTimer = null;
        if (!destroyed) playRecorded();
      }, 1400);
    }
  }

  onDestroy(() => {
    destroyed = true;
    if (compareTimer) clearTimeout(compareTimer);
    if (mediaRecorder && recording) {
      mediaRecorder.stop();
    }
    cleanupUrl();
  });
</script>

<div class="rounded-2xl border border-zinc-800/80 bg-zinc-900/50 px-4 py-3">
  <div class="flex flex-wrap items-center justify-between gap-2">
    <span class="text-xs font-medium text-zinc-400">Shadowing A/B</span>

    <div class="flex items-center gap-2">
      <button
        type="button"
        onclick={onPlaySample}
        class="inline-flex items-center gap-1 rounded-lg border border-zinc-700 px-2.5 py-1.5 text-xs text-zinc-300 hover:bg-zinc-800"
      >
        <Play class="h-3 w-3" />
        Mẫu (A)
      </button>

      <button
        type="button"
        onclick={toggleRecord}
        class="inline-flex items-center gap-1 rounded-lg px-2.5 py-1.5 text-xs font-medium transition {recording
          ? 'bg-rose-600 text-white animate-pulse'
          : 'border border-zinc-700 text-zinc-200 hover:bg-zinc-800'}"
      >
        {#if recording}
          <Square class="h-3 w-3" />
          Dừng
        {:else}
          <Mic class="h-3 w-3" />
          Ghi âm (B)
        {/if}
      </button>

      {#if recordedUrl}
        <button
          type="button"
          onclick={playRecorded}
          class="inline-flex items-center gap-1 rounded-lg border border-emerald-500/40 bg-emerald-500/10 px-2.5 py-1.5 text-xs text-emerald-300 hover:bg-emerald-500/20"
        >
          <Play class="h-3 w-3" />
          Giọng bạn (B)
        </button>

        <button
          type="button"
          onclick={compareAB}
          class="inline-flex items-center gap-1 rounded-lg bg-indigo-600/20 px-2.5 py-1.5 text-xs font-medium text-indigo-300 hover:bg-indigo-600/30"
        >
          <Repeat class="h-3 w-3" />
          So sánh A→B
        </button>
      {/if}
    </div>
  </div>

  {#if micError}
    <p class="mt-2 text-xs text-rose-400">{micError}</p>
  {/if}
</div>
