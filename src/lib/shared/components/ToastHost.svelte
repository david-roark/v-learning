<script lang="ts">
  import { AlertCircle, AlertTriangle, CheckCircle2, Info, X } from "@lucide/svelte";
  import { toast } from "$lib/shared/toast.svelte";
  import type { ToastKind } from "$lib/shared/toastQueue";

  const STYLES: Record<ToastKind, string> = {
    success: "border-emerald-500/35 bg-zinc-900/95 text-emerald-200",
    error: "border-rose-500/40 bg-zinc-900/95 text-rose-200",
    warning: "border-amber-500/40 bg-zinc-900/95 text-amber-200",
    info: "border-indigo-500/35 bg-zinc-900/95 text-indigo-200",
  };

  const ICON_COLORS: Record<ToastKind, string> = {
    success: "text-emerald-400",
    error: "text-rose-400",
    warning: "text-amber-400",
    info: "text-indigo-400",
  };
</script>

{#if toast.items.length > 0}
  <div
    class="pointer-events-none fixed top-3 right-3 left-3 z-[100] mx-auto flex max-w-md flex-col gap-2 sm:top-4 sm:right-4 sm:left-auto sm:mx-0 sm:w-96 sm:max-w-none"
    aria-live="polite"
  >
    {#each toast.items as item (item.id)}
      <div
        role={item.kind === "error" ? "alert" : "status"}
        class={`pointer-events-auto flex items-start gap-2.5 rounded-2xl border px-3.5 py-3 shadow-2xl backdrop-blur-md transition-all ${STYLES[item.kind]}`}
      >
        <div class={`mt-0.5 shrink-0 ${ICON_COLORS[item.kind]}`}>
          {#if item.kind === "success"}
            <CheckCircle2 class="h-4 w-4" />
          {:else if item.kind === "error"}
            <AlertCircle class="h-4 w-4" />
          {:else if item.kind === "warning"}
            <AlertTriangle class="h-4 w-4" />
          {:else}
            <Info class="h-4 w-4" />
          {/if}
        </div>

        <p class="min-w-0 flex-1 text-xs leading-relaxed break-words">{item.message}</p>

        <button
          type="button"
          aria-label="Đóng thông báo"
          onclick={() => toast.dismiss(item.id)}
          class="-mr-1 shrink-0 rounded-lg p-1 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200"
        >
          <X class="h-3.5 w-3.5" />
        </button>
      </div>
    {/each}
  </div>
{/if}
