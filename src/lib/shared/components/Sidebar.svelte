<script lang="ts">
  import {
    BarChart3,
    BookA,
    BookOpenText,
    Cloud,
    Layers,
    Palette,
    Settings,
    Sparkles,
    Volume2,
  } from "@lucide/svelte";

  export type NavTab = "decks" | "dictionary" | "reader" | "analytics";

  interface Props {
    activeTab: NavTab;
    onSelectTab: (tab: NavTab) => void;
    onOpenAppearance: () => void;
    onOpenTtsSettings: () => void;
    onOpenSync: () => void;
    onOpenAiSettings: () => void;
  }

  let {
    activeTab,
    onSelectTab,
    onOpenAppearance,
    onOpenTtsSettings,
    onOpenSync,
    onOpenAiSettings,
  }: Props = $props();

  const navItems: Array<{ id: NavTab; label: string; shortcut: string; icon: typeof Layers }> = [
    { id: "decks", label: "Bộ thẻ", shortcut: "1", icon: Layers },
    { id: "dictionary", label: "Từ điển", shortcut: "2", icon: BookA },
    { id: "reader", label: "Đọc & Xem", shortcut: "3", icon: BookOpenText },
    { id: "analytics", label: "Thống kê", shortcut: "4", icon: BarChart3 },
  ];
</script>

<aside
  class="flex h-full w-56 shrink-0 flex-col justify-between border-r border-zinc-800 bg-zinc-900/50 p-3.5 select-none"
>
  <div class="space-y-6">
    <!-- Brand -->
    <div class="flex items-center gap-2.5 px-2 pt-1">
      <div
        class="flex h-8 w-8 items-center justify-center rounded-xl bg-indigo-600 text-white shadow-md shadow-indigo-600/20"
      >
        <Sparkles class="h-4 w-4" />
      </div>
      <div>
        <h1 class="text-sm font-bold tracking-tight text-white">V-Learning</h1>
        <p class="text-[10px] text-zinc-400">FSRS · Offline-first</p>
      </div>
    </div>

    <!-- Navigation -->
    <nav class="space-y-1">
      {#each navItems as item (item.id)}
        {@const Icon = item.icon}
        {@const active = activeTab === item.id}
        <button
          type="button"
          onclick={() => onSelectTab(item.id)}
          class="flex w-full items-center justify-between rounded-xl px-3 py-2 text-xs font-medium transition {active
            ? 'bg-indigo-600 text-white shadow-sm'
            : 'text-zinc-400 hover:bg-zinc-800/70 hover:text-zinc-100'}"
        >
          <span class="flex items-center gap-2.5">
            <Icon class="h-4 w-4" />
            {item.label}
          </span>
          <kbd
            class="rounded px-1.5 py-0.5 font-mono text-[10px] {active
              ? 'bg-black/20 text-white/80'
              : 'bg-zinc-800 text-zinc-500'}"
          >
            ^{item.shortcut}
          </kbd>
        </button>
      {/each}
    </nav>
  </div>

  <!-- Bottom Utilities -->
  <div class="space-y-2.5 border-t border-zinc-800 pt-3">
    <div class="rounded-xl bg-zinc-950/60 px-3 py-2 text-[11px] text-zinc-400">
      <span class="block font-medium text-zinc-300">Tra từ nhanh mọi nơi</span>
      <kbd
        class="mt-1 inline-block rounded bg-zinc-800 px-1.5 py-0.5 font-mono text-[10px] text-zinc-300"
      >
        Ctrl+Shift+D
      </kbd>
    </div>

    <div class="space-y-0.5">
      <button
        type="button"
        onclick={onOpenAppearance}
        class="flex w-full items-center gap-2.5 rounded-xl px-3 py-2 text-xs text-zinc-400 transition hover:bg-zinc-800/70 hover:text-zinc-100"
      >
        <Palette class="h-4 w-4" />
        Giao diện & Font
      </button>
      <button
        type="button"
        onclick={onOpenTtsSettings}
        class="flex w-full items-center gap-2.5 rounded-xl px-3 py-2 text-xs text-zinc-400 transition hover:bg-zinc-800/70 hover:text-zinc-100"
      >
        <Volume2 class="h-4 w-4" />
        Phát âm (TTS)
      </button>
      <button
        type="button"
        onclick={onOpenSync}
        class="flex w-full items-center gap-2.5 rounded-xl px-3 py-2 text-xs text-zinc-400 transition hover:bg-zinc-800/70 hover:text-zinc-100"
      >
        <Cloud class="h-4 w-4" />
        Sao lưu & Sync
      </button>
      <button
        type="button"
        onclick={onOpenAiSettings}
        class="flex w-full items-center gap-2.5 rounded-xl px-3 py-2 text-xs text-zinc-400 transition hover:bg-zinc-800/70 hover:text-zinc-100"
      >
        <Settings class="h-4 w-4" />
        Cấu hình AI
      </button>
    </div>
  </div>
</aside>
