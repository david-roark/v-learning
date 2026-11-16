<script lang="ts">
  import "../app.css";
  import { appearance, applyAppearance } from "$lib/features/appearance/appearance.svelte";

  let { children } = $props();

  $effect(() => {
    applyAppearance(appearance);
  });
</script>

<div class="min-h-screen bg-zinc-950 text-zinc-100 selection:bg-indigo-500/30">
  {@render children()}
</div>
