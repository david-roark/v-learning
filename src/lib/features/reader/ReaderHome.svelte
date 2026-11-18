<script lang="ts">
  import {
    BookOpenText,
    Clock,
    FileText,
    Globe,
    Loader2,
    Play,
    Trash2,
    Upload,
  } from "@lucide/svelte";
  import type { ReaderHistoryItem } from "./readerHistory";

  interface Props {
    webUrl: string;
    parsingFile: boolean;
    loadingTarget: string;
    openingHistoryId: string | null;
    history: ReaderHistoryItem[];
    onLoadUrl: (e: SubmitEvent) => void;
    onFileUpload: (e: Event) => void;
    onStartPasteText: () => void;
    onOpenHistoryItem: (item: ReaderHistoryItem) => void;
    onDeleteHistoryItem: (id: string) => void;
    onClearHistory: () => void;
  }

  let {
    webUrl = $bindable(),
    parsingFile,
    loadingTarget,
    openingHistoryId,
    history,
    onLoadUrl,
    onFileUpload,
    onStartPasteText,
    onOpenHistoryItem,
    onDeleteHistoryItem,
    onClearHistory,
  }: Props = $props();

  function formatRelativeTime(ts: number): string {
    const mins = Math.floor((Date.now() - ts) / 60000);
    if (mins < 1) return "vừa xong";
    if (mins < 60) return `${mins} phút trước`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours} giờ trước`;
    return `${Math.floor(hours / 24)} ngày trước`;
  }
</script>

<div class="w-full space-y-5">
  <!-- Hero Input Bar -->
  <div class="space-y-3 rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div class="flex items-center gap-3">
        <div class="rounded-xl bg-indigo-500/15 p-2.5 text-indigo-400">
          <BookOpenText class="h-5 w-5" />
        </div>
        <div>
          <h2 class="text-sm font-semibold text-white">Đọc báo, Sách & Video</h2>
          <p class="text-xs text-zinc-400">
            Click 1 từ hoặc bôi đen cả cụm từ / câu để tra nghĩa & lưu flashcard ngay.
          </p>
        </div>
      </div>

      <div class="flex flex-wrap items-center gap-2">
        <label
          class="inline-flex cursor-pointer items-center gap-1.5 rounded-xl border border-zinc-700 bg-zinc-950/60 px-3.5 py-2 text-xs font-medium text-zinc-200 hover:border-indigo-500 hover:bg-zinc-800"
        >
          <Upload class="h-3.5 w-3.5 text-indigo-400" />
          {parsingFile ? "Đang mở..." : "Mở file (.pdf, .epub, .txt, .srt)"}
          <input
            type="file"
            accept=".epub,.pdf,.txt,.md,.srt,.vtt"
            onchange={onFileUpload}
            class="hidden"
          />
        </label>
        <button
          type="button"
          onclick={onStartPasteText}
          class="rounded-xl border border-zinc-700 bg-zinc-950/60 px-3.5 py-2 text-xs font-medium text-zinc-200 hover:border-indigo-500 hover:bg-zinc-800"
        >
          Dán văn bản
        </button>
      </div>
    </div>

    <form onsubmit={onLoadUrl} class="flex gap-2">
      <div class="relative flex-1">
        <Globe class="absolute top-1/2 left-3.5 h-4 w-4 -translate-y-1/2 text-zinc-500" />
        <input
          type="url"
          bind:value={webUrl}
          placeholder="Dán link bài báo hoặc video YouTube (https://...)..."
          class="w-full rounded-xl border border-zinc-700 bg-zinc-950 py-2.5 pr-4 pl-10 text-xs text-white placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
        />
      </div>
      <button
        type="submit"
        disabled={parsingFile || !webUrl.trim()}
        class="inline-flex items-center gap-1.5 rounded-xl bg-indigo-600 px-5 py-2.5 text-xs font-semibold text-white hover:bg-indigo-500 disabled:opacity-40"
      >
        {#if parsingFile}
          <Loader2 class="h-3.5 w-3.5 animate-spin" />
          Đang tải...
        {:else}
          Mở URL
        {/if}
      </button>
    </form>

    {#if parsingFile}
      <div
        class="flex items-center gap-2.5 rounded-xl border border-indigo-500/30 bg-indigo-500/10 px-3.5 py-2.5 text-xs text-indigo-200"
      >
        <Loader2 class="h-4 w-4 shrink-0 animate-spin text-indigo-400" />
        <span class="truncate">Đang tải nội dung: <strong>{loadingTarget}</strong>...</span>
      </div>
    {/if}
  </div>

  <!-- Recent History Grid -->
  <div class="space-y-3">
    <div class="flex items-center justify-between px-1">
      <div class="flex items-center gap-2 text-xs font-semibold text-zinc-300">
        <Clock class="h-4 w-4 text-indigo-400" />
        <span>Gần đây ({history.length})</span>
      </div>
      {#if history.length > 0}
        <button
          type="button"
          onclick={onClearHistory}
          class="text-xs text-zinc-400 hover:text-rose-400"
        >
          Xóa tất cả
        </button>
      {/if}
    </div>

    {#if history.length === 0}
      <div
        class="rounded-2xl border border-dashed border-zinc-800 bg-zinc-900/30 py-14 text-center text-xs text-zinc-500"
      >
        Chưa có lịch sử đọc gần đây. Hãy dán một đường link YouTube/bài báo hoặc mở file sách ở
        trên.
      </div>
    {:else}
      <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {#each history as item (item.id)}
          {@const isOpening = openingHistoryId === item.id}
          <div
            class="group relative flex flex-col justify-between rounded-2xl border p-4 transition {isOpening
              ? 'border-indigo-500 bg-indigo-500/15 shadow-lg shadow-indigo-500/10'
              : parsingFile
                ? 'pointer-events-none border-zinc-800/70 bg-zinc-900/40 opacity-50'
                : 'border-zinc-800/90 bg-zinc-900/60 hover:border-indigo-500/50 hover:bg-zinc-900'}"
          >
            <button
              type="button"
              disabled={parsingFile}
              onclick={() => onOpenHistoryItem(item)}
              class="flex flex-col items-start text-left"
            >
              <div class="mb-2.5 flex w-full items-center justify-between gap-2">
                <span
                  class="inline-flex items-center gap-1.5 rounded-lg px-2 py-1 text-[10px] font-medium {item.kind ===
                  'youtube'
                    ? 'bg-rose-500/15 text-rose-300'
                    : item.kind === 'article'
                      ? 'bg-sky-500/15 text-sky-300'
                      : item.kind === 'pdf'
                        ? 'bg-amber-500/15 text-amber-300'
                        : 'bg-emerald-500/15 text-emerald-300'}"
                >
                  {#if isOpening}
                    <Loader2 class="h-3 w-3 animate-spin" />
                    Đang mở...
                  {:else if item.kind === "youtube"}
                    <Play class="h-3 w-3" />
                    YouTube
                  {:else if item.kind === "article"}
                    <Globe class="h-3 w-3" />
                    Web
                  {:else}
                    <FileText class="h-3 w-3" />
                    {item.kind.toUpperCase()}
                  {/if}
                </span>
                <span class="text-[11px] text-zinc-500">{formatRelativeTime(item.openedAt)}</span>
              </div>

              <h3 class="line-clamp-2 pr-5 text-sm font-semibold text-zinc-100">
                {item.title}
              </h3>
              <p class="mt-1 line-clamp-1 w-full text-[11px] text-zinc-400">{item.subtitle}</p>
            </button>

            <div class="mt-3 flex items-center justify-between border-t border-zinc-800/70 pt-2">
              <span class="text-[11px] text-zinc-400">
                {#if item.lastPos !== undefined}
                  Đang đọc: {item.kind === "pdf"
                    ? `Trang ${item.lastPos}`
                    : `Chương ${item.lastPos + 1}`}
                {:else}
                  Nhấn để mở lại
                {/if}
              </span>
              <button
                type="button"
                onclick={() => onDeleteHistoryItem(item.id)}
                title="Xóa khỏi lịch sử"
                class="rounded-lg p-1 text-zinc-500 opacity-70 transition hover:bg-zinc-800 hover:text-rose-400 group-hover:opacity-100"
              >
                <Trash2 class="h-3.5 w-3.5" />
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
