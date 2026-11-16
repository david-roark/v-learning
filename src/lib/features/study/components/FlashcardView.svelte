<script lang="ts">
  import { Volume2 } from "@lucide/svelte";
  import type { Note } from "$lib/shared/types";

  interface Props {
    note: Note;
    revealed: boolean;
    onSpeak: (text: string) => void;
  }

  let { note, revealed, onSpeak }: Props = $props();

  const clozePrompt = $derived.by(() => {
    if (note.noteType !== "cloze" || !note.exampleSentence) {
      return note.exampleSentence ?? "";
    }
    const hint = note.word.length > 1 ? `${note.word[0]}______` : "______";
    const escaped = note.word.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    return note.exampleSentence.replace(new RegExp(escaped, "gi"), hint);
  });
</script>

<div
  class="flex min-h-[320px] flex-col justify-between rounded-3xl border border-zinc-800 bg-zinc-900/90 p-8 shadow-xl"
>
  <!-- Front -->
  <div class="flex flex-col items-center text-center">
    <span
      class="mb-3 rounded-full bg-zinc-800 px-3 py-1 text-[11px] font-medium tracking-wide text-zinc-400 uppercase"
    >
      {note.noteType === "cloze" ? "Cloze" : "Vocabulary"}
    </span>

    {#if note.noteType === "cloze" && !revealed}
      <p class="mt-2 max-w-xl text-xl leading-relaxed font-medium text-zinc-100">
        {clozePrompt || "______"}
      </p>
      <p class="mt-3 text-sm text-zinc-400">Gợi ý nghĩa: {note.meaning}</p>
    {:else}
      <div class="flex items-center gap-3">
        <h2 class="text-3xl font-bold tracking-tight text-white">{note.word}</h2>
        <button
          type="button"
          onclick={() => onSpeak(note.word)}
          title="Phát âm (R)"
          class="rounded-xl border border-zinc-700 bg-zinc-800/80 p-2 text-zinc-300 transition hover:border-indigo-500 hover:text-indigo-400"
        >
          <Volume2 class="h-4 w-4" />
        </button>
      </div>

      {#if note.ipa}
        <p class="mt-1.5 font-mono text-sm text-indigo-400">{note.ipa}</p>
      {/if}
    {/if}
  </div>

  <!-- Back -->
  {#if revealed}
    <div class="mt-6 border-t border-zinc-800 pt-6 text-center">
      <p class="text-xl font-semibold text-emerald-400">{note.meaning}</p>

      {#if note.exampleSentence}
        <div class="mt-4 flex items-center justify-center gap-2">
          <p class="max-w-xl text-sm leading-relaxed text-zinc-300 italic">
            "{note.exampleSentence}"
          </p>
          <button
            type="button"
            onclick={() => onSpeak(note.exampleSentence ?? "")}
            title="Nghe câu ví dụ"
            class="rounded-lg p-1.5 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200"
          >
            <Volume2 class="h-3.5 w-3.5" />
          </button>
        </div>
      {/if}

      {#if note.collocations.length > 0}
        <div class="mt-4 flex flex-wrap justify-center gap-1.5">
          {#each note.collocations as coll}
            <span class="rounded-lg bg-zinc-800 px-2.5 py-1 text-xs text-zinc-300">
              {coll}
            </span>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>
