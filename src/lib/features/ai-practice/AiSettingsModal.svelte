<script lang="ts">
  import { CheckCircle2, RefreshCw, X } from "@lucide/svelte";
  import Select from "$lib/shared/components/Select.svelte";
  import { aiApi } from "./api";

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  type ProviderId = "gemini" | "openai" | "ollama" | "custom";

  const PRESETS: Record<Exclude<ProviderId, "custom">, { label: string; baseUrl: string; defaultModel: string }> = {
    gemini: {
      label: "Google Gemini",
      baseUrl: "https://generativelanguage.googleapis.com/v1beta/openai",
      defaultModel: "gemini-2.5-flash",
    },
    openai: {
      label: "OpenAI",
      baseUrl: "https://api.openai.com/v1",
      defaultModel: "gpt-4o-mini",
    },
    ollama: {
      label: "Ollama (Local)",
      baseUrl: "http://localhost:11434/v1",
      defaultModel: "llama3.2",
    },
  };

  let { open, onClose }: Props = $props();

  let activeProvider = $state<ProviderId>("gemini");
  let baseUrl = $state(PRESETS.gemini.baseUrl);
  let apiKey = $state("");
  let model = $state(PRESETS.gemini.defaultModel);

  let savedKeys = $state<Record<ProviderId, string>>({
    gemini: "",
    openai: "",
    ollama: "",
    custom: "",
  });

  let availableModels = $state<string[]>([]);
  let testing = $state(false);
  let testMsg = $state("");
  let testError = $state("");
  let saving = $state(false);
  let saved = $state(false);

  function detectProvider(url: string): ProviderId {
    const clean = url.trim().replace(/\/+$/, "");
    if (clean === PRESETS.gemini.baseUrl) return "gemini";
    if (clean === PRESETS.openai.baseUrl) return "openai";
    if (clean === PRESETS.ollama.baseUrl) return "ollama";
    return "custom";
  }

  function selectProvider(next: ProviderId) {
    savedKeys[activeProvider] = apiKey;
    activeProvider = next;
    availableModels = [];
    testMsg = "";
    testError = "";

    if (next !== "custom") {
      baseUrl = PRESETS[next].baseUrl;
      model = PRESETS[next].defaultModel;
    }
    apiKey = savedKeys[next] ?? "";
  }

  async function loadSavedSettings() {
    const s = await aiApi.getSettings();
    const detected = detectProvider(s.baseUrl);
    activeProvider = detected;
    baseUrl = s.baseUrl;
    apiKey = s.apiKey;
    model = s.model;
    savedKeys[detected] = s.apiKey;
    availableModels = [];
    testMsg = "";
    testError = "";
    saved = false;
  }

  $effect(() => {
    if (open) {
      void loadSavedSettings();
    }
  });

  async function handleTestAndFetchModels() {
    if (testing) return;
    testing = true;
    testMsg = "";
    testError = "";
    try {
      const models = await aiApi.listModels(baseUrl, apiKey);
      availableModels = models;
      if (models.length > 0 && !models.includes(model)) {
        model = models[0];
      }
      testMsg = `Kết nối thành công! Tìm thấy ${models.length} model.`;
    } catch (err) {
      testError = String(err);
    } finally {
      testing = false;
    }
  }

  async function handleSave(e: Event) {
    e.preventDefault();
    saving = true;
    saved = false;
    try {
      await aiApi.saveSettings({ baseUrl, apiKey, model });
      saved = true;
      setTimeout(() => onClose(), 500);
    } finally {
      saving = false;
    }
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-xs">
    <div class="w-full max-w-md rounded-2xl border border-zinc-800 bg-zinc-900 p-6 shadow-2xl">
      <div class="mb-4 flex items-center justify-between">
        <h2 class="text-base font-semibold text-white">Cấu hình AI Gia sư</h2>
        <button
          type="button"
          onclick={onClose}
          class="rounded-lg p-1.5 text-zinc-400 hover:bg-zinc-800"
        >
          <X class="h-4 w-4" />
        </button>
      </div>

      <div class="mb-4 grid grid-cols-4 gap-1.5 rounded-xl bg-zinc-950 p-1">
        {#each (["gemini", "openai", "ollama", "custom"] as const) as pid}
          <button
            type="button"
            onclick={() => selectProvider(pid)}
            class="rounded-lg px-2 py-1.5 text-xs font-medium transition {activeProvider === pid
              ? 'bg-indigo-600 text-white'
              : 'text-zinc-400 hover:text-zinc-200'}"
          >
            {pid === "custom" ? "Tuỳ chỉnh" : pid === "gemini" ? "Gemini" : pid === "openai" ? "OpenAI" : "Ollama"}
          </button>
        {/each}
      </div>

      <form onsubmit={handleSave} class="space-y-3.5">
        <div>
          <label class="mb-1 block text-xs font-medium text-zinc-300" for="ai-url">Base URL</label>
          <input
            id="ai-url"
            type="text"
            bind:value={baseUrl}
            oninput={() => (activeProvider = detectProvider(baseUrl))}
            class="w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2 text-xs text-zinc-100 focus:border-indigo-500 focus:outline-none"
          />
        </div>

        <div>
          <label class="mb-1 block text-xs font-medium text-zinc-300" for="ai-key">
            API Key {activeProvider === "ollama" ? "(để trống cho Ollama)" : ""}
          </label>
          <div class="flex gap-2">
            <input
              id="ai-key"
              type="password"
              bind:value={apiKey}
              placeholder={activeProvider === "gemini" ? "AIza..." : "sk-..."}
              class="flex-1 rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2 text-xs text-zinc-100 focus:border-indigo-500 focus:outline-none"
            />
            <button
              type="button"
              onclick={handleTestAndFetchModels}
              disabled={testing}
              class="inline-flex shrink-0 items-center gap-1.5 rounded-xl border border-indigo-500/40 bg-indigo-500/15 px-3 py-2 text-xs font-medium text-indigo-300 hover:bg-indigo-500/25 disabled:opacity-50"
            >
              <RefreshCw class="h-3.5 w-3.5 {testing ? 'animate-spin' : ''}" />
              {testing ? "Đang test..." : "Test & Lấy Model"}
            </button>
          </div>
        </div>

        <div>
          <label class="mb-1 block text-xs font-medium text-zinc-300" for="ai-model">Model</label>
          {#if availableModels.length > 0}
            <Select
              options={availableModels.map((m) => ({ value: m, label: m }))}
              bind:value={model}
              placeholder="Chọn model..."
            />
          {:else}
            <input
              id="ai-model"
              type="text"
              bind:value={model}
              class="w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2 text-xs text-zinc-100 focus:border-indigo-500 focus:outline-none"
            />
          {/if}
        </div>

        {#if testMsg}
          <p class="flex items-center gap-1.5 text-xs text-emerald-400">
            <CheckCircle2 class="h-3.5 w-3.5 shrink-0" />
            {testMsg}
          </p>
        {/if}
        {#if testError}
          <p class="rounded-xl bg-rose-500/10 p-2.5 text-xs text-rose-400">{testError}</p>
        {/if}
        {#if saved}
          <p class="text-xs text-emerald-400">Đã lưu cấu hình AI!</p>
        {/if}

        <div class="flex justify-end gap-2 pt-2">
          <button
            type="button"
            onclick={onClose}
            class="rounded-xl border border-zinc-700 px-4 py-2 text-xs font-medium text-zinc-300 hover:bg-zinc-800"
          >
            Đóng
          </button>
          <button
            type="submit"
            disabled={saving}
            class="rounded-xl bg-indigo-600 px-4 py-2 text-xs font-semibold text-white hover:bg-indigo-500 disabled:opacity-50"
          >
            {saving ? "Đang lưu..." : "Lưu cấu hình"}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
