<script lang="ts">
  import { Check, ChevronDown } from "@lucide/svelte";

  export interface SelectOption {
    value: string;
    label: string;
  }

  interface Props {
    options: SelectOption[];
    value: string;
    placeholder?: string;
    prefix?: string;
    class?: string;
  }

  let {
    options,
    value = $bindable(""),
    placeholder = "Chọn...",
    prefix = "",
    class: className = "",
  }: Props = $props();

  let open = $state(false);
  let containerEl = $state<HTMLDivElement | null>(null);

  const selectedLabel = $derived(options.find((o) => o.value === value)?.label ?? placeholder);

  function choose(val: string) {
    value = val;
    open = false;
  }

  function handleWindowClick(e: MouseEvent) {
    if (open && containerEl && !containerEl.contains(e.target as Node)) {
      open = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (open && e.key === "Escape") {
      open = false;
    }
  }
</script>

<svelte:window onclick={handleWindowClick} onkeydown={handleKeydown} />

<div bind:this={containerEl} class="relative {className}">
  <button
    type="button"
    aria-haspopup="listbox"
    aria-expanded={open}
    onclick={() => (open = !open)}
    class="flex w-full items-center justify-between gap-2 rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2 text-xs text-zinc-200 transition hover:border-zinc-600 focus:border-indigo-500 focus:outline-none"
  >
    <span class="truncate">
      {#if prefix}
        <span class="text-zinc-400">{prefix}</span>
      {/if}
      {selectedLabel}
    </span>
    <ChevronDown class="h-3.5 w-3.5 shrink-0 text-zinc-400 transition {open ? 'rotate-180' : ''}" />
  </button>

  {#if open}
    <div
      role="listbox"
      class="absolute right-0 z-50 mt-1.5 max-h-56 w-full min-w-[180px] overflow-y-auto rounded-xl border border-zinc-700/80 bg-zinc-900 p-1 shadow-xl"
    >
      {#each options as opt (opt.value)}
        {@const active = opt.value === value}
        <button
          type="button"
          role="option"
          aria-selected={active}
          onclick={() => choose(opt.value)}
          class="flex w-full items-center justify-between gap-2 rounded-lg px-2.5 py-2 text-left text-xs transition {active
            ? 'bg-indigo-600/20 text-indigo-300'
            : 'text-zinc-300 hover:bg-zinc-800'}"
        >
          <span class="truncate">{opt.label}</span>
          {#if active}
            <Check class="h-3.5 w-3.5 shrink-0 text-indigo-400" />
          {/if}
        </button>
      {:else}
        <div class="px-2.5 py-2 text-xs text-zinc-500">Không có lựa chọn</div>
      {/each}
    </div>
  {/if}
</div>
