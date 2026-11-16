<script lang="ts">
  import { X } from "@lucide/svelte";
  import type { NoteType } from "$lib/shared/types";
  import { notesApi } from "../api";

  interface Props {
    open: boolean;
    deckId: string;
    deckName: string;
    onClose: () => void;
    onSaved: () => void;
  }

  let { open, deckId, deckName, onClose, onSaved }: Props = $props();

  let noteType = $state<NoteType>("vocab");
  let word = $state("");
  let ipa = $state("");
  let meaning = $state("");
  let exampleSentence = $state("");
  let collocationsText = $state("");
  let saving = $state(false);
  let error = $state("");

  function resetForm() {
    word = "";
    ipa = "";
    meaning = "";
    exampleSentence = "";
    collocationsText = "";
    error = "";
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!word.trim() || !meaning.trim()) {
      error = "Vui lòng nhập từ vựng và nghĩa.";
      return;
    }

    saving = true;
    error = "";
    try {
      const collocations = collocationsText
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);

      await notesApi.create({
        deckId,
        noteType,
        word: word.trim(),
        ipa: ipa.trim() || undefined,
        meaning: meaning.trim(),
        exampleSentence: exampleSentence.trim() || undefined,
        collocations,
      });
      resetForm();
      onSaved();
    } catch (err) {
      error = String(err);
    } finally {
      saving = false;
    }
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-xs">
    <div class="w-full max-w-lg rounded-2xl border border-zinc-800 bg-zinc-900 p-6 shadow-2xl">
      <div class="mb-4 flex items-center justify-between">
        <div>
          <h2 class="text-lg font-semibold text-zinc-100">Thêm thẻ mới</h2>
          <p class="text-xs text-zinc-400">Bộ thẻ: {deckName}</p>
        </div>
        <button
          type="button"
          onclick={onClose}
          class="rounded-lg p-1.5 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200"
        >
          <X class="h-4 w-4" />
        </button>
      </div>

      <form onsubmit={handleSubmit} class="space-y-3.5">
        <div class="flex gap-2 rounded-xl bg-zinc-950 p-1">
          <button
            type="button"
            onclick={() => (noteType = "vocab")}
            class="flex-1 rounded-lg py-1.5 text-xs font-medium transition {noteType === 'vocab'
              ? 'bg-indigo-600 text-white'
              : 'text-zinc-400 hover:text-zinc-200'}"
          >
            Từ vựng (Vocabulary)
          </button>
          <button
            type="button"
            onclick={() => (noteType = "cloze")}
            class="flex-1 rounded-lg py-1.5 text-xs font-medium transition {noteType === 'cloze'
              ? 'bg-indigo-600 text-white'
              : 'text-zinc-400 hover:text-zinc-200'}"
          >
            Điền khuyết (Cloze)
          </button>
        </div>

        <div class="grid grid-cols-3 gap-3">
          <div class="col-span-2">
            <label class="mb-1 block text-xs font-medium text-zinc-300" for="note-word">
              {noteType === "cloze" ? "Từ cần điền" : "Từ / Cụm từ"}
            </label>
            <input
              id="note-word"
              type="text"
              bind:value={word}
              placeholder="e.g. ubiquitous"
              class="w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
            />
          </div>
          <div>
            <label class="mb-1 block text-xs font-medium text-zinc-300" for="note-ipa">IPA</label>
            <input
              id="note-ipa"
              type="text"
              bind:value={ipa}
              placeholder="/juːˈbɪk.wɪ.təs/"
              class="w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
            />
          </div>
        </div>

        <div>
          <label class="mb-1 block text-xs font-medium text-zinc-300" for="note-meaning">
            Nghĩa
          </label>
          <input
            id="note-meaning"
            type="text"
            bind:value={meaning}
            placeholder="có mặt ở khắp mọi nơi"
            class="w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
          />
        </div>

        <div>
          <label class="mb-1 block text-xs font-medium text-zinc-300" for="note-example">
            Câu ví dụ {noteType === "cloze" ? "(dùng từ cần điền trong câu)" : "(tuỳ chọn)"}
          </label>
          <textarea
            id="note-example"
            rows="2"
            bind:value={exampleSentence}
            placeholder="Smartphones have become ubiquitous in modern life."
            class="w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
          ></textarea>
        </div>

        <div>
          <label class="mb-1 block text-xs font-medium text-zinc-300" for="note-colloc">
            Collocations (cách nhau bằng dấu phẩy)
          </label>
          <input
            id="note-colloc"
            type="text"
            bind:value={collocationsText}
            placeholder="become ubiquitous, ubiquitous presence"
            class="w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
          />
        </div>

        {#if error}
          <p class="text-xs text-rose-400">{error}</p>
        {/if}

        <div class="flex justify-end gap-2 pt-2">
          <button
            type="button"
            onclick={onClose}
            class="rounded-xl border border-zinc-700 px-4 py-2 text-sm font-medium text-zinc-300 hover:bg-zinc-800"
          >
            Đóng
          </button>
          <button
            type="submit"
            disabled={saving}
            class="rounded-xl bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-500 disabled:opacity-50"
          >
            {saving ? "Đang lưu..." : "Lưu & Thêm tiếp"}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
