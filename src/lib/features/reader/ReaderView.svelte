<script lang="ts">
  import {
    ArrowLeft,
    ChevronLeft,
    ChevronRight,
    Globe,
    Loader2,
    Maximize2,
    MoveHorizontal,
    Upload,
    ZoomIn,
    ZoomOut,
  } from "@lucide/svelte";
  import { onDestroy, untrack } from "svelte";
  import { notesApi } from "$lib/features/card-editor/api";
  import { decksApi } from "$lib/features/decks/api";
  import { dictionaryApi } from "$lib/features/dictionary/api";
  import Select from "$lib/shared/components/Select.svelte";
  import { invokeCmd } from "$lib/shared/tauri";
  import type { DeckWithStats, DictionaryEntry } from "$lib/shared/types";
  import { openPdfSession, type BookSection, type PdfFitMode, type PdfSession } from "./pdfParser";
  import ReaderHome from "./ReaderHome.svelte";
  import {
    clearAllHistory,
    loadCachedReaderFile,
    loadReaderHistory,
    recordHistoryEntry,
    removeHistoryEntry,
    updateHistoryPosition,
    type ReaderHistoryItem,
  } from "./readerHistory";
  import ReaderLookupSidebar from "./ReaderLookupSidebar.svelte";
  import { stripSubtitleMetadata, tokenizeWithSentences, type TextToken } from "./textParser";
  import YouTubePlayer, { type TranscriptLine } from "./YouTubePlayer.svelte";

  interface Props {
    decks: DeckWithStats[];
    onCardAdded: () => void;
  }

  interface EpubBook {
    title: string;
    chapters: BookSection[];
  }

  interface ArticleBlock {
    tag: "h1" | "h2" | "h3" | "p" | "blockquote" | "li";
    text: string;
  }

  interface WebReaderResult {
    kind: "article" | "youtube";
    title: string;
    articleText: string | null;
    articleBlocks: ArticleBlock[];
    youtubeId: string | null;
    transcript: TranscriptLine[];
  }

  let { decks, onCardAdded }: Props = $props();

  let articleText = $state("");
  let bookTitle = $state("");
  let sections = $state<BookSection[]>([]);
  let sectionIndex = $state(0);
  let pdfSession = $state<PdfSession | null>(null);
  let pdfPageNum = $state(1);
  let pdfFitMode = $state<PdfFitMode>("fit-width");
  let pdfZoom = $state(1.25);
  let pdfCanvas = $state<HTMLCanvasElement | null>(null);
  let pdfTextLayer = $state<HTMLDivElement | null>(null);
  let pdfContainer = $state<HTMLDivElement | null>(null);
  let textContainer = $state<HTMLDivElement | null>(null);
  let youtubeContainer = $state<HTMLDivElement | null>(null);
  let readerRoot = $state<HTMLDivElement | null>(null);
  let editing = $state(false);
  let parsingFile = $state(false);
  let loadingTarget = $state("");
  let openingHistoryId = $state<string | null>(null);
  let showUrlBar = $state(false);
  let history = $state<ReaderHistoryItem[]>(loadReaderHistory());
  let activeHistoryId = $state<string | null>(null);
  let webUrl = $state("");
  let youtubeId = $state<string | null>(null);
  let youtubeTranscript = $state<TranscriptLine[]>([]);
  let articleBlocks = $state<ArticleBlock[]>([]);

  let selectedDeckId = $state("");
  let creatingDeck = $state(false);
  let newDeckName = $state("");
  let manualQuery = $state("");
  let customMeaning = $state("");
  let contextSentence = $state("");
  let selectedToken = $state<TextToken | null>(null);
  let lookupResult = $state<DictionaryEntry | null>(null);
  let lookingUp = $state(false);
  let statusMsg = $state("");
  let error = $state("");

  $effect(() => {
    if (!selectedDeckId && decks.length > 0) {
      selectedDeckId = decks[0].id;
    }
  });

  const displayText = $derived(
    sections.length > 0 ? (sections[sectionIndex]?.text ?? "") : articleText,
  );
  const tokens = $derived(tokenizeWithSentences(displayText));

  const totalItems = $derived(pdfSession ? pdfSession.numPages : sections.length);
  const currentPos = $derived(pdfSession ? pdfPageNum - 1 : sectionIndex);
  const hasActiveContent = $derived(
    editing ||
      Boolean(
        youtubeId ||
        pdfSession ||
        sections.length > 0 ||
        articleBlocks.length > 0 ||
        articleText.trim(),
      ),
  );

  async function closeActiveContent() {
    await clearPdfSession();
    bookTitle = "";
    sections = [];
    sectionIndex = 0;
    articleText = "";
    articleBlocks = [];
    youtubeId = null;
    youtubeTranscript = [];
    editing = false;
    showUrlBar = false;
    activeHistoryId = null;
    selectedToken = null;
    lookupResult = null;
    error = "";
    statusMsg = "";
  }

  function loadPdfPage(pageNum: number) {
    if (!pdfSession || pageNum < 1 || pageNum > pdfSession.numPages) return;
    pdfPageNum = pageNum;
    selectedToken = null;
    lookupResult = null;
    if (pdfContainer) pdfContainer.scrollTop = 0;
    history = updateHistoryPosition(activeHistoryId, pageNum);
  }

  $effect(() => {
    if (pdfSession && !editing && pdfCanvas && pdfContainer) {
      const scale = pdfFitMode === "custom" ? pdfZoom : untrack(() => pdfZoom);
      const opts = {
        scale,
        fitMode: pdfFitMode,
        containerWidth: pdfContainer.clientWidth,
        containerHeight: pdfContainer.clientHeight,
      };
      void pdfSession.renderPage(pdfPageNum, pdfCanvas, pdfTextLayer, opts).then((actualScale) => {
        if (pdfFitMode !== "custom") {
          pdfZoom = Math.round(actualScale * 100) / 100;
        }
      });
    }
  });

  function handleReaderTextSelect(e: MouseEvent | TouchEvent) {
    if (editing) return;
    const target = e.target as Node | null;
    if (!target) return;
    const inPdf = Boolean(pdfSession && pdfTextLayer?.contains(target));
    const inText = Boolean(!pdfSession && !youtubeId && textContainer?.contains(target));
    const inYt = Boolean(youtubeId && youtubeContainer?.contains(target));
    if (!inPdf && !inText && !inYt) return;

    const rawSel = window.getSelection()?.toString() ?? "";
    const cleaned = rawSel
      .replace(/-\s*\n\s*/g, "")
      .replace(/\s+/g, " ")
      .trim();
    if (!cleaned || cleaned.length > 500) return;

    // In text/YouTube mode, single clicks are handled by word buttons; only intercept multi-word drag selections
    if (!inPdf && !cleaned.includes(" ")) return;

    selectedToken = null;
    manualQuery = cleaned;
    void lookupAndSelect(cleaned, cleaned.includes(" ") ? cleaned : undefined);
  }

  function adjustZoom(delta: number) {
    pdfFitMode = "custom";
    pdfZoom = Math.max(0.4, Math.min(3.0, Math.round((pdfZoom + delta) * 100) / 100));
  }

  function goToPosition(idx: number) {
    if (pdfSession) {
      loadPdfPage(idx + 1);
      return;
    }
    if (idx < 0 || idx >= sections.length) return;
    sectionIndex = idx;
    selectedToken = null;
    lookupResult = null;
    if (textContainer) textContainer.scrollTop = 0;
    history = updateHistoryPosition(activeHistoryId, idx);
  }

  function handleReaderKeydown(e: KeyboardEvent) {
    if (!readerRoot || readerRoot.offsetParent === null || editing || totalItems === 0) return;
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
    if (e.key === "ArrowLeft" && currentPos > 0) {
      e.preventDefault();
      goToPosition(currentPos - 1);
    } else if (e.key === "ArrowRight" && currentPos < totalItems - 1) {
      e.preventDefault();
      goToPosition(currentPos + 1);
    }
  }

  async function handleQuickCreateDeck(e: Event) {
    e.preventDefault();
    const name = newDeckName.trim();
    if (!name) return;
    error = "";
    try {
      const created = await decksApi.create({ name });
      newDeckName = "";
      creatingDeck = false;
      selectedDeckId = created.id;
      statusMsg = `Đã tạo bộ thẻ "${created.name}"!`;
      onCardAdded();
    } catch (err) {
      error = String(err);
    }
  }

  async function lookupAndSelect(word: string, sentence?: string) {
    const clean = word.trim();
    if (!clean) return;
    lookingUp = true;
    error = "";
    statusMsg = "";
    lookupResult = null;
    customMeaning = "";
    try {
      const res = await dictionaryApi.lookup(clean);
      lookupResult = res;
      customMeaning = res.meaningVi ?? res.meaningEn ?? "";
      contextSentence = sentence ?? res.exampleSentence ?? "";
    } catch (err) {
      error = String(err);
      if (sentence) contextSentence = sentence;
    } finally {
      lookingUp = false;
    }
  }

  async function handleWordClick(tok: TextToken) {
    if (!tok.isWord) return;
    selectedToken = tok;
    manualQuery = tok.text;
    await lookupAndSelect(tok.text, tok.sentence);
  }

  async function handleManualSearch(e: Event) {
    e.preventDefault();
    selectedToken = null;
    await lookupAndSelect(manualQuery);
  }

  async function handleSaveWithContext() {
    const targetWord = (lookupResult?.word ?? manualQuery).trim();
    if (!targetWord || !selectedDeckId) return;
    const meaning =
      customMeaning.trim() || lookupResult?.meaningVi || lookupResult?.meaningEn || targetWord;
    error = "";
    try {
      await notesApi.create({
        deckId: selectedDeckId,
        noteType: "vocab",
        word: targetWord,
        ipa: lookupResult?.ipa ?? undefined,
        meaning,
        exampleSentence: contextSentence.trim() || undefined,
      });
      statusMsg = `Đã lưu "${targetWord}" vào bộ thẻ!`;
      onCardAdded();
    } catch (err) {
      error = String(err);
    }
  }

  async function clearPdfSession() {
    if (pdfSession) {
      await pdfSession.destroy();
      pdfSession = null;
    }
  }

  async function openUrlTarget(target: string, label?: string) {
    if (!target || parsingFile) return;

    parsingFile = true;
    loadingTarget = label || target;
    error = "";
    statusMsg = "";
    editing = false;
    await clearPdfSession();
    sections = [];
    youtubeId = null;
    youtubeTranscript = [];
    articleBlocks = [];

    try {
      const res = await invokeCmd<WebReaderResult>("load_reader_url", { url: target });
      bookTitle = res.title;
      if (res.kind === "youtube" && res.youtubeId) {
        youtubeId = res.youtubeId;
        youtubeTranscript = res.transcript;
      } else {
        articleText = res.articleText ?? "";
        articleBlocks = res.articleBlocks ?? [];
      }
      const saved = await recordHistoryEntry({
        kind: res.kind === "youtube" ? "youtube" : "article",
        title: res.title,
        subtitle: target,
        url: target,
      });
      activeHistoryId = saved.item.id;
      history = saved.items;
      webUrl = target;
      showUrlBar = false;
    } catch (err) {
      error = String(err);
    } finally {
      parsingFile = false;
      loadingTarget = "";
      openingHistoryId = null;
    }
  }

  async function handleLoadUrl(e: Event) {
    e.preventDefault();
    await openUrlTarget(webUrl.trim());
  }

  async function openFileBuffer(
    fileName: string,
    buffer: ArrayBuffer,
    restorePos?: number,
    cacheInHistory = true,
  ) {
    parsingFile = true;
    loadingTarget = fileName;
    error = "";
    statusMsg = "";
    editing = false;
    youtubeId = null;
    youtubeTranscript = [];
    articleBlocks = [];
    await clearPdfSession();

    try {
      const lower = fileName.toLowerCase();
      let kind: ReaderHistoryItem["kind"] = "text";
      let title = fileName;
      let startPos = restorePos ?? 0;

      if (lower.endsWith(".epub")) {
        kind = "epub";
        const bytes = Array.from(new Uint8Array(buffer));
        const book = await invokeCmd<EpubBook>("parse_epub", { bytes });
        title = book.title || fileName;
        bookTitle = title;
        sections = book.chapters;
        sectionIndex = Math.min(
          Math.max(0, restorePos ?? 0),
          Math.max(0, book.chapters.length - 1),
        );
        startPos = sectionIndex;
      } else if (lower.endsWith(".pdf")) {
        kind = "pdf";
        // Clone buffer before pdf.js transfers/detaches it to a worker
        const session = await openPdfSession(buffer.slice(0));
        pdfSession = session;
        sections = [];
        title = fileName.replace(/\.pdf$/i, "");
        bookTitle = title;
        const page = Math.min(Math.max(1, restorePos ?? 1), session.numPages);
        startPos = page;
        loadPdfPage(page);
      } else {
        kind = "text";
        const raw = new TextDecoder().decode(buffer);
        const isSub = /\.(srt|vtt)$/i.test(lower);
        bookTitle = fileName;
        sections = [];
        articleText = isSub ? stripSubtitleMetadata(raw) : raw;
      }

      if (cacheInHistory) {
        const saved = await recordHistoryEntry(
          {
            kind,
            title,
            subtitle: fileName,
            fileName,
            lastPos: kind === "pdf" || kind === "epub" ? startPos : undefined,
          },
          buffer,
        );
        activeHistoryId = saved.item.id;
        history = saved.items;
      }
    } catch (err) {
      error = String(err);
    } finally {
      parsingFile = false;
      loadingTarget = "";
      openingHistoryId = null;
    }
  }

  async function handleFileUpload(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    try {
      const buffer = await file.arrayBuffer();
      await openFileBuffer(file.name, buffer);
    } finally {
      input.value = "";
    }
  }

  async function handleOpenHistoryItem(item: ReaderHistoryItem) {
    openingHistoryId = item.id;
    if (item.url) {
      await openUrlTarget(item.url, item.title);
      return;
    }
    if (item.fileName) {
      const cached = await loadCachedReaderFile(item.id);
      if (!cached) {
        openingHistoryId = null;
        error = `Không tìm thấy cache cho "${item.title}". Vui lòng chọn lại file từ máy.`;
        return;
      }
      activeHistoryId = item.id;
      await openFileBuffer(item.fileName, cached, item.lastPos, false);
    }
  }

  async function handleDeleteHistoryItem(id: string) {
    history = await removeHistoryEntry(id);
    if (activeHistoryId === id) activeHistoryId = null;
  }

  async function handleClearHistory() {
    history = await clearAllHistory();
    activeHistoryId = null;
  }

  onDestroy(() => {
    void clearPdfSession();
  });
</script>

<svelte:window
  onmouseup={handleReaderTextSelect}
  ontouchend={handleReaderTextSelect}
  onkeydown={handleReaderKeydown}
/>

{#if !hasActiveContent}
  <ReaderHome
    bind:webUrl
    {parsingFile}
    {loadingTarget}
    {openingHistoryId}
    {error}
    {history}
    onLoadUrl={handleLoadUrl}
    onFileUpload={handleFileUpload}
    onStartPasteText={() => {
      bookTitle = "Văn bản tùy chỉnh";
      editing = true;
    }}
    onOpenHistoryItem={handleOpenHistoryItem}
    onDeleteHistoryItem={handleDeleteHistoryItem}
    onClearHistory={handleClearHistory}
  />
{:else}
  <div bind:this={readerRoot} class="grid gap-4 lg:grid-cols-12">
    <div class="flex flex-col space-y-3 lg:col-span-8 xl:col-span-9">
      <div
        class="flex flex-wrap items-center justify-between gap-2 rounded-2xl border border-zinc-800 bg-zinc-900/60 px-4 py-2.5"
      >
        <div class="flex min-w-0 flex-1 items-center gap-2.5">
          <button
            type="button"
            onclick={closeActiveContent}
            title="Quay về danh sách Gần đây"
            class="inline-flex shrink-0 items-center gap-1 rounded-xl border border-zinc-700 px-2.5 py-1.5 text-xs text-zinc-300 hover:bg-zinc-800 hover:text-white"
          >
            <ArrowLeft class="h-3.5 w-3.5" />
            Gần đây
          </button>
          <h2 class="truncate text-sm font-semibold text-white">
            {bookTitle || "Đọc báo, Sách & Video"}
          </h2>
        </div>

        <div class="flex shrink-0 flex-wrap items-center gap-1.5">
          <button
            type="button"
            onclick={() => (showUrlBar = !showUrlBar)}
            class="inline-flex items-center gap-1.5 rounded-xl border px-2.5 py-1.5 text-xs transition {showUrlBar
              ? 'border-indigo-500 bg-indigo-500/15 text-indigo-300'
              : 'border-zinc-700 text-zinc-300 hover:bg-zinc-800'}"
          >
            <Globe class="h-3.5 w-3.5" />
            Mở URL
          </button>
          <label
            class="inline-flex cursor-pointer items-center gap-1.5 rounded-xl border border-zinc-700 px-2.5 py-1.5 text-xs text-zinc-300 hover:bg-zinc-800"
          >
            <Upload class="h-3.5 w-3.5" />
            {parsingFile ? "Đang mở..." : "Mở file"}
            <input
              type="file"
              accept=".epub,.pdf,.txt,.md,.srt,.vtt"
              onchange={handleFileUpload}
              class="hidden"
            />
          </label>
          {#if !youtubeId && !pdfSession && sections.length === 0 && articleBlocks.length === 0}
            <button
              type="button"
              onclick={() => (editing = !editing)}
              class="rounded-xl border border-zinc-700 px-2.5 py-1.5 text-xs font-medium text-zinc-200 hover:bg-zinc-800"
            >
              {editing ? "Xem để đọc" : "Sửa"}
            </button>
          {/if}
        </div>
      </div>

      {#if showUrlBar}
        <form
          onsubmit={handleLoadUrl}
          class="flex items-center gap-2 rounded-xl border border-zinc-800 bg-zinc-900/60 px-3 py-2"
        >
          <Globe class="h-4 w-4 shrink-0 text-indigo-400" />
          <input
            type="url"
            bind:value={webUrl}
            placeholder="Dán link bài báo hoặc YouTube (https://...)..."
            class="flex-1 bg-transparent text-xs text-white placeholder-zinc-500 focus:outline-none"
          />
          <button
            type="submit"
            disabled={parsingFile || !webUrl.trim()}
            class="rounded-lg bg-indigo-600 px-3 py-1 text-xs font-medium text-white hover:bg-indigo-500 disabled:opacity-40"
          >
            {parsingFile ? "Đang tải..." : "Mở"}
          </button>
        </form>
      {/if}

      {#if parsingFile}
        <div
          class="flex items-center gap-2.5 rounded-xl border border-indigo-500/30 bg-indigo-500/10 px-3.5 py-2 text-xs text-indigo-200"
        >
          <Loader2 class="h-4 w-4 shrink-0 animate-spin text-indigo-400" />
          <span class="truncate">Đang tải nội dung: <strong>{loadingTarget}</strong>...</span>
        </div>
      {/if}

      {#if totalItems > 0}
        {@const navOptions = pdfSession
          ? Array.from({ length: pdfSession.numPages }, (_, i) => ({
              value: String(i),
              label: `Trang ${i + 1}`,
            }))
          : sections.map((s, idx) => ({ value: String(idx), label: s.title }))}
        <div
          class="flex flex-wrap items-center justify-between gap-2 rounded-xl border border-zinc-800 bg-zinc-900/50 px-3 py-1.5 text-xs"
        >
          <div class="flex items-center gap-1.5">
            <button
              type="button"
              disabled={currentPos === 0}
              onclick={() => goToPosition(currentPos - 1)}
              title="Trang / Chương trước"
              class="rounded-lg border border-zinc-700 p-1.5 text-zinc-300 hover:bg-zinc-800 disabled:opacity-40"
            >
              <ChevronLeft class="h-3.5 w-3.5" />
            </button>

            <Select
              options={navOptions}
              bind:value={() => String(currentPos), (v) => goToPosition(Number(v))}
              class="w-36"
            />

            <button
              type="button"
              disabled={currentPos >= totalItems - 1}
              onclick={() => goToPosition(currentPos + 1)}
              title="Trang / Chương sau"
              class="rounded-lg border border-zinc-700 p-1.5 text-zinc-300 hover:bg-zinc-800 disabled:opacity-40"
            >
              <ChevronRight class="h-3.5 w-3.5" />
            </button>

            <span class="ml-1 font-mono text-[11px] text-zinc-400">
              {currentPos + 1}/{totalItems}
            </span>
          </div>

          {#if pdfSession}
            <div class="flex items-center gap-1.5">
              <button
                type="button"
                onclick={() => adjustZoom(-0.15)}
                title="Thu nhỏ"
                class="rounded-lg border border-zinc-700 p-1.5 text-zinc-300 hover:bg-zinc-800"
              >
                <ZoomOut class="h-3.5 w-3.5" />
              </button>
              <span class="min-w-11 text-center font-mono text-zinc-200">
                {Math.round(pdfZoom * 100)}%
              </span>
              <button
                type="button"
                onclick={() => adjustZoom(0.15)}
                title="Phóng to"
                class="rounded-lg border border-zinc-700 p-1.5 text-zinc-300 hover:bg-zinc-800"
              >
                <ZoomIn class="h-3.5 w-3.5" />
              </button>

              <div class="mx-1 h-4 w-px bg-zinc-800"></div>

              <button
                type="button"
                onclick={() => (pdfFitMode = "fit-width")}
                title="Vừa chiều ngang"
                class="inline-flex items-center gap-1 rounded-lg border px-2 py-1 transition {pdfFitMode ===
                'fit-width'
                  ? 'border-indigo-500 bg-indigo-500/20 text-indigo-300'
                  : 'border-zinc-700 text-zinc-300 hover:bg-zinc-800'}"
              >
                <MoveHorizontal class="h-3.5 w-3.5" />
                Width
              </button>
              <button
                type="button"
                onclick={() => (pdfFitMode = "fit-page")}
                title="Vừa toàn trang"
                class="inline-flex items-center gap-1 rounded-lg border px-2 py-1 transition {pdfFitMode ===
                'fit-page'
                  ? 'border-indigo-500 bg-indigo-500/20 text-indigo-300'
                  : 'border-zinc-700 text-zinc-300 hover:bg-zinc-800'}"
              >
                <Maximize2 class="h-3.5 w-3.5" />
                Page
              </button>
            </div>
          {/if}
        </div>
      {/if}

      {#if editing}
        <textarea
          rows="16"
          bind:value={articleText}
          placeholder="Dán đoạn văn hoặc bài báo tiếng Anh vào đây..."
          class="h-[calc(100vh-13.5rem)] w-full rounded-2xl border border-zinc-700 bg-zinc-900 p-4 text-sm leading-relaxed text-zinc-100 focus:border-indigo-500 focus:outline-none"
        ></textarea>
      {:else if youtubeId}
        <div bind:this={youtubeContainer}>
          <YouTubePlayer
            videoId={youtubeId}
            transcript={youtubeTranscript}
            selectedWord={selectedToken?.text ?? ""}
            onWordClick={handleWordClick}
          />
        </div>
      {:else if pdfSession}
        <div
          bind:this={pdfContainer}
          class="flex h-[calc(100vh-13.5rem)] items-start justify-center overflow-auto rounded-2xl border border-zinc-800 bg-zinc-950 p-4"
        >
          <div class="relative inline-block shadow-lg">
            <canvas bind:this={pdfCanvas} class="block rounded-lg"></canvas>
            <div bind:this={pdfTextLayer} class="textLayer"></div>
          </div>
        </div>
      {:else}
        {#snippet renderTokens(tokenList: TextToken[])}
          {#each tokenList as tok, tIdx (tIdx)}
            {#if tok.isWord}
              <button
                type="button"
                onclick={() => handleWordClick(tok)}
                class="select-text rounded px-0.5 transition hover:bg-indigo-500/25 hover:text-indigo-200 {selectedToken?.text ===
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
        {/snippet}

        <div
          bind:this={textContainer}
          class="h-[55vh] overflow-y-auto rounded-2xl border border-zinc-800 bg-zinc-900/70 p-4 text-base leading-8 text-zinc-200 sm:p-6 lg:h-[calc(100vh-13.5rem)]"
        >
          {#if articleBlocks.length > 0 && sections.length === 0}
            <article class="mx-auto max-w-3xl space-y-4">
              {#each articleBlocks as block, bIdx (bIdx)}
                {@const blockTokens = tokenizeWithSentences(block.text)}
                {#if block.tag === "h1"}
                  <h1 class="text-2xl leading-snug font-bold text-white">
                    {@render renderTokens(blockTokens)}
                  </h1>
                {:else if block.tag === "h2"}
                  <h2
                    class="mt-6 border-b border-zinc-800 pb-1.5 text-xl font-semibold text-zinc-100"
                  >
                    {@render renderTokens(blockTokens)}
                  </h2>
                {:else if block.tag === "h3"}
                  <h3 class="mt-4 text-lg font-semibold text-zinc-200">
                    {@render renderTokens(blockTokens)}
                  </h3>
                {:else if block.tag === "blockquote"}
                  <blockquote class="border-l-2 border-indigo-500/60 pl-4 text-zinc-300 italic">
                    {@render renderTokens(blockTokens)}
                  </blockquote>
                {:else if block.tag === "li"}
                  <div class="flex gap-2.5 pl-3">
                    <span class="mt-0.5 text-indigo-400">•</span>
                    <div class="flex-1">{@render renderTokens(blockTokens)}</div>
                  </div>
                {:else}
                  <p class="leading-7 text-zinc-200">
                    {@render renderTokens(blockTokens)}
                  </p>
                {/if}
              {/each}
            </article>
          {:else}
            {@render renderTokens(tokens)}
          {/if}
        </div>
      {/if}
    </div>

    <ReaderLookupSidebar
      {decks}
      bind:selectedDeckId
      bind:creatingDeck
      bind:newDeckName
      bind:manualQuery
      {lookingUp}
      {lookupResult}
      bind:customMeaning
      bind:contextSentence
      {statusMsg}
      {error}
      onQuickCreateDeck={handleQuickCreateDeck}
      onManualSearch={handleManualSearch}
      onSaveWithContext={handleSaveWithContext}
    />
  </div>
{/if}
