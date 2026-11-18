<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { appearance, applyAppearance } from "$lib/features/appearance/appearance.svelte";
  import {
    flushPendingAutoBackup,
    runStartupGoogleDriveSync,
  } from "$lib/features/sync/autoBackup.svelte";
  import ToastHost from "$lib/shared/components/ToastHost.svelte";

  let { children } = $props();

  $effect(() => {
    applyAppearance(appearance);
  });

  onMount(() => {
    void runStartupGoogleDriveSync();
    const onHide = () => {
      if (document.visibilityState === "hidden") {
        flushPendingAutoBackup();
      }
    };
    const onPageHide = () => flushPendingAutoBackup();
    document.addEventListener("visibilitychange", onHide);
    window.addEventListener("pagehide", onPageHide);
    return () => {
      document.removeEventListener("visibilitychange", onHide);
      window.removeEventListener("pagehide", onPageHide);
    };
  });
</script>

<div class="min-h-screen bg-zinc-950 text-zinc-100 selection:bg-indigo-500/30">
  {@render children()}
  <ToastHost />
</div>
