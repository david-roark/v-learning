<script lang="ts">
  import {
    BookOpenText,
    ChevronLeft,
    ChevronRight,
    Eye,
    FileText,
    Maximize2,
    MoveHorizontal,
    Plus,
    Search,
    Upload,
    Volume2,
    ZoomIn,
    ZoomOut,
  } from "@lucide/svelte";
  import { onDestroy } from "svelte";
  import { notesApi } from "$lib/features/card-editor/api";
  import { dictionaryApi } from "$lib/features/dictionary/api";
  import { playWordAudio } from "$lib/shared/audio";
  import Select from "$lib/shared/components/Select.svelte";
  import { invokeCmd } from "$lib/shared/tauri";
  import type { DeckWithStats, DictionaryEntry } from "$lib/shared/types";
  import { openPdfSession, type BookSection, type PdfFitMode, type PdfSession } from "./pdfParser";
  import { stripSubtitleMetadata, tokenizeWithSentences, type TextToken } from "./textParser";

  interface Props {
    decks: DeckWithStats[];
    onCardAdded: () => void;
  }

  interface EpubBook {
    title: string;
    chapters: BookSection[];
  }

  let { decks, onCardAdded }: Props = $props();

  let articleText = $state(
    "Smartphones have become ubiquitous in modern life. Taking a pragmatic approach helps learners stay resilient when facing ambiguous grammar rules.",
  );
  let bookTitle = $state("");
  let sections = $state<BookSection[]>([]);
  let sectionIndex = $state(0);
  let pdfSession = $state<PdfSession | null>(null);
  let pdfPageNum = $state(1);
  let pdfViewMode = $state<"text" | "canvas">("text");
  let pdfFitMode = $state<PdfFitMode>("fit-width");
  let pdfZoom = $state(1.25);
  let pdfCanvas = $state<HTMLCanvasElement | null>(null);
  let pdfContainer = $state<HTMLDivElement | null>(null);
  let editing = $state(false);
  let parsingFile = $state(false);

  let selectedDeckId = $state("");
  let manualQuery = $state("");
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

  async function loadPdfPage(pageNum: number) {
    if (!pdfSession || pageNum < 1 || pageNum > pdfSession.numPages) return;
    pdfPageNum = pageNum;
    selectedToken = null;
    lookupResult = null;
    articleText = await pdfSession.getPageText(pageNum);
  }

  $effect(() => {
    if (pdfSession && pdfViewMode === "canvas" && pdfCanvas && pdfContainer) {
      const opts = {
        scale: pdfZoom,
        fitMode: pdfFitMode,
        containerWidth: pdfContainer.clientWidth,
        containerHeight: pdfContainer.clientHeight,
      };
      void pdfSession.renderPage(pdfPageNum, pdfCanvas, opts).then((actualScale) => {
        if (pdfFitMode !== "custom") {
          pdfZoom = Math.round(actualScale * 100) / 100;
        }
      });
    }
  });

  function adjustZoom(delta: number) {
    pdfFitMode = "custom";
    pdfZoom = Math.max(0.4, Math.min(3.0, Math.round((pdfZoom + delta) * 100) / 100));
  }

  function goToPosition(idx: number) {
    if (pdfSession) {
      void loadPdfPage(idx + 1);
      return;
    }
    if (idx < 0 || idx >= sections.length) return;
    sectionIndex = idx;
    selectedToken = null;
    lookupResult = null;
  }

  async function lookupAndSelect(word: string, sentence?: string) {
    const clean = word.trim();
    if (!clean) return;
    lookingUp = true;
    error = "";
    statusMsg = "";
    lookupResult = null;
    try {
      const res = await dictionaryApi.lookup(clean);
      lookupResult = res;
      contextSentence = sentence ?? res.exampleSentence ?? "";
    } catch (err) {
      error = String(err);
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
    if (!lookupResult || !selectedDeckId) return;
    error = "";
    try {
      await notesApi.create({
        deckId: selectedDeckId,
        noteType: "vocab",
        word: lookupResult.word,
        ipa: lookupResult.ipa ?? undefined,
        meaning: lookupResult.meaningVi ?? lookupResult.meaningEn ?? lookupResult.word,
        exampleSentence: contextSentence.trim() || undefined,
      });
      statusMsg = `Đã lưu "${lookupResult.word}" vào bộ thẻ!`;
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

  async function handleFileUpload(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    parsingFile = true;
    error = "";
    statusMsg = "";
    editing = false;
    await clearPdfSession();

    try {
      const name = file.name.toLowerCase();
      if (name.endsWith(".epub")) {
        const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
        const book = await invokeCmd<EpubBook>("parse_epub", { bytes });
        bookTitle = book.title;
        sections = book.chapters;
        sectionIndex = 0;
      } else if (name.endsWith(".pdf")) {
        const session = await openPdfSession(await file.arrayBuffer());
        pdfSession = session;
        sections = [];
        bookTitle = file.name.replace(/\.pdf$/i, "");
        pdfViewMode = "text";
        await loadPdfPage(1);
      } else {
        const raw = await file.text();
        const isSub = /\.(srt|vtt)$/i.test(name);
        bookTitle = file.name;
        sections = [];
        articleText = isSub ? stripSubtitleMetadata(raw) : raw;
      }
    } catch (err) {
      error = String(err);
    } finally {
      parsingFile = false;
      input.value = "";
    }
  }

  onDestroy(() => {
    void clearPdfSession();
  });
</script>

<div class="grid gap-4 lg:grid-cols-12">
  <div class="flex flex-col space-y-3 lg:col-span-8 xl:col-span-9">
    <div
      class="flex flex-wrap items-center justify-between gap-2 rounded-2xl border border-zinc-800 bg-zinc-900/60 px-4 py-3"
    >
      <div class="flex items-center gap-2.5">
        <BookOpenText class="h-5 w-5 text-indigo-400" />
        <div>
          <h2 class="text-sm font-semibold text-white">
            {bookTitle || "Trình đọc & Khai thác câu (Sentence Mining)"}
          </h2>
          <p class="text-xs text-zinc-400">Hỗ trợ .epub, .pdf, .txt, .md, .srt, .vtt</p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        {#if pdfSession}
          <div class="flex rounded-xl border border-zinc-700 bg-zinc-950 p-0.5 text-xs">
            <button
              type="button"
              onclick={() => (pdfViewMode = "text")}
              class="inline-flex items-center gap-1 rounded-lg px-2.5 py-1 transition {pdfViewMode ===
              'text'
                ? 'bg-indigo-600 text-white'
                : 'text-zinc-400 hover:text-zinc-200'}"
            >
              <FileText class="h-3.5 w-3.5" />
              Text Mining
            </button>
            <button
              type="button"
              onclick={() => (pdfViewMode = "canvas")}
              class="inline-flex items-center gap-1 rounded-lg px-2.5 py-1 transition {pdfViewMode ===
              'canvas'
                ? 'bg-indigo-600 text-white'
                : 'text-zinc-400 hover:text-zinc-200'}"
            >
              <Eye class="h-3.5 w-3.5" />
              Trang gốc
            </button>
          </div>
        {/if}

        <label
          class="inline-flex cursor-pointer items-center gap-1.5 rounded-xl border border-zinc-700 px-3 py-1.5 text-xs text-zinc-300 hover:bg-zinc-800"
        >
          <Upload class="h-3.5 w-3.5" />
          {parsingFile ? "Đang mở..." : "Mở sách / file"}
          <input
            type="file"
            accept=".epub,.pdf,.txt,.md,.srt,.vtt"
            onchange={handleFileUpload}
            class="hidden"
          />
        </label>
        <button
          type="button"
          onclick={async () => {
            await clearPdfSession();
            sections = [];
            bookTitle = "";
            editing = !editing;
          }}
          class="rounded-xl border border-zinc-700 px-3 py-1.5 text-xs font-medium text-zinc-200 hover:bg-zinc-800"
        >
          {editing ? "Xem để đọc" : "Dán văn bản"}
        </button>
      </div>
    </div>

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

        {#if pdfSession && pdfViewMode === "canvas"}
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
    {:else if pdfSession && pdfViewMode === "canvas"}
      <div
        bind:this={pdfContainer}
        class="flex h-[calc(100vh-13.5rem)] items-start justify-center overflow-auto rounded-2xl border border-zinc-800 bg-zinc-950 p-4"
      >
        <canvas bind:this={pdfCanvas} class="rounded-lg shadow-lg"></canvas>
      </div>
    {:else}
      <div
        class="h-[calc(100vh-13.5rem)] overflow-y-auto rounded-2xl border border-zinc-800 bg-zinc-900/70 p-6 text-base leading-8 text-zinc-200"
      >
        {#if pdfSession && !displayText}
          <p class="py-10 text-center text-sm text-zinc-500">
            Trang này không có lớp chữ (hoặc là trang ảnh). Bạn có thể chuyển sang chế độ <strong
              >Trang gốc</strong
            > ở góc trên để xem.
          </p>
        {:else}
          {#each tokens as tok}
            {#if tok.isWord}
              <button
                type="button"
                onclick={() => handleWordClick(tok)}
                class="rounded px-0.5 transition hover:bg-indigo-500/25 hover:text-indigo-200 {selectedToken?.text ===
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
        {/if}
      </div>
    {/if}
  </div>

  <!-- Side Lookup Panel -->
  <div
    class="h-fit space-y-3 rounded-2xl border border-zinc-800 bg-zinc-900/70 p-4 lg:sticky lg:top-0 lg:col-span-4 xl:col-span-3"
  >
    <h3 class="text-sm font-semibold text-white">Tra cứu nhanh</h3>

    {#if decks.length > 0}
      <Select
        options={decks.map((d) => ({ value: d.id, label: d.name }))}
        bind:value={selectedDeckId}
        prefix="Lưu vào: "
      />
    {/if}

    <form onsubmit={handleManualSearch} class="flex gap-1.5">
      <div class="relative flex-1">
        <Search class="absolute top-1/2 left-2.5 h-3.5 w-3.5 -translate-y-1/2 text-zinc-500" />
        <input
          type="text"
          bind:value={manualQuery}
          placeholder="Nhập từ cần tra..."
          class="w-full rounded-xl border border-zinc-700 bg-zinc-950 py-2 pr-2.5 pl-8 text-xs text-white placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
        />
      </div>
      <button
        type="submit"
        disabled={lookingUp || !manualQuery.trim()}
        class="rounded-xl bg-zinc-800 px-3 py-2 text-xs font-medium text-zinc-200 hover:bg-indigo-600 hover:text-white disabled:opacity-40"
      >
        Tra
      </button>
    </form>

    {#if lookingUp}
      <p class="py-6 text-center text-xs text-zinc-400">Đang tra từ...</p>
    {:else if lookupResult}
      <div class="space-y-3 border-t border-zinc-800 pt-3">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-lg font-bold text-white">{lookupResult.word}</p>
            {#if lookupResult.ipa}
              <p class="font-mono text-xs text-indigo-400">{lookupResult.ipa}</p>
            {/if}
          </div>
          <button
            type="button"
            onclick={() => playWordAudio(lookupResult!.word)}
            class="rounded-xl border border-zinc-700 p-2 text-zinc-300 hover:border-indigo-500"
          >
            <Volume2 class="h-4 w-4" />
          </button>
        </div>

        <p class="text-sm text-emerald-400">
          {lookupResult.meaningVi ?? lookupResult.meaningEn ?? "Chưa có nghĩa"}
        </p>

        <div>
          <label class="mb-1 block text-[11px] text-zinc-400" for="reader-context">
            Câu ngữ cảnh (tuỳ chọn)
          </label>
          <textarea
            id="reader-context"
            rows="2"
            bind:value={contextSentence}
            placeholder="Nhập hoặc sửa câu ví dụ..."
            class="w-full rounded-xl border border-zinc-700 bg-zinc-950 p-2.5 text-xs text-zinc-200 placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
          ></textarea>
        </div>

        <button
          type="button"
          onclick={handleSaveWithContext}
          disabled={!selectedDeckId}
          class="flex w-full items-center justify-center gap-1.5 rounded-xl bg-indigo-600 py-2.5 text-xs font-semibold text-white hover:bg-indigo-500 disabled:opacity-40"
        >
          <Plus class="h-3.5 w-3.5" />
          Lưu vào bộ thẻ
        </button>
      </div>
    {:else}
      <p class="py-6 text-center text-xs text-zinc-500">
        Click từ trong văn bản hoặc nhập từ vào ô trên để tra và lưu thành flashcard.
      </p>
    {/if}

    {#if statusMsg}
      <p class="rounded-xl bg-emerald-500/10 p-2.5 text-xs text-emerald-400">{statusMsg}</p>
    {/if}
    {#if error}
      <p class="rounded-xl bg-rose-500/10 p-2.5 text-xs text-rose-400">{error}</p>
    {/if}
  </div>
</div>
