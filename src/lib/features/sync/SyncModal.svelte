<script lang="ts">
  import { CloudDownload, CloudUpload, Download, Upload, X } from "@lucide/svelte";
  import { syncApi } from "./api";

  interface Props {
    open: boolean;
    onClose: () => void;
    onRestored: () => void;
  }

  let { open, onClose, onRestored }: Props = $props();

  let url = $state("");
  let username = $state("");
  let password = $state("");
  let busy = $state(false);
  let msg = $state("");
  let error = $state("");

  $effect(() => {
    if (open) {
      msg = "";
      error = "";
      void syncApi.getWebDavSettings().then((s) => {
        url = s.url;
        username = s.username;
        password = s.password;
      });
    }
  });

  async function handleExportJson() {
    busy = true;
    error = "";
    try {
      const json = await syncApi.exportBackup();
      const blob = new Blob([json], { type: "application/json" });
      const dlUrl = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = dlUrl;
      a.download = `vlearning-backup-${new Date().toISOString().slice(0, 10)}.json`;
      a.click();
      URL.revokeObjectURL(dlUrl);
      msg = "Đã xuất file sao lưu JSON thành công!";
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handleImportJson(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    busy = true;
    error = "";
    try {
      const text = await file.text();
      const rep = await syncApi.importBackup(text);
      msg = `Đã khôi phục: ${rep.decks} bộ thẻ, ${rep.notes} ghi chú, ${rep.reviewLogs} lịch sử ôn.`;
      onRestored();
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
      input.value = "";
    }
  }

  async function handlePush() {
    busy = true;
    error = "";
    try {
      await syncApi.saveWebDavSettings({ url, username, password });
      await syncApi.pushWebDav();
      msg = "Đã đẩy bản sao lưu lên WebDAV thành công!";
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function handlePull() {
    busy = true;
    error = "";
    try {
      await syncApi.saveWebDavSettings({ url, username, password });
      const rep = await syncApi.pullWebDav();
      msg = `Đã đồng bộ về máy: ${rep.decks} bộ thẻ, ${rep.notes} ghi chú.`;
      onRestored();
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-xs">
    <div class="w-full max-w-lg rounded-2xl border border-zinc-800 bg-zinc-900 p-6 shadow-2xl">
      <div class="mb-5 flex items-center justify-between">
        <h2 class="text-base font-semibold text-white">Sao lưu & Đồng bộ (Local / WebDAV)</h2>
        <button
          type="button"
          onclick={onClose}
          class="rounded-lg p-1.5 text-zinc-400 hover:bg-zinc-800"
        >
          <X class="h-4 w-4" />
        </button>
      </div>

      <div class="space-y-5">
        <!-- Local Backup -->
        <div class="rounded-xl border border-zinc-800 bg-zinc-950/60 p-4">
          <h3 class="text-xs font-semibold text-zinc-200 uppercase">Sao lưu cục bộ (JSON)</h3>
          <div class="mt-3 flex flex-wrap gap-2.5">
            <button
              type="button"
              disabled={busy}
              onclick={handleExportJson}
              class="inline-flex items-center gap-1.5 rounded-xl bg-indigo-600 px-3.5 py-2 text-xs font-medium text-white hover:bg-indigo-500 disabled:opacity-50"
            >
              <Download class="h-3.5 w-3.5" />
              Xuất file Backup (.json)
            </button>

            <label class="inline-flex cursor-pointer items-center gap-1.5 rounded-xl border border-zinc-700 px-3.5 py-2 text-xs font-medium text-zinc-200 hover:bg-zinc-800">
              <Upload class="h-3.5 w-3.5" />
              Khôi phục từ .json
              <input type="file" accept=".json" onchange={handleImportJson} class="hidden" />
            </label>
          </div>
        </div>

        <!-- WebDAV Sync -->
        <div class="space-y-3 rounded-xl border border-zinc-800 bg-zinc-950/60 p-4">
          <h3 class="text-xs font-semibold text-zinc-200 uppercase">WebDAV Cloud Sync (Nextcloud / NAS)</h3>

          <input
            type="text"
            bind:value={url}
            placeholder="https://cloud.example.com/remote.php/dav/files/user/vlearning"
            class="w-full rounded-xl border border-zinc-700 bg-zinc-900 px-3 py-2 text-xs text-zinc-100 focus:border-indigo-500 focus:outline-none"
          />
          <div class="grid grid-cols-2 gap-2.5">
            <input
              type="text"
              bind:value={username}
              placeholder="Username"
              class="rounded-xl border border-zinc-700 bg-zinc-900 px-3 py-2 text-xs text-zinc-100 focus:border-indigo-500 focus:outline-none"
            />
            <input
              type="password"
              bind:value={password}
              placeholder="App Password"
              class="rounded-xl border border-zinc-700 bg-zinc-900 px-3 py-2 text-xs text-zinc-100 focus:border-indigo-500 focus:outline-none"
            />
          </div>

          <div class="flex gap-2.5 pt-1">
            <button
              type="button"
              disabled={busy}
              onclick={handlePush}
              class="inline-flex flex-1 items-center justify-center gap-1.5 rounded-xl border border-indigo-500/40 bg-indigo-500/15 py-2 text-xs font-medium text-indigo-300 hover:bg-indigo-500/25 disabled:opacity-50"
            >
              <CloudUpload class="h-3.5 w-3.5" />
              Đẩy lên Cloud (Push)
            </button>
            <button
              type="button"
              disabled={busy}
              onclick={handlePull}
              class="inline-flex flex-1 items-center justify-center gap-1.5 rounded-xl border border-emerald-500/40 bg-emerald-500/15 py-2 text-xs font-medium text-emerald-300 hover:bg-emerald-500/25 disabled:opacity-50"
            >
              <CloudDownload class="h-3.5 w-3.5" />
              Kéo về máy (Pull)
            </button>
          </div>
        </div>

        {#if msg}
          <p class="rounded-xl bg-emerald-500/10 p-3 text-xs text-emerald-400">{msg}</p>
        {/if}
        {#if error}
          <p class="rounded-xl bg-rose-500/10 p-3 text-xs text-rose-400">{error}</p>
        {/if}
      </div>
    </div>
  </div>
{/if}
