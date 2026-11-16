<script lang="ts">
  import type { IntervalPreview, Rating } from "$lib/shared/types";

  interface Props {
    intervals: IntervalPreview;
    submitting: boolean;
    onRate: (rating: Rating) => void;
  }

  let { intervals, submitting, onRate }: Props = $props();

  const options: Array<{
    rating: Rating;
    label: string;
    key: string;
    color: string;
    getInterval: (i: IntervalPreview) => string;
  }> = [
    {
      rating: 1,
      label: "Quên",
      key: "1",
      color: "border-rose-500/30 bg-rose-500/10 text-rose-300 hover:bg-rose-500/20",
      getInterval: (i) => i.again,
    },
    {
      rating: 2,
      label: "Khó",
      key: "2",
      color: "border-amber-500/30 bg-amber-500/10 text-amber-300 hover:bg-amber-500/20",
      getInterval: (i) => i.hard,
    },
    {
      rating: 3,
      label: "Nhớ",
      key: "3",
      color: "border-emerald-500/30 bg-emerald-500/10 text-emerald-300 hover:bg-emerald-500/20",
      getInterval: (i) => i.good,
    },
    {
      rating: 4,
      label: "Dễ",
      key: "4",
      color: "border-sky-500/30 bg-sky-500/10 text-sky-300 hover:bg-sky-500/20",
      getInterval: (i) => i.easy,
    },
  ];
</script>

<div class="grid grid-cols-4 gap-3">
  {#each options as opt}
    <button
      type="button"
      disabled={submitting}
      onclick={() => onRate(opt.rating)}
      class="flex flex-col items-center rounded-2xl border px-3 py-3 transition disabled:opacity-50 {opt.color}"
    >
      <span class="text-xs font-medium opacity-75">{opt.getInterval(intervals)}</span>
      <span class="mt-0.5 text-sm font-semibold">{opt.label}</span>
      <span class="mt-1 rounded bg-black/25 px-1.5 py-0.5 font-mono text-[10px] opacity-70">
        {opt.key}
      </span>
    </button>
  {/each}
</div>
