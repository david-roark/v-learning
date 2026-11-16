<script lang="ts">
  import { Check, RotateCcw, Volume2 } from "@lucide/svelte";
  import { computeDictationDiff, type DictationResult } from "./diff";

  interface Props {
    targetText: string;
    onPlay: () => void;
    onChecked: () => void;
  }

  let { targetText, onPlay, onChecked }: Props = $props();

  let userInput = $state("");
  let result = $state<DictationResult | null>(null);

  function handleCheck(e: Event) {
    e.preventDefault();
    result = computeDictationDiff(targetText, userInput);
    onChecked();
  }

  function handleReset() {
    userInput = "";
    result = null;
  }
</script>

<div class="rounded-3xl border border-zinc-800 bg-zinc-900/90 p-8 shadow-xl">
  <div class="mb-5 flex items-center justify-between">
    <span
      class="rounded-full bg-indigo-500/15 px-3 py-1 text-[11px] font-medium text-indigo-300 uppercase"
    >
      Dictation Mode
    </span>
    <button
      type="button"
      onclick={onPlay}
      class="inline-flex items-center gap-2 rounded-xl border border-zinc-700 bg-zinc-800 px-3.5 py-2 text-xs font-medium text-zinc-200 hover:border-indigo-500 hover:text-indigo-300"
    >
      <Volume2 class="h-4 w-4" />
      Nghe lại (R)
    </button>
  </div>

  <form onsubmit={handleCheck} class="space-y-4">
    <input
      type="text"
      bind:value={userInput}
      placeholder="Gõ lại những gì bạn nghe được..."
      class="w-full rounded-2xl border border-zinc-700 bg-zinc-950 px-4 py-3.5 text-base text-white placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
    />

    <div class="flex items-center justify-end gap-2">
      {#if result}
        <button
          type="button"
          onclick={handleReset}
          class="inline-flex items-center gap-1.5 rounded-xl border border-zinc-700 px-3.5 py-2 text-xs font-medium text-zinc-300 hover:bg-zinc-800"
        >
          <RotateCcw class="h-3.5 w-3.5" />
          Thử lại
        </button>
      {/if}
      <button
        type="submit"
        class="inline-flex items-center gap-1.5 rounded-xl bg-indigo-600 px-4 py-2 text-xs font-semibold text-white hover:bg-indigo-500"
      >
        <Check class="h-3.5 w-3.5" />
        Kiểm tra
      </button>
    </div>
  </form>

  {#if result}
    <div class="mt-6 border-t border-zinc-800 pt-5">
      <div class="mb-3 flex items-center justify-between text-xs">
        <span class="text-zinc-400">Kết quả đối chiếu:</span>
        <span class="font-semibold {result.isExact ? 'text-emerald-400' : 'text-amber-400'}">
          Độ chính xác: {result.accuracy}%
        </span>
      </div>

      <div class="flex flex-wrap gap-2 rounded-2xl bg-zinc-950 p-4 text-sm">
        {#each result.tokens as tok}
          {#if tok.status === "correct"}
            <span class="rounded-lg bg-emerald-500/15 px-2 py-1 text-emerald-300">{tok.text}</span>
          {:else if tok.status === "wrong"}
            <span class="rounded-lg bg-rose-500/15 px-2 py-1 text-rose-300">
              <s class="opacity-70">{tok.text}</s> → <strong>{tok.expected}</strong>
            </span>
          {:else if tok.status === "missing"}
            <span class="rounded-lg bg-amber-500/15 px-2 py-1 text-amber-300">+{tok.text}</span>
          {:else}
            <span class="rounded-lg bg-zinc-800 px-2 py-1 text-zinc-400 line-through"
              >{tok.text}</span
            >
          {/if}
        {/each}
      </div>
    </div>
  {/if}
</div>
