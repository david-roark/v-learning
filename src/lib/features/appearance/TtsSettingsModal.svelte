<script lang="ts">
  import { ArrowDown, ArrowUp, Volume2, X } from "@lucide/svelte";
  import { playAudioBytes, playWordAudio, speakSystemTts, ttsApi } from "$lib/shared/audio";
  import Select, { type SelectOption } from "$lib/shared/components/Select.svelte";
  import { toast } from "$lib/shared/toast.svelte";

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  interface ProviderMeta {
    id: string;
    label: string;
    desc: string;
    sampleText: string;
    voices: SelectOption[];
    defaultVoice: string;
  }

  const ALL_PROVIDERS: ProviderMeta[] = [
    {
      id: "google_tts",
      label: "Google Translate TTS",
      desc: "Ổn định, hỗ trợ từ đơn, cụm từ và câu dài.",
      sampleText: "Ubiquitous learners stay resilient.",
      voices: [
        { value: "en-US", label: "Mỹ (en-US)" },
        { value: "en-GB", label: "Anh (en-GB)" },
        { value: "en-AU", label: "Úc (en-AU)" },
        { value: "en-IN", label: "Ấn Độ (en-IN)" },
      ],
      defaultVoice: "en-US",
    },
    {
      id: "youdao_tts",
      label: "Youdao Voice (US / UK)",
      desc: "Giọng đọc rõ ràng, hỗ trợ tốt từ đơn và cụm từ.",
      sampleText: "give up",
      voices: [
        { value: "us", label: "Anh - Mỹ (US)" },
        { value: "uk", label: "Anh - Anh (UK)" },
      ],
      defaultVoice: "us",
    },
    {
      id: "system_tts",
      label: "Giọng máy (Offline / Hệ điều hành)",
      desc: "Dùng giọng có sẵn của OS (Web Speech API / spd-say / espeak-ng).",
      sampleText: "Hello, stay resilient.",
      voices: [],
      defaultVoice: "",
    },
  ];

  let orderedItems = $state<Array<{ id: string; enabled: boolean }>>([]);
  let providerVoices = $state<Record<string, string>>({
    google_tts: "en-US",
    youdao_tts: "us",
  });
  let saving = $state(false);
  let testingId = $state<string | null>(null);

  $effect(() => {
    if (open) {
      void ttsApi.getSettings().then((s) => {
        const nextVoices: Record<string, string> = {};
        for (const p of ALL_PROVIDERS) {
          if (p.voices.length > 0) {
            const saved = s.providerVoices?.[p.id] ?? (p.id === "google_tts" ? s.voice : "");
            nextVoices[p.id] = p.voices.some((v) => v.value === saved) ? saved : p.defaultVoice;
          }
        }
        providerVoices = nextVoices;

        const enabledSet = new Set(s.fallbackOrder);
        const list: Array<{ id: string; enabled: boolean }> = [];
        for (const id of s.fallbackOrder) {
          if (ALL_PROVIDERS.some((p) => p.id === id)) {
            list.push({ id, enabled: true });
          }
        }
        for (const p of ALL_PROVIDERS) {
          if (!enabledSet.has(p.id)) {
            list.push({ id: p.id, enabled: false });
          }
        }
        orderedItems = list;
      });
    }
  });

  function moveItem(idx: number, delta: -1 | 1) {
    const next = idx + delta;
    if (next < 0 || next >= orderedItems.length) return;
    const copy = [...orderedItems];
    const [item] = copy.splice(idx, 1);
    copy.splice(next, 0, item);
    orderedItems = copy;
  }

  function toggleItem(idx: number) {
    orderedItems = orderedItems.map((item, i) =>
      i === idx ? { ...item, enabled: !item.enabled } : item,
    );
  }

  async function saveCurrent(): Promise<boolean> {
    const fallbackOrder = orderedItems.filter((i) => i.enabled).map((i) => i.id);
    if (fallbackOrder.length === 0) {
      toast.error("Cần bật ít nhất 1 nguồn TTS.");
      return false;
    }
    saving = true;
    try {
      await ttsApi.saveSettings({
        fallbackOrder,
        voice: providerVoices.google_tts ?? "en-US",
        providerVoices,
      });
      return true;
    } catch (err) {
      toast.error(err);
      return false;
    } finally {
      saving = false;
    }
  }

  async function handleSave(e: Event) {
    e.preventDefault();
    if (await saveCurrent()) {
      toast.success("Đã lưu cấu hình phát âm (TTS)!");
    }
  }

  async function handlePreviewProvider(meta: ProviderMeta) {
    testingId = meta.id;
    try {
      if (meta.id === "system_tts") {
        await speakSystemTts(meta.sampleText);
        return;
      }
      const voice = providerVoices[meta.id] ?? meta.defaultVoice;
      const bytes = await ttsApi.previewProvider(meta.id, voice, meta.sampleText);
      await playAudioBytes(bytes);
    } catch (err) {
      toast.error(err);
    } finally {
      testingId = null;
    }
  }

  async function handleTestFallback() {
    if (!(await saveCurrent())) return;
    testingId = "fallback";
    try {
      await playWordAudio("Ubiquitous learners stay resilient.");
    } finally {
      testingId = null;
    }
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-sm">
    <div class="w-full max-w-xl rounded-2xl border border-zinc-800 bg-zinc-900 p-6 shadow-2xl">
      <div class="mb-4 flex items-center justify-between">
        <h3 class="text-base font-semibold text-white">Cấu hình phát âm (TTS Online)</h3>
        <button
          type="button"
          onclick={onClose}
          class="rounded-lg p-1 text-zinc-400 hover:bg-zinc-800 hover:text-white"
        >
          <X class="h-4 w-4" />
        </button>
      </div>

      <form onsubmit={handleSave} class="space-y-4 text-xs">
        <div>
          <p class="mb-2 font-medium text-zinc-300">
            Thứ tự ưu tiên, chọn giọng & nghe thử từng nguồn
          </p>
          <div class="space-y-2.5">
            {#each orderedItems as item, idx (item.id)}
              {@const meta = ALL_PROVIDERS.find((p) => p.id === item.id)!}
              <div class="space-y-2 rounded-xl border border-zinc-800 bg-zinc-950/70 px-3.5 py-3">
                <div class="flex items-start justify-between gap-3">
                  <label class="flex flex-1 cursor-pointer items-start gap-2.5">
                    <input
                      type="checkbox"
                      checked={item.enabled}
                      onchange={() => toggleItem(idx)}
                      class="mt-0.5 rounded border-zinc-600 bg-zinc-900 text-indigo-500"
                    />
                    <div>
                      <p class="font-medium text-zinc-100">
                        {idx + 1}. {meta.label}
                      </p>
                      <p class="text-[11px] text-zinc-400">{meta.desc}</p>
                    </div>
                  </label>

                  <div class="flex shrink-0 items-center gap-1">
                    <button
                      type="button"
                      disabled={testingId !== null}
                      onclick={() => handlePreviewProvider(meta)}
                      title="Nghe thử riêng nguồn này"
                      class="inline-flex items-center gap-1 rounded-lg border border-zinc-700 px-2 py-1 text-[11px] text-zinc-200 hover:border-indigo-500 hover:text-indigo-300 disabled:opacity-40"
                    >
                      <Volume2 class="h-3 w-3 text-indigo-400" />
                      {testingId === meta.id ? "Đang tải..." : "Nghe thử"}
                    </button>
                    <button
                      type="button"
                      disabled={idx === 0}
                      onclick={() => moveItem(idx, -1)}
                      title="Đưa lên ưu tiên cao hơn"
                      class="rounded-lg border border-zinc-800 p-1 text-zinc-400 hover:bg-zinc-800 hover:text-white disabled:opacity-30"
                    >
                      <ArrowUp class="h-3.5 w-3.5" />
                    </button>
                    <button
                      type="button"
                      disabled={idx === orderedItems.length - 1}
                      onclick={() => moveItem(idx, 1)}
                      title="Đưa xuống ưu tiên thấp hơn"
                      class="rounded-lg border border-zinc-800 p-1 text-zinc-400 hover:bg-zinc-800 hover:text-white disabled:opacity-30"
                    >
                      <ArrowDown class="h-3.5 w-3.5" />
                    </button>
                  </div>
                </div>

                {#if meta.voices.length > 0}
                  <div class="flex items-center gap-2 pl-6">
                    <span class="shrink-0 text-[11px] text-zinc-400">Giọng đọc:</span>
                    <Select
                      options={meta.voices}
                      bind:value={providerVoices[meta.id]}
                      class="w-52"
                    />
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        </div>

        <div class="flex items-center justify-between gap-2 pt-2">
          <button
            type="button"
            disabled={testingId !== null}
            onclick={handleTestFallback}
            class="inline-flex items-center gap-1.5 rounded-xl border border-zinc-700 px-3.5 py-2 font-medium text-zinc-200 hover:bg-zinc-800 disabled:opacity-50"
          >
            <Volume2 class="h-3.5 w-3.5 text-indigo-400" />
            {testingId === "fallback" ? "Đang phát..." : "Nghe thử theo Fallback"}
          </button>

          <div class="flex items-center gap-2">
            <button
              type="button"
              onclick={onClose}
              class="rounded-xl px-4 py-2 text-zinc-400 hover:bg-zinc-800 hover:text-white"
            >
              Đóng
            </button>
            <button
              type="submit"
              disabled={saving}
              class="rounded-xl bg-indigo-600 px-4 py-2 font-semibold text-white hover:bg-indigo-500 disabled:opacity-50"
            >
              {saving ? "Đang lưu..." : "Lưu cấu hình"}
            </button>
          </div>
        </div>
      </form>
    </div>
  </div>
{/if}
