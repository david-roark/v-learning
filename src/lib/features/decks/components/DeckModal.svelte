<script lang="ts">
  import { X } from "@lucide/svelte";
  import { decksApi } from "../api";

  interface Props {
    open: boolean;
    onClose: () => void;
    onCreated: () => void;
  }

  let { open, onClose, onCreated }: Props = $props();

  let name = $state("");
  let description = $state("");
  let newCardsPerDay = $state(20);
  let desiredRetention = $state(90);
  let saving = $state(false);
  let error = $state("");

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!name.trim()) {
      error = "Vui lòng nhập tên bộ thẻ.";
      return;
    }

    saving = true;
    error = "";
    try {
      await decksApi.create({
        name: name.trim(),
        description: description.trim() || undefined,
        newCardsPerDay,
        desiredRetention: desiredRetention / 100,
      });
      name = "";
      description = "";
      onCreated();
      onClose();
    } catch (err) {
      error = String(err);
    } finally {
      saving = false;
    }
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-xs">
    <div class="w-full max-w-md rounded-2xl border border-zinc-800 bg-zinc-900 p-6 shadow-2xl">
      <div class="mb-5 flex items-center justify-between">
        <h2 class="text-lg font-semibold text-zinc-100">Tạo bộ thẻ mới</h2>
        <button
          type="button"
          onclick={onClose}
          class="rounded-lg p-1.5 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200"
        >
          <X class="h-4 w-4" />
        </button>
      </div>

      <form onsubmit={handleSubmit} class="space-y-4">
        <div>
          <label class="mb-1.5 block text-sm font-medium text-zinc-300" for="deck-name">
            Tên bộ thẻ
          </label>
          <input
            id="deck-name"
            type="text"
            bind:value={name}
            placeholder="Ví dụ: IELTS Core Vocabulary"
            class="w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3.5 py-2.5 text-sm text-zinc-100 placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
          />
        </div>

        <div>
          <label class="mb-1.5 block text-sm font-medium text-zinc-300" for="deck-desc">
            Mô tả (tuỳ chọn)
          </label>
          <input
            id="deck-desc"
            type="text"
            bind:value={description}
            placeholder="Từ vựng học thuật, collocations..."
            class="w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3.5 py-2.5 text-sm text-zinc-100 placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
          />
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="mb-1.5 block text-xs font-medium text-zinc-400" for="deck-new-limit">
              Thẻ mới / ngày
            </label>
            <input
              id="deck-new-limit"
              type="number"
              min="1"
              max="500"
              bind:value={newCardsPerDay}
              class="w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 focus:border-indigo-500 focus:outline-none"
            />
          </div>
          <div>
            <label class="mb-1.5 block text-xs font-medium text-zinc-400" for="deck-retention">
              Mục tiêu nhớ (%)
            </label>
            <input
              id="deck-retention"
              type="number"
              min="70"
              max="99"
              bind:value={desiredRetention}
              class="w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 focus:border-indigo-500 focus:outline-none"
            />
          </div>
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
            Huỷ
          </button>
          <button
            type="submit"
            disabled={saving}
            class="rounded-xl bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-500 disabled:opacity-50"
          >
            {saving ? "Đang tạo..." : "Tạo bộ thẻ"}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
