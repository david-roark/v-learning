<script lang="ts">
  import { RotateCcw, X } from "@lucide/svelte";
  import Select from "$lib/shared/components/Select.svelte";
  import {
    appearance,
    GOOGLE_FONT_OPTIONS,
    IPA_FONT_OPTIONS,
    resetAppearance,
    THEME_OPTIONS,
  } from "./appearance.svelte";

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-xs">
    <div class="w-full max-w-lg rounded-2xl border border-zinc-800 bg-zinc-900 p-6 shadow-2xl">
      <div class="mb-5 flex items-center justify-between">
        <h2 class="text-base font-semibold text-white">Giao diện, Font chữ & Cỡ chữ</h2>
        <button
          type="button"
          onclick={onClose}
          class="rounded-lg p-1.5 text-zinc-400 hover:bg-zinc-800"
        >
          <X class="h-4 w-4" />
        </button>
      </div>

      <div class="space-y-5">
        <!-- Theme Picker -->
        <div>
          <span class="mb-2 block text-xs font-medium text-zinc-300">Chủ đề (Theme)</span>
          <div class="grid grid-cols-2 gap-2.5 sm:grid-cols-4">
            {#each THEME_OPTIONS as t (t.id)}
              {@const active = appearance.theme === t.id}
              <button
                type="button"
                onclick={() => (appearance.theme = t.id)}
                class="flex flex-col items-start gap-2 rounded-xl border p-2.5 text-left transition {active
                  ? 'border-indigo-500 ring-1 ring-indigo-500'
                  : 'border-zinc-700 hover:border-zinc-500'}"
                style={`background-color: ${t.bg};`}
              >
                <div class="flex w-full items-center gap-1.5">
                  <span
                    class="h-3.5 w-3.5 rounded-full"
                    style={`background-color: ${t.card}; border: 1px solid ${t.accent};`}
                  ></span>
                  <span class="h-2 flex-1 rounded-full" style={`background-color: ${t.accent};`}
                  ></span>
                </div>
                <span
                  class="text-[11px] font-medium"
                  style={`color: ${t.isLight ? "#1e293b" : "#f4f4f5"}`}
                >
                  {t.name}
                </span>
              </button>
            {/each}
          </div>
        </div>

        <!-- Font Family -->
        <div class="grid gap-3 sm:grid-cols-2">
          <div>
            <span class="mb-1.5 block text-xs font-medium text-zinc-300"
              >Font chữ chính (Google Fonts)</span
            >
            <Select options={GOOGLE_FONT_OPTIONS} bind:value={appearance.fontFamily} />
          </div>
          <div>
            <span class="mb-1.5 block text-xs font-medium text-zinc-300">Font phiên âm (IPA)</span>
            <Select options={IPA_FONT_OPTIONS} bind:value={appearance.ipaFontFamily} />
          </div>
        </div>

        <!-- Font Size -->
        <div>
          <div class="mb-1.5 flex items-center justify-between text-xs">
            <span class="font-medium text-zinc-300">Cỡ chữ toàn app</span>
            <span class="font-mono font-semibold text-indigo-400">{appearance.fontSize}px</span>
          </div>
          <div class="flex items-center gap-3">
            <input
              type="range"
              min="13"
              max="22"
              step="1"
              bind:value={appearance.fontSize}
              class="flex-1 accent-indigo-500"
            />
            <div class="flex gap-1">
              {#each [14, 16, 18, 20] as sz}
                <button
                  type="button"
                  onclick={() => (appearance.fontSize = sz)}
                  class="rounded-lg border px-2 py-1 text-[11px] {appearance.fontSize === sz
                    ? 'border-indigo-500 bg-indigo-500/20 text-indigo-300'
                    : 'border-zinc-700 text-zinc-400 hover:bg-zinc-800'}"
                >
                  {sz}
                </button>
              {/each}
            </div>
          </div>
        </div>

        <!-- Live Preview -->
        <div class="rounded-xl border border-zinc-800 bg-zinc-950 p-4 text-center">
          <p class="text-lg font-bold text-white">ubiquitous</p>
          <p class="mt-0.5 font-mono text-xs text-indigo-400">/juːˈbɪk.wɪ.təs/ · [ˌʌbɪkˈwɪtɪ]</p>
          <p class="mt-1 text-xs text-emerald-400">có mặt ở khắp mọi nơi (Tiếng Việt & English)</p>
        </div>

        <div class="flex items-center justify-between pt-1">
          <button
            type="button"
            onclick={resetAppearance}
            class="inline-flex items-center gap-1.5 text-xs text-zinc-400 hover:text-zinc-200"
          >
            <RotateCcw class="h-3.5 w-3.5" />
            Mặc định
          </button>
          <button
            type="button"
            onclick={onClose}
            class="rounded-xl bg-indigo-600 px-4 py-2 text-xs font-semibold text-white hover:bg-indigo-500"
          >
            Xong
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
