<script lang="ts">
  import { ChevronDown, ChevronUp, Pencil, Plus, Search, Volume2, X } from "@lucide/svelte";
  import { onDestroy, onMount } from "svelte";
  import { playWordAudio } from "$lib/shared/audio";
  import Select from "$lib/shared/components/Select.svelte";
  import type { DeckWithStats, DictionaryEntry } from "$lib/shared/types";

  export interface LookupAnchorRect {
    left: number;
    top: number;
    right: number;
    bottom: number;
  }

  interface Props {
    open: boolean;
    anchorRect: LookupAnchorRect | null;
    decks: DeckWithStats[];
    selectedDeckId: string;
    creatingDeck: boolean;
    newDeckName: string;
    manualQuery: string;
    lookingUp: boolean;
    lookupResult: DictionaryEntry | null;
    customMeaning: string;
    contextSentence: string;
    statusMsg: string | null;
    error: string | null;
    onQuickCreateDeck: (e: SubmitEvent) => void;
    onManualSearch: (e: SubmitEvent) => void;
    onSaveWithContext: () => void;
    onClose: () => void;
  }

  let {
    open,
    anchorRect,
    decks,
    selectedDeckId = $bindable(),
    creatingDeck = $bindable(),
    newDeckName = $bindable(),
    manualQuery = $bindable(),
    lookingUp,
    lookupResult,
    customMeaning = $bindable(),
    contextSentence = $bindable(),
    statusMsg,
    error,
    onQuickCreateDeck,
    onManualSearch,
    onSaveWithContext,
    onClose,
  }: Props = $props();

  let popupEl = $state<HTMLDivElement | null>(null);
  let isMobile = $state(false);
  let expanded = $state(false);
  let viewportW = $state(1280);
  let viewportH = $state(800);
  let touchStartY = 0;

  // Reset to compact peek mode whenever a new anchor word is clicked on mobile
  $effect(() => {
    if (open) {
      expanded = !anchorRect && !lookupResult;
    }
  });

  function updateViewport() {
    if (typeof window === "undefined") return;
    viewportW = window.innerWidth;
    viewportH = window.innerHeight;
    isMobile = viewportW < 768;
  }

  function handleWindowPointerDown(e: PointerEvent) {
    if (!open || !popupEl) return;
    const target = e.target as Node | null;
    if (!target || popupEl.contains(target)) return;
    // Let word-token buttons update the anchor directly instead of closing then reopening
    if (target instanceof HTMLElement && target.closest("[data-reader-word='true']")) return;
    onClose();
  }

  function handleWindowKeydown(e: KeyboardEvent) {
    if (open && e.key === "Escape") {
      e.preventDefault();
      onClose();
    }
  }

  function handleTouchStart(e: TouchEvent) {
    touchStartY = e.touches[0]?.clientY ?? 0;
  }

  function handleTouchEnd(e: TouchEvent) {
    const endY = e.changedTouches[0]?.clientY ?? 0;
    const delta = endY - touchStartY;
    if (delta > 60) {
      if (expanded) expanded = false;
      else onClose();
    } else if (delta < -50 && !expanded) {
      expanded = true;
    }
  }

  onMount(() => {
    updateViewport();
    window.addEventListener("resize", updateViewport);
    window.addEventListener("pointerdown", handleWindowPointerDown);
    window.addEventListener("keydown", handleWindowKeydown);
  });

  onDestroy(() => {
    if (typeof window === "undefined") return;
    window.removeEventListener("resize", updateViewport);
    window.removeEventListener("pointerdown", handleWindowPointerDown);
    window.removeEventListener("keydown", handleWindowKeydown);
  });

  const desktopStyle = $derived.by(() => {
    const width = Math.min(380, viewportW - 24);
    if (!anchorRect) {
      const left = Math.max(12, Math.round((viewportW - width) / 2));
      return `width: ${width}px; left: ${left}px; top: 76px;`;
    }

    const centerX = (anchorRect.left + anchorRect.right) / 2;
    const left = Math.max(12, Math.min(viewportW - width - 12, Math.round(centerX - width / 2)));
    const spaceBelow = viewportH - anchorRect.bottom - 12;
    const spaceAbove = anchorRect.top - 12;

    if (spaceBelow >= 270 || spaceBelow >= spaceAbove) {
      const top = Math.min(viewportH - 120, Math.round(anchorRect.bottom + 8));
      return `width: ${width}px; left: ${left}px; top: ${top}px; max-height: ${Math.max(220, spaceBelow - 4)}px;`;
    }
    const bottom = Math.round(viewportH - anchorRect.top + 8);
    return `width: ${width}px; left: ${left}px; bottom: ${bottom}px; max-height: ${Math.max(220, spaceAbove - 4)}px;`;
  });

  const primaryMeaning = $derived(
    lookupResult?.senses[0]?.definitionVi ??
      lookupResult?.meaningVi ??
      lookupResult?.senses[0]?.definitionEn ??
      lookupResult?.meaningEn ??
      "",
  );
</script>

{#if open}
  {#if isMobile}
    <!-- Mobile Bottom Sheet -->
    <div
      bind:this={popupEl}
      role="dialog"
      aria-label="Tra cứu nhanh"
      class="fixed inset-x-0 bottom-0 z-50 flex flex-col rounded-t-3xl border-t border-zinc-700/80 bg-zinc-900/95 pb-[env(safe-area-inset-bottom)] shadow-2xl backdrop-blur-md transition-all {expanded
        ? 'max-h-[82vh]'
        : 'max-h-[44vh]'}"
    >
      <!-- Drag / Swipe Handle -->
      <div class="flex shrink-0 items-center justify-between px-4 pt-2.5 pb-1.5">
        <div class="w-8"></div>
        <button
          type="button"
          ontouchstart={handleTouchStart}
          ontouchend={handleTouchEnd}
          onclick={() => (expanded = !expanded)}
          aria-label={expanded ? "Thu gọn" : "Mở rộng"}
          class="flex h-6 w-24 items-center justify-center"
        >
          <span class="h-1 w-10 rounded-full bg-zinc-600"></span>
        </button>
        <div class="flex items-center gap-1">
          <button
            type="button"
            onclick={() => (expanded = !expanded)}
            title={expanded ? "Thu gọn" : "Mở rộng chi tiết"}
            class="rounded-lg p-1 text-zinc-400 hover:bg-zinc-800 hover:text-white"
          >
            {#if expanded}
              <ChevronDown class="h-4 w-4" />
            {:else}
              <ChevronUp class="h-4 w-4" />
            {/if}
          </button>
          <button
            type="button"
            onclick={onClose}
            title="Đóng"
            class="rounded-lg p-1 text-zinc-400 hover:bg-zinc-800 hover:text-white"
          >
            <X class="h-4 w-4" />
          </button>
        </div>
      </div>

      <div class="flex-1 space-y-3 overflow-y-auto px-4 pb-4">
        {#if !expanded && lookupResult && !lookingUp}
          <!-- Mobile Peek View: Word + IPA + Primary Meaning + 1-tap Save -->
          <div class="flex items-start justify-between gap-2">
            <div class="min-w-0">
              <div class="flex flex-wrap items-center gap-1.5">
                <h3 class="text-lg font-bold text-white">{lookupResult.word}</h3>
                {#if lookupResult.matchedForm}
                  <span class="rounded bg-amber-500/15 px-1.5 py-0.5 text-[10px] text-amber-300">
                    ← {lookupResult.matchedForm}
                  </span>
                {/if}
              </div>
              <div class="flex flex-wrap gap-2 font-mono text-xs">
                {#if lookupResult.ipaUs}
                  <span class="text-indigo-400">US {lookupResult.ipaUs}</span>
                {/if}
                {#if lookupResult.ipaUk && lookupResult.ipaUk !== lookupResult.ipaUs}
                  <span class="text-sky-400">UK {lookupResult.ipaUk}</span>
                {:else if !lookupResult.ipaUs && lookupResult.ipa}
                  <span class="text-indigo-400">{lookupResult.ipa}</span>
                {/if}
              </div>
            </div>
            <button
              type="button"
              onclick={() => playWordAudio(lookupResult!.word)}
              class="shrink-0 rounded-xl border border-zinc-700 bg-zinc-800/80 p-2 text-zinc-200 hover:border-indigo-500"
            >
              <Volume2 class="h-4 w-4" />
            </button>
          </div>

          <p class="text-sm leading-relaxed text-emerald-400">
            {primaryMeaning || "Chưa có nghĩa"}
          </p>

          <div class="flex items-center gap-2 pt-1">
            <button
              type="button"
              onclick={onSaveWithContext}
              disabled={!selectedDeckId}
              class="flex flex-1 items-center justify-center gap-1.5 rounded-xl bg-indigo-600 py-2.5 text-xs font-semibold text-white hover:bg-indigo-500 disabled:opacity-40"
            >
              <Plus class="h-3.5 w-3.5" />
              Lưu nhanh
            </button>
            <button
              type="button"
              onclick={() => (expanded = true)}
              class="inline-flex items-center gap-1 rounded-xl border border-zinc-700 px-3 py-2.5 text-xs text-zinc-200 hover:bg-zinc-800"
            >
              <Pencil class="h-3.5 w-3.5" />
              Chi tiết & Sửa
            </button>
          </div>
        {:else}
          {@render fullContent()}
        {/if}

        {#if statusMsg}
          <p class="rounded-xl bg-emerald-500/10 p-2.5 text-xs text-emerald-400">{statusMsg}</p>
        {/if}
        {#if error}
          <p class="rounded-xl bg-rose-500/10 p-2.5 text-xs text-rose-400">{error}</p>
        {/if}
      </div>
    </div>
  {:else}
    <!-- Desktop Anchored Floating Popover -->
    <div
      bind:this={popupEl}
      role="dialog"
      aria-label="Tra cứu nhanh"
      style={desktopStyle}
      class="fixed z-50 flex flex-col overflow-hidden rounded-2xl border border-zinc-700/80 bg-zinc-900/95 shadow-2xl backdrop-blur-md"
    >
      <div class="flex shrink-0 items-center justify-between border-b border-zinc-800 px-3.5 py-2">
        <span class="text-xs font-semibold text-zinc-300">Tra cứu nhanh</span>
        <div class="flex items-center gap-1">
          <button
            type="button"
            onclick={() => (expanded = !expanded)}
            title={expanded ? "Thu gọn form sửa" : "Mở rộng chi tiết & sửa thẻ"}
            class="inline-flex items-center gap-1 rounded-lg px-2 py-1 text-[11px] text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200"
          >
            <Pencil class="h-3 w-3" />
            {expanded ? "Gọn" : "Sửa & Lưu"}
          </button>
          <button
            type="button"
            onclick={onClose}
            title="Đóng (Esc)"
            class="rounded-lg p-1 text-zinc-400 hover:bg-zinc-800 hover:text-white"
          >
            <X class="h-3.5 w-3.5" />
          </button>
        </div>
      </div>

      <div class="flex-1 space-y-3 overflow-y-auto p-3.5">
        {@render fullContent()}

        {#if statusMsg}
          <p class="rounded-xl bg-emerald-500/10 p-2.5 text-xs text-emerald-400">{statusMsg}</p>
        {/if}
        {#if error}
          <p class="rounded-xl bg-rose-500/10 p-2.5 text-xs text-rose-400">{error}</p>
        {/if}
      </div>
    </div>
  {/if}
{/if}

{#snippet fullContent()}
  {#if expanded || !lookupResult}
    <div class="flex items-center gap-1.5">
      {#if decks.length > 0}
        <Select
          options={decks.map((d) => ({ value: d.id, label: d.name }))}
          bind:value={selectedDeckId}
          prefix="Lưu vào: "
          class="flex-1"
        />
      {:else}
        <span class="flex-1 text-xs text-zinc-400">Chưa có bộ thẻ nào</span>
      {/if}
      <button
        type="button"
        onclick={() => (creatingDeck = !creatingDeck)}
        title="Tạo nhanh bộ thẻ mới"
        class="shrink-0 rounded-xl border border-zinc-700 p-2 text-zinc-300 hover:border-indigo-500 hover:text-indigo-300"
      >
        <Plus class="h-3.5 w-3.5" />
      </button>
    </div>

    {#if creatingDeck}
      <form onsubmit={onQuickCreateDeck} class="flex gap-1.5">
        <input
          type="text"
          bind:value={newDeckName}
          placeholder="Tên bộ thẻ mới..."
          class="flex-1 rounded-xl border border-zinc-700 bg-zinc-950 px-2.5 py-1.5 text-xs text-white placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
        />
        <button
          type="submit"
          disabled={!newDeckName.trim()}
          class="rounded-xl bg-indigo-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-indigo-500 disabled:opacity-40"
        >
          Tạo
        </button>
      </form>
    {/if}

    <form onsubmit={onManualSearch} class="flex gap-1.5">
      <div class="relative flex-1">
        <Search class="absolute top-1/2 left-2.5 h-3.5 w-3.5 -translate-y-1/2 text-zinc-500" />
        <input
          type="text"
          bind:value={manualQuery}
          placeholder="Nhập từ, cụm từ hoặc câu..."
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
  {/if}

  {#if lookingUp}
    <p class="py-5 text-center text-xs text-zinc-400">Đang tra từ...</p>
  {:else if lookupResult}
    <div class="space-y-2.5 {expanded ? 'border-t border-zinc-800 pt-2.5' : ''}">
      <div class="flex items-center justify-between gap-2">
        <div class="min-w-0">
          <div class="flex flex-wrap items-center gap-1.5">
            <p class="text-lg font-bold text-white">{lookupResult.word}</p>
            {#if lookupResult.matchedForm}
              <span class="rounded bg-amber-500/15 px-1.5 py-0.5 text-[10px] text-amber-300">
                ← {lookupResult.matchedForm}
              </span>
            {/if}
          </div>
          <div class="flex flex-wrap gap-2 font-mono text-xs">
            {#if lookupResult.ipaUs}
              <span class="text-indigo-400">US {lookupResult.ipaUs}</span>
            {/if}
            {#if lookupResult.ipaUk && lookupResult.ipaUk !== lookupResult.ipaUs}
              <span class="text-sky-400">UK {lookupResult.ipaUk}</span>
            {:else if !lookupResult.ipaUs && lookupResult.ipa}
              <span class="text-indigo-400">{lookupResult.ipa}</span>
            {/if}
          </div>
        </div>
        <button
          type="button"
          onclick={() => playWordAudio(lookupResult!.word)}
          class="shrink-0 rounded-xl border border-zinc-700 p-2 text-zinc-300 hover:border-indigo-500"
        >
          <Volume2 class="h-4 w-4" />
        </button>
      </div>

      {#if lookupResult.senses.length > 0}
        <div class="max-h-44 space-y-2 overflow-y-auto pr-1">
          {#each lookupResult.senses as sense, sIdx (sIdx)}
            <div class="space-y-0.5 border-l-2 border-zinc-800 pl-2.5 text-xs">
              <div class="flex flex-wrap items-center gap-1">
                <span class="rounded bg-indigo-500/15 px-1 py-0.5 text-[10px] text-indigo-300">
                  {sense.pos}
                </span>
                {#each sense.tags as tag, tagIdx (tagIdx)}
                  <span class="rounded bg-zinc-800 px-1 py-0.5 text-[10px] text-zinc-400">
                    {tag}
                  </span>
                {/each}
              </div>
              {#if sense.definitionVi}
                <p class="text-emerald-400">{sense.definitionVi}</p>
              {/if}
              {#if sense.definitionEn}
                <p class="text-zinc-300">{sense.definitionEn}</p>
              {/if}
              {#if sense.synonyms.length > 0}
                <p class="text-[11px] text-zinc-400">
                  Đồng nghĩa: {sense.synonyms.join(", ")}
                </p>
              {/if}
            </div>
          {/each}
        </div>
      {:else}
        <p class="text-sm text-emerald-400">
          {lookupResult.meaningVi ?? lookupResult.meaningEn ?? "Chưa có nghĩa"}
        </p>
      {/if}

      {#if expanded}
        <div>
          <label class="mb-1 block text-[11px] text-zinc-400" for="reader-meaning">
            Nghĩa / Bản dịch (có thể sửa trước khi lưu)
          </label>
          <textarea
            id="reader-meaning"
            rows="2"
            bind:value={customMeaning}
            placeholder="Nhập hoặc chỉnh sửa nghĩa..."
            class="w-full rounded-xl border border-zinc-700 bg-zinc-950 p-2 text-xs text-zinc-200 placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
          ></textarea>
        </div>

        <div>
          <label class="mb-1 block text-[11px] text-zinc-400" for="reader-context">
            Câu ngữ cảnh (tuỳ chọn)
          </label>
          <textarea
            id="reader-context"
            rows="2"
            bind:value={contextSentence}
            placeholder="Nhập hoặc sửa câu ví dụ..."
            class="w-full rounded-xl border border-zinc-700 bg-zinc-950 p-2 text-xs text-zinc-200 placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
          ></textarea>
        </div>
      {/if}

      <button
        type="button"
        onclick={onSaveWithContext}
        disabled={!selectedDeckId}
        class="flex w-full items-center justify-center gap-1.5 rounded-xl bg-indigo-600 py-2 text-xs font-semibold text-white hover:bg-indigo-500 disabled:opacity-40"
      >
        <Plus class="h-3.5 w-3.5" />
        Lưu vào bộ thẻ
      </button>
    </div>
  {:else if manualQuery.trim()}
    <div class="space-y-2.5 border-t border-zinc-800 pt-2.5">
      <p class="text-sm font-semibold text-white">{manualQuery.trim()}</p>
      <div>
        <label class="mb-1 block text-[11px] text-zinc-400" for="reader-meaning-fallback">
          Nghĩa / Bản dịch
        </label>
        <textarea
          id="reader-meaning-fallback"
          rows="2"
          bind:value={customMeaning}
          placeholder="Nhập nghĩa hoặc bản dịch cho cụm từ / câu này..."
          class="w-full rounded-xl border border-zinc-700 bg-zinc-950 p-2 text-xs text-zinc-200 placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
        ></textarea>
      </div>
      <button
        type="button"
        onclick={onSaveWithContext}
        disabled={!selectedDeckId}
        class="flex w-full items-center justify-center gap-1.5 rounded-xl bg-indigo-600 py-2 text-xs font-semibold text-white hover:bg-indigo-500 disabled:opacity-40"
      >
        <Plus class="h-3.5 w-3.5" />
        Lưu vào bộ thẻ
      </button>
    </div>
  {:else}
    <p class="py-4 text-center text-xs text-zinc-500">
      Click 1 từ hoặc bôi đen cụm từ / câu để tra nghĩa nhanh.
    </p>
  {/if}
{/snippet}
