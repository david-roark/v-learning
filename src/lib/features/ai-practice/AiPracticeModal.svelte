<script lang="ts">
  import { Send, Sparkles, X } from "@lucide/svelte";
  import { aiApi } from "./api";

  interface Props {
    open: boolean;
    targetWords: string[];
    onClose: () => void;
  }

  let { open, targetWords, onClose }: Props = $props();

  let userText = $state("");
  let feedback = $state("");
  let loading = $state(false);
  let error = $state("");

  async function handleEvaluate(e: Event) {
    e.preventDefault();
    if (!userText.trim() || loading) return;
    loading = true;
    error = "";
    feedback = "";
    try {
      const res = await aiApi.evaluatePractice(targetWords, userText.trim());
      feedback = res.feedback;
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-xs">
    <div class="w-full max-w-xl rounded-2xl border border-zinc-800 bg-zinc-900 p-6 shadow-2xl">
      <div class="mb-4 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <Sparkles class="h-4 w-4 text-indigo-400" />
          <h2 class="text-base font-semibold text-white">Luyện đặt câu với AI</h2>
        </div>
        <button
          type="button"
          onclick={onClose}
          class="rounded-lg p-1.5 text-zinc-400 hover:bg-zinc-800"
        >
          <X class="h-4 w-4" />
        </button>
      </div>

      <div class="mb-4">
        <p class="text-xs text-zinc-400">Hãy viết 1–2 câu tiếng Anh sử dụng các từ vừa ôn:</p>
        <div class="mt-2 flex flex-wrap gap-1.5">
          {#each targetWords as w (w)}
            <span
              class="rounded-lg bg-indigo-500/15 px-2.5 py-1 text-xs font-medium text-indigo-300"
            >
              {w}
            </span>
          {/each}
        </div>
      </div>

      <form onsubmit={handleEvaluate} class="space-y-3">
        <textarea
          rows="3"
          bind:value={userText}
          placeholder="Ví dụ: Despite the inevitable challenges, she remained pragmatic and resilient."
          class="w-full rounded-xl border border-zinc-700 bg-zinc-950 p-3 text-sm text-zinc-100 placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
        ></textarea>

        <div class="flex justify-end">
          <button
            type="submit"
            disabled={loading || !userText.trim()}
            class="inline-flex items-center gap-1.5 rounded-xl bg-indigo-600 px-4 py-2 text-xs font-semibold text-white hover:bg-indigo-500 disabled:opacity-40"
          >
            <Send class="h-3.5 w-3.5" />
            {loading ? "AI đang chấm..." : "Chấm câu"}
          </button>
        </div>
      </form>

      {#if error}
        <p class="mt-3 rounded-xl bg-rose-500/10 p-3 text-xs text-rose-400">{error}</p>
      {/if}

      {#if feedback}
        <div
          class="mt-4 max-h-60 overflow-y-auto rounded-xl border border-zinc-800 bg-zinc-950 p-4 text-xs leading-relaxed whitespace-pre-wrap text-zinc-200"
        >
          {feedback}
        </div>
      {/if}
    </div>
  </div>
{/if}
