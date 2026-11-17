<script lang="ts">
  import {
    BarChart3,
    BookA,
    BookOpenText,
    Cloud,
    Gamepad2,
    Info,
    Layers,
    Palette,
    PanelLeftClose,
    PanelLeftOpen,
    Settings,
    Volume2,
  } from "@lucide/svelte";
  import { onMount } from "svelte";

  export type NavTab = "decks" | "dictionary" | "reader" | "games" | "analytics";

  interface Props {
    activeTab: NavTab;
    onSelectTab: (tab: NavTab) => void;
    onOpenAppearance: () => void;
    onOpenTtsSettings: () => void;
    onOpenSync: () => void;
    onOpenAiSettings: () => void;
    onOpenAbout: () => void;
  }

  let {
    activeTab,
    onSelectTab,
    onOpenAppearance,
    onOpenTtsSettings,
    onOpenSync,
    onOpenAiSettings,
    onOpenAbout,
  }: Props = $props();

  let showMobileSettings = $state(false);
  let collapsed = $state(false);

  const COLLAPSE_KEY = "vlearning_sidebar_collapsed";

  function toggleCollapsed() {
    collapsed = !collapsed;
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(COLLAPSE_KEY, collapsed ? "1" : "0");
    }
  }

  onMount(() => {
    if (typeof localStorage !== "undefined") {
      collapsed = localStorage.getItem(COLLAPSE_KEY) === "1";
    }
  });

  const navItems: Array<{ id: NavTab; label: string; shortcut: string; icon: typeof Layers }> = [
    { id: "decks", label: "Bộ thẻ", shortcut: "1", icon: Layers },
    { id: "dictionary", label: "Từ điển", shortcut: "2", icon: BookA },
    { id: "reader", label: "Đọc & Xem", shortcut: "3", icon: BookOpenText },
    { id: "games", label: "Mini-game", shortcut: "4", icon: Gamepad2 },
    { id: "analytics", label: "Thống kê", shortcut: "5", icon: BarChart3 },
  ];
</script>

<aside
  class="hidden h-full shrink-0 flex-col justify-between border-r border-zinc-800 bg-zinc-900/50 select-none transition-all duration-200 md:flex {collapsed
    ? 'w-16 p-2.5'
    : 'w-56 p-3.5'}"
>
  <div class="space-y-5">
    <!-- Brand + Collapse Toggle -->
    <div
      class="flex items-center pt-1 {collapsed ? 'flex-col gap-2' : 'justify-between gap-2 px-1.5'}"
    >
      <div class="flex min-w-0 items-center gap-2.5">
        <img
          src="/logo.png"
          alt="V-Learning"
          class="h-8 w-8 shrink-0 rounded-xl shadow-md shadow-indigo-600/20"
        />
        {#if !collapsed}
          <div class="min-w-0">
            <h1 class="truncate text-sm font-bold tracking-tight text-white">V-Learning</h1>
            <p class="truncate text-[10px] text-zinc-400">FSRS · Offline-first</p>
          </div>
        {/if}
      </div>

      <button
        type="button"
        onclick={toggleCollapsed}
        title={collapsed ? "Mở rộng thanh điều hướng" : "Thu gọn thanh điều hướng"}
        aria-label={collapsed ? "Mở rộng thanh điều hướng" : "Thu gọn thanh điều hướng"}
        class="shrink-0 rounded-lg p-1.5 text-zinc-400 transition hover:bg-zinc-800 hover:text-zinc-100"
      >
        {#if collapsed}
          <PanelLeftOpen class="h-4 w-4" />
        {:else}
          <PanelLeftClose class="h-4 w-4" />
        {/if}
      </button>
    </div>

    <!-- Navigation -->
    <nav class="space-y-1">
      {#each navItems as item (item.id)}
        {@const Icon = item.icon}
        {@const active = activeTab === item.id}
        <button
          type="button"
          onclick={() => onSelectTab(item.id)}
          title={collapsed ? `${item.label} (Ctrl+${item.shortcut})` : undefined}
          class="flex w-full items-center rounded-xl py-2 text-xs font-medium transition {collapsed
            ? 'justify-center px-0'
            : 'justify-between px-3'} {active
            ? 'bg-indigo-600 text-white shadow-sm'
            : 'text-zinc-400 hover:bg-zinc-800/70 hover:text-zinc-100'}"
        >
          <span class="flex items-center gap-2.5">
            <Icon class="h-4 w-4 shrink-0" />
            {#if !collapsed}
              <span>{item.label}</span>
            {/if}
          </span>
          {#if !collapsed}
            <kbd
              class="rounded px-1.5 py-0.5 font-mono text-[10px] {active
                ? 'bg-black/20 text-white/80'
                : 'bg-zinc-800 text-zinc-500'}"
            >
              ^{item.shortcut}
            </kbd>
          {/if}
        </button>
      {/each}
    </nav>
  </div>

  <!-- Bottom Utilities -->
  <div class="space-y-2.5 border-t border-zinc-800 pt-3">
    {#if !collapsed}
      <div class="rounded-xl bg-zinc-950/60 px-3 py-2 text-[11px] text-zinc-400">
        <span class="block font-medium text-zinc-300">Tra từ nhanh mọi nơi</span>
        <kbd
          class="mt-1 inline-block rounded bg-zinc-800 px-1.5 py-0.5 font-mono text-[10px] text-zinc-300"
        >
          Ctrl+Shift+D
        </kbd>
      </div>
    {/if}

    <div class="space-y-0.5">
      <button
        type="button"
        onclick={onOpenAppearance}
        title={collapsed ? "Giao diện & Font" : undefined}
        class="flex w-full items-center gap-2.5 rounded-xl py-2 text-xs text-zinc-400 transition hover:bg-zinc-800/70 hover:text-zinc-100 {collapsed
          ? 'justify-center px-0'
          : 'px-3'}"
      >
        <Palette class="h-4 w-4 shrink-0" />
        {#if !collapsed}
          <span>Giao diện & Font</span>
        {/if}
      </button>
      <button
        type="button"
        onclick={onOpenTtsSettings}
        title={collapsed ? "Phát âm (TTS)" : undefined}
        class="flex w-full items-center gap-2.5 rounded-xl py-2 text-xs text-zinc-400 transition hover:bg-zinc-800/70 hover:text-zinc-100 {collapsed
          ? 'justify-center px-0'
          : 'px-3'}"
      >
        <Volume2 class="h-4 w-4 shrink-0" />
        {#if !collapsed}
          <span>Phát âm (TTS)</span>
        {/if}
      </button>
      <button
        type="button"
        onclick={onOpenSync}
        title={collapsed ? "Sao lưu & Sync" : undefined}
        class="flex w-full items-center gap-2.5 rounded-xl py-2 text-xs text-zinc-400 transition hover:bg-zinc-800/70 hover:text-zinc-100 {collapsed
          ? 'justify-center px-0'
          : 'px-3'}"
      >
        <Cloud class="h-4 w-4 shrink-0" />
        {#if !collapsed}
          <span>Sao lưu & Sync</span>
        {/if}
      </button>
      <button
        type="button"
        onclick={onOpenAiSettings}
        title={collapsed ? "Cấu hình AI" : undefined}
        class="flex w-full items-center gap-2.5 rounded-xl py-2 text-xs text-zinc-400 transition hover:bg-zinc-800/70 hover:text-zinc-100 {collapsed
          ? 'justify-center px-0'
          : 'px-3'}"
      >
        <Settings class="h-4 w-4 shrink-0" />
        {#if !collapsed}
          <span>Cấu hình AI</span>
        {/if}
      </button>
      <button
        type="button"
        onclick={onOpenAbout}
        title={collapsed ? "Giới thiệu (About)" : undefined}
        class="flex w-full items-center gap-2.5 rounded-xl py-2 text-xs text-zinc-400 transition hover:bg-zinc-800/70 hover:text-zinc-100 {collapsed
          ? 'justify-center px-0'
          : 'px-3'}"
      >
        <Info class="h-4 w-4 shrink-0" />
        {#if !collapsed}
          <span>Giới thiệu</span>
        {/if}
      </button>
    </div>
  </div>
</aside>

<!-- Mobile Bottom Navigation Bar -->
{#if showMobileSettings}
  <div
    class="fixed right-2 bottom-16 z-50 w-52 space-y-1 rounded-2xl border border-zinc-800 bg-zinc-900/95 p-2 shadow-2xl backdrop-blur-md md:hidden"
  >
    <button
      type="button"
      onclick={() => {
        showMobileSettings = false;
        onOpenAppearance();
      }}
      class="flex w-full items-center gap-2.5 rounded-xl px-3 py-2 text-xs text-zinc-200 hover:bg-zinc-800"
    >
      <Palette class="h-4 w-4 text-indigo-400" />
      Giao diện & Font
    </button>
    <button
      type="button"
      onclick={() => {
        showMobileSettings = false;
        onOpenTtsSettings();
      }}
      class="flex w-full items-center gap-2.5 rounded-xl px-3 py-2 text-xs text-zinc-200 hover:bg-zinc-800"
    >
      <Volume2 class="h-4 w-4 text-indigo-400" />
      Phát âm (TTS)
    </button>
    <button
      type="button"
      onclick={() => {
        showMobileSettings = false;
        onOpenSync();
      }}
      class="flex w-full items-center gap-2.5 rounded-xl px-3 py-2 text-xs text-zinc-200 hover:bg-zinc-800"
    >
      <Cloud class="h-4 w-4 text-indigo-400" />
      Sao lưu & Sync
    </button>
    <button
      type="button"
      onclick={() => {
        showMobileSettings = false;
        onOpenAiSettings();
      }}
      class="flex w-full items-center gap-2.5 rounded-xl px-3 py-2 text-xs text-zinc-200 hover:bg-zinc-800"
    >
      <Settings class="h-4 w-4 text-indigo-400" />
      Cấu hình AI
    </button>
    <button
      type="button"
      onclick={() => {
        showMobileSettings = false;
        onOpenAbout();
      }}
      class="flex w-full items-center gap-2.5 rounded-xl px-3 py-2 text-xs text-zinc-200 hover:bg-zinc-800"
    >
      <Info class="h-4 w-4 text-indigo-400" />
      Giới thiệu (About)
    </button>
  </div>
{/if}

<nav
  class="fixed inset-x-0 bottom-0 z-40 flex items-center justify-around border-t border-zinc-800 bg-zinc-950/95 px-1 pt-1.5 pb-[calc(0.375rem+env(safe-area-inset-bottom))] backdrop-blur md:hidden"
>
  {#each navItems as item (item.id)}
    {@const Icon = item.icon}
    {@const active = activeTab === item.id}
    <button
      type="button"
      onclick={() => {
        showMobileSettings = false;
        onSelectTab(item.id);
      }}
      class="flex flex-1 flex-col items-center gap-0.5 rounded-xl py-1 text-[10px] font-medium transition {active
        ? 'text-indigo-400'
        : 'text-zinc-400 hover:text-zinc-200'}"
    >
      <Icon class="h-4 w-4" />
      <span class="truncate">{item.label}</span>
    </button>
  {/each}
  <button
    type="button"
    onclick={() => (showMobileSettings = !showMobileSettings)}
    class="flex flex-1 flex-col items-center gap-0.5 rounded-xl py-1 text-[10px] font-medium transition {showMobileSettings
      ? 'text-indigo-400'
      : 'text-zinc-400 hover:text-zinc-200'}"
  >
    <Settings class="h-4 w-4" />
    <span class="truncate">Cài đặt</span>
  </button>
</nav>
