<script lang="ts">
  import {
    AlertTriangle,
    BookOpen,
    CheckCircle2,
    List,
    Play,
    Plus,
    Search,
    Trash2,
  } from "@lucide/svelte";
  import type { DeckWithStats } from "$lib/shared/types";

  interface Props {
    decks: DeckWithStats[];
    onStudy: (deck: DeckWithStats) => void;
    onAddCard: (deck: DeckWithStats) => void;
    onManageNotes: (deck: DeckWithStats) => void;
    onDelete: (deck: DeckWithStats) => void;
    onCreateDeck: () => void;
  }

  let { decks, onStudy, onAddCard, onManageNotes, onDelete, onCreateDeck }: Props = $props();

  let searchQuery = $state("");
  let onlyDue = $state(false);
  let confirmDeleteTarget = $state<DeckWithStats | null>(null);

  const filteredDecks = $derived.by(() => {
    const q = searchQuery.trim().toLowerCase();
    return decks.filter((d) => {
      if (onlyDue && d.newCount + d.dueCount === 0) return false;
      if (!q) return true;
      return (
        d.name.toLowerCase().includes(q) ||
        (d.description?.toLowerCase().includes(q) ?? false)
      );
    });
  });

  function handleConfirmDelete() {
    if (!confirmDeleteTarget) return;
    const target = confirmDeleteTarget;
    confirmDeleteTarget = null;
    onDelete(target);
  }
</script>

<svelte:window
  onkeydown={(e) => {
    if (confirmDeleteTarget && e.key === "Escape") {
      confirmDeleteTarget = null;
    }
  }}
/>

{#if decks.length === 0}
  <div
    class="flex flex-col items-center justify-center rounded-2xl border border-dashed border-zinc-800 bg-zinc-900/40 px-6 py-16 text-center"
  >
    <div class="mb-4 rounded-2xl bg-indigo-500/10 p-4 text-indigo-400">
      <BookOpen class="h-8 w-8" />
    </div>
    <h3 class="text-lg font-semibold text-zinc-100">Chưa có bộ thẻ nào</h3>
    <p class="mt-1.5 max-w-sm text-sm text-zinc-400">
      Tạo bộ thẻ đầu tiên để bắt đầu học tiếng Anh với thuật toán lặp lại ngắt quãng FSRS.
    </p>
    <button
      type="button"
      onclick={onCreateDeck}
      class="mt-6 inline-flex items-center gap-2 rounded-xl bg-indigo-600 px-4 py-2.5 text-sm font-medium text-white hover:bg-indigo-500"
    >
      <Plus class="h-4 w-4" />
      Tạo bộ thẻ đầu tiên
    </button>
  </div>
{:else}
  <!-- Search & Filter Bar -->
  <div class="mb-4 flex flex-wrap items-center justify-between gap-2">
    <div class="relative min-w-48 flex-1 sm:max-w-xs">
      <Search class="absolute top-1/2 left-3 h-3.5 w-3.5 -translate-y-1/2 text-zinc-500" />
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Tìm bộ thẻ..."
        class="w-full rounded-xl border border-zinc-800 bg-zinc-900/70 py-1.5 pr-3 pl-8 text-xs text-zinc-100 placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
      />
    </div>

    <div class="flex items-center gap-2 text-xs">
      <button
        type="button"
        onclick={() => (onlyDue = !onlyDue)}
        class="rounded-xl border px-3 py-1.5 font-medium transition {onlyDue
          ? 'border-indigo-500/60 bg-indigo-500/15 text-indigo-300'
          : 'border-zinc-800 bg-zinc-900/60 text-zinc-400 hover:text-zinc-200'}"
      >
        Cần ôn hôm nay
      </button>
      <span class="text-[11px] text-zinc-500">
        {filteredDecks.length}/{decks.length} bộ
      </span>
    </div>
  </div>

  {#if filteredDecks.length === 0}
    <div
      class="rounded-2xl border border-dashed border-zinc-800 bg-zinc-900/30 py-12 text-center text-xs text-zinc-400"
    >
      Không tìm thấy bộ thẻ phù hợp với bộ lọc hiện tại.
    </div>
  {:else}
    <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
      {#each filteredDecks as deck (deck.id)}
        {@const readyCount = deck.newCount + deck.dueCount}
        {@const learnedCount = Math.max(0, deck.totalCards - deck.newCount)}
        {@const progressPct =
          deck.totalCards > 0 ? Math.round((learnedCount / deck.totalCards) * 100) : 0}
        <div
          class="group flex flex-col justify-between rounded-2xl border p-5 transition {readyCount >
          0
            ? 'border-zinc-800 bg-zinc-900/80 hover:border-indigo-500/50 hover:bg-zinc-900'
            : 'border-zinc-800/80 bg-zinc-900/50 hover:border-zinc-700'}"
        >
          <div class="flex flex-1 flex-col">
            <!-- Header -->
            <div class="flex items-start justify-between gap-2">
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <h3 class="truncate text-base font-semibold text-zinc-100">{deck.name}</h3>
                  {#if deck.totalCards > 0 && readyCount === 0}
                    <CheckCircle2 class="h-4 w-4 shrink-0 text-emerald-400" />
                  {/if}
                </div>
                <p class="mt-0.5 text-[11px] text-zinc-400">
                  FSRS {Math.round(deck.desiredRetention * 100)}% · {deck.newCardsPerDay} thẻ mới/ngày
                </p>
              </div>
              <button
                type="button"
                onclick={() => (confirmDeleteTarget = deck)}
                title="Xoá bộ thẻ"
                class="shrink-0 rounded-lg p-1.5 text-zinc-400 transition sm:opacity-0 sm:group-hover:opacity-100 hover:bg-rose-500/10 hover:text-rose-400"
              >
                <Trash2 class="h-4 w-4" />
              </button>
            </div>

            <p class="mt-2 min-h-8 line-clamp-2 text-xs text-zinc-400">
              {deck.description ?? ""}
            </p>

            <div class="mt-auto pt-3">
              <!-- Progress Bar -->
              <div class="space-y-1.5">
                <div class="flex items-center justify-between text-[11px] text-zinc-400">
                  <span>Tiến độ đã học</span>
                  <span class="font-mono text-zinc-300"
                    >{learnedCount}/{deck.totalCards} ({progressPct}%)</span
                  >
                </div>
                <div class="h-1.5 w-full overflow-hidden rounded-full bg-zinc-800">
                  <div
                    class="h-full rounded-full bg-indigo-600 transition-all"
                    style={`width: ${progressPct}%`}
                  ></div>
                </div>
              </div>

              <!-- Counters -->
              <div class="mt-3.5 grid grid-cols-3 gap-2 text-center text-xs">
                <div class="rounded-xl bg-sky-500/10 px-2 py-1.5">
                  <p class="font-mono text-sm font-bold text-sky-400">{deck.newCount}</p>
                  <p class="text-[10px] text-sky-400">Mới</p>
                </div>
                <div class="rounded-xl bg-amber-500/10 px-2 py-1.5">
                  <p class="font-mono text-sm font-bold text-amber-400">{deck.dueCount}</p>
                  <p class="text-[10px] text-amber-400">Cần ôn</p>
                </div>
                <div class="rounded-xl bg-zinc-800 px-2 py-1.5">
                  <p class="font-mono text-sm font-bold text-zinc-200">{deck.totalCards}</p>
                  <p class="text-[10px] text-zinc-400">Tổng thẻ</p>
                </div>
              </div>
            </div>
          </div>

          <!-- Actions -->
          <div class="mt-4 flex items-center gap-2 border-t border-zinc-800/80 pt-3.5">
            <button
              type="button"
              onclick={() => onStudy(deck)}
              disabled={readyCount === 0}
              class="flex flex-1 items-center justify-center gap-1.5 rounded-xl px-3 py-2 text-xs font-semibold whitespace-nowrap transition {readyCount >
              0
                ? 'bg-indigo-600 text-white hover:bg-indigo-500'
                : 'cursor-not-allowed border border-zinc-700 bg-zinc-800 text-zinc-300 opacity-75'}"
            >
              <Play class="h-3.5 w-3.5 shrink-0" />
              {readyCount > 0 ? `Học ngay (${readyCount})` : "Đã xong"}
            </button>
            <button
              type="button"
              onclick={() => onAddCard(deck)}
              title="Thêm thẻ mới"
              class="inline-flex items-center gap-1 rounded-xl border border-zinc-700 px-3 py-2 text-xs font-medium text-zinc-300 hover:bg-zinc-800"
            >
              <Plus class="h-3.5 w-3.5" />
              Thẻ
            </button>
            <button
              type="button"
              onclick={() => onManageNotes(deck)}
              title="Xem danh sách thẻ"
              class="inline-flex items-center rounded-xl border border-zinc-700 p-2 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200"
            >
              <List class="h-3.5 w-3.5" />
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
{/if}

{#if confirmDeleteTarget}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-xs"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) confirmDeleteTarget = null;
    }}
  >
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="delete-deck-title"
      class="w-full max-w-md rounded-2xl border border-zinc-800 bg-zinc-900 p-6 shadow-2xl"
    >
      <div class="flex items-start gap-3">
        <div class="shrink-0 rounded-xl bg-rose-500/15 p-2.5 text-rose-400">
          <AlertTriangle class="h-5 w-5" />
        </div>
        <div>
          <h3 id="delete-deck-title" class="text-base font-semibold text-white">
            Xoá bộ thẻ "{confirmDeleteTarget.name}"?
          </h3>
          <p class="mt-1.5 text-xs leading-relaxed text-zinc-400">
            Bộ thẻ này đang có <strong class="text-zinc-200">{confirmDeleteTarget.totalCards}</strong>
            thẻ. Toàn bộ từ vựng, thẻ và lịch sử ôn tập thuộc bộ này sẽ bị xoá vĩnh viễn và không thể
            hoàn tác.
          </p>
        </div>
      </div>

      <div class="mt-6 flex justify-end gap-2.5">
        <button
          type="button"
          onclick={() => (confirmDeleteTarget = null)}
          class="rounded-xl border border-zinc-700 px-4 py-2 text-xs font-medium text-zinc-300 hover:bg-zinc-800"
        >
          Huỷ
        </button>
        <button
          type="button"
          onclick={handleConfirmDelete}
          class="rounded-xl bg-rose-600 px-4 py-2 text-xs font-semibold text-white hover:bg-rose-500"
        >
          Xoá vĩnh viễn
        </button>
      </div>
    </div>
  </div>
{/if}
