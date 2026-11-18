<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { CloudDownload, CloudUpload, Download, LogIn, LogOut, Upload, X } from "@lucide/svelte";
  import { toast } from "$lib/shared/toast.svelte";
  import type { GoogleDriveBackupInfo } from "$lib/shared/types";
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
  let gdriveConnected = $state(false);
  let gdriveEmail = $state("");
  let gdriveClientId = $state("");
  let gdriveClientSecret = $state("");
  let showGdriveConfig = $state(false);
  let gdriveLoggingIn = $state(false);
  let gdriveBackupInfo = $state<GoogleDriveBackupInfo | null>(null);
  let loadingBackupInfo = $state(false);
  let busy = $state(false);

  async function refreshGoogleBackupInfo() {
    loadingBackupInfo = true;
    try {
      gdriveBackupInfo = await syncApi.getGoogleDriveBackupInfo();
    } catch {
      gdriveBackupInfo = null;
    } finally {
      loadingBackupInfo = false;
    }
  }

  function formatBytes(bytes: number | null): string {
    if (bytes === null) return "";
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  }

  function formatBackupTime(iso: string | null): string {
    if (!iso) return "";
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return iso;
    return d.toLocaleString("vi-VN");
  }

  $effect(() => {
    if (open) {
      void syncApi.getWebDavSettings().then((s) => {
        url = s.url;
        username = s.username;
        password = s.password;
      });
      void syncApi.getGoogleDriveStatus().then((st) => {
        gdriveConnected = st.connected;
        gdriveEmail = st.userEmail;
        gdriveClientId = st.clientId;
        showGdriveConfig = !st.clientId;
        if (st.connected) {
          void refreshGoogleBackupInfo();
        } else {
          gdriveBackupInfo = null;
        }
      });
    } else if (gdriveLoggingIn) {
      gdriveLoggingIn = false;
      busy = false;
      void syncApi.cancelGoogleDriveLogin();
    }
  });

  async function handleGoogleLogin() {
    busy = true;
    gdriveLoggingIn = true;
    try {
      const { authUrl } = await syncApi.startGoogleDriveLogin(gdriveClientId, gdriveClientSecret);
      await openUrl(authUrl);
      const st = await syncApi.completeGoogleDriveLogin();
      gdriveConnected = st.connected;
      gdriveEmail = st.userEmail;
      gdriveClientId = st.clientId;
      gdriveClientSecret = "";
      showGdriveConfig = false;
      toast.success(
        st.userEmail ? `Đã kết nối Google Drive (${st.userEmail}).` : "Đã kết nối Google Drive.",
      );
      void refreshGoogleBackupInfo();
    } catch (e) {
      void syncApi.cancelGoogleDriveLogin();
      toast.error(e);
    } finally {
      gdriveLoggingIn = false;
      busy = false;
    }
  }

  async function handleCancelGoogleLogin() {
    await syncApi.cancelGoogleDriveLogin();
    gdriveLoggingIn = false;
    busy = false;
  }

  async function handleGoogleLogout() {
    busy = true;
    try {
      const st = await syncApi.logoutGoogleDrive();
      gdriveConnected = st.connected;
      gdriveEmail = st.userEmail;
      gdriveBackupInfo = null;
      toast.info("Đã đăng xuất Google Drive.");
    } catch (e) {
      toast.error(e);
    } finally {
      busy = false;
    }
  }

  async function handleGooglePush() {
    busy = true;
    try {
      await syncApi.pushGoogleDrive();
      toast.success("Đã sao lưu dữ liệu lên Google Drive thành công!");
      void refreshGoogleBackupInfo();
    } catch (e) {
      toast.error(e);
    } finally {
      busy = false;
    }
  }

  async function handleGooglePull() {
    busy = true;
    try {
      const report = await syncApi.pullGoogleDrive();
      toast.success(
        `Đã khôi phục từ Google Drive: ${report.decks} bộ thẻ, ${report.notes} ghi chú.`,
      );
      onRestored();
    } catch (e) {
      toast.error(e);
    } finally {
      busy = false;
    }
  }

  async function handleExportJson() {
    busy = true;
    try {
      const json = await syncApi.exportBackup();
      const blob = new Blob([json], { type: "application/json" });
      const dlUrl = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = dlUrl;
      a.download = `vlearning-backup-${new Date().toISOString().slice(0, 10)}.json`;
      a.click();
      URL.revokeObjectURL(dlUrl);
      toast.success("Đã xuất file sao lưu JSON thành công!");
    } catch (err) {
      toast.error(err);
    } finally {
      busy = false;
    }
  }

  async function handleImportJson(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    busy = true;
    try {
      const text = await file.text();
      const rep = await syncApi.importBackup(text);
      toast.success(
        `Đã khôi phục: ${rep.decks} bộ thẻ, ${rep.notes} ghi chú, ${rep.reviewLogs} lịch sử ôn.`,
      );
      onRestored();
    } catch (err) {
      toast.error(err);
    } finally {
      busy = false;
      input.value = "";
    }
  }

  async function handlePush() {
    busy = true;
    try {
      await syncApi.saveWebDavSettings({ url, username, password });
      await syncApi.pushWebDav();
      toast.success("Đã đẩy bản sao lưu lên WebDAV thành công!");
    } catch (err) {
      toast.error(err);
    } finally {
      busy = false;
    }
  }

  async function handlePull() {
    busy = true;
    try {
      await syncApi.saveWebDavSettings({ url, username, password });
      const rep = await syncApi.pullWebDav();
      toast.success(`Đã đồng bộ về máy: ${rep.decks} bộ thẻ, ${rep.notes} ghi chú.`);
      onRestored();
    } catch (err) {
      toast.error(err);
    } finally {
      busy = false;
    }
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-xs">
    <div
      class="max-h-[90vh] w-full max-w-lg overflow-y-auto rounded-2xl border border-zinc-800 bg-zinc-900 p-6 shadow-2xl"
    >
      <div class="mb-5 flex items-center justify-between">
        <h2 class="text-base font-semibold text-white">
          Sao lưu & Đồng bộ (Google Drive / Local / WebDAV)
        </h2>
        <button
          type="button"
          onclick={onClose}
          class="rounded-lg p-1.5 text-zinc-400 hover:bg-zinc-800"
        >
          <X class="h-4 w-4" />
        </button>
      </div>

      <div class="space-y-5">
        <!-- Google Drive Sync -->
        <div class="space-y-3 rounded-xl border border-zinc-800 bg-zinc-950/60 p-4">
          <div class="flex items-center justify-between gap-2">
            <div>
              <h3 class="text-xs font-semibold text-zinc-200 uppercase">Google Drive (App Data)</h3>
              <p class="mt-0.5 text-[11px] text-zinc-400">
                {#if gdriveConnected}
                  Đã kết nối: <span class="text-emerald-400">{gdriveEmail || "Google Account"}</span
                  >
                {:else}
                  Lưu riêng tư trong Google Drive của bạn (miễn phí).
                {/if}
              </p>
              {#if gdriveConnected}
                <p class="mt-0.5 text-[11px] text-zinc-400">
                  {#if loadingBackupInfo}
                    Đang kiểm tra bản sao lưu trên Drive...
                  {:else if gdriveBackupInfo?.exists}
                    Bản sao lưu trên Drive:
                    <span class="text-zinc-200"
                      >{formatBackupTime(gdriveBackupInfo.modifiedTime)}</span
                    >
                    {#if gdriveBackupInfo.sizeBytes !== null}
                      <span> ({formatBytes(gdriveBackupInfo.sizeBytes)})</span>
                    {/if}
                  {:else if gdriveBackupInfo}
                    Chưa có bản sao lưu nào trên Google Drive.
                  {/if}
                </p>
              {/if}
            </div>
            <button
              type="button"
              onclick={() => (showGdriveConfig = !showGdriveConfig)}
              class="text-[11px] text-zinc-400 underline hover:text-zinc-200"
            >
              {showGdriveConfig ? "Ẩn OAuth ID" : "Cấu hình OAuth"}
            </button>
          </div>

          {#if showGdriveConfig}
            <div class="space-y-2">
              <input
                type="text"
                bind:value={gdriveClientId}
                placeholder="Google OAuth Client ID (Desktop App)"
                class="w-full rounded-xl border border-zinc-700 bg-zinc-900 px-3 py-2 text-xs text-zinc-100 focus:border-indigo-500 focus:outline-none"
              />
              <input
                type="password"
                bind:value={gdriveClientSecret}
                placeholder="Client Secret (tuỳ chọn cho Desktop OAuth)"
                class="w-full rounded-xl border border-zinc-700 bg-zinc-900 px-3 py-2 text-xs text-zinc-100 focus:border-indigo-500 focus:outline-none"
              />
            </div>
          {/if}

          {#if gdriveConnected}
            <div class="flex flex-wrap gap-2.5 pt-1">
              <button
                type="button"
                disabled={busy}
                onclick={handleGooglePush}
                class="inline-flex flex-1 items-center justify-center gap-1.5 rounded-xl border border-indigo-500/40 bg-indigo-500/15 py-2 text-xs font-medium text-indigo-300 hover:bg-indigo-500/25 disabled:opacity-50"
              >
                <CloudUpload class="h-3.5 w-3.5" />
                Sao lưu lên Drive
              </button>
              <button
                type="button"
                disabled={busy}
                onclick={handleGooglePull}
                class="inline-flex flex-1 items-center justify-center gap-1.5 rounded-xl border border-emerald-500/40 bg-emerald-500/15 py-2 text-xs font-medium text-emerald-300 hover:bg-emerald-500/25 disabled:opacity-50"
              >
                <CloudDownload class="h-3.5 w-3.5" />
                Khôi phục từ Drive
              </button>
              <button
                type="button"
                disabled={busy}
                onclick={handleGoogleLogout}
                class="inline-flex items-center justify-center gap-1.5 rounded-xl border border-zinc-700 px-3 py-2 text-xs font-medium text-zinc-300 hover:bg-zinc-800 disabled:opacity-50"
              >
                <LogOut class="h-3.5 w-3.5" />
                Đăng xuất
              </button>
            </div>
          {:else if gdriveLoggingIn}
            <div class="flex gap-2.5">
              <div
                class="inline-flex flex-1 items-center justify-center gap-2 rounded-xl bg-white/80 px-4 py-2 text-xs font-semibold text-zinc-900"
              >
                <LogIn class="h-3.5 w-3.5" />
                Đang chờ xác thực trình duyệt...
              </div>
              <button
                type="button"
                onclick={handleCancelGoogleLogin}
                class="inline-flex items-center justify-center rounded-xl border border-zinc-700 px-3.5 py-2 text-xs font-medium text-zinc-300 hover:bg-zinc-800"
              >
                Hủy
              </button>
            </div>
          {:else}
            <button
              type="button"
              disabled={busy}
              onclick={handleGoogleLogin}
              class="inline-flex w-full items-center justify-center gap-2 rounded-xl bg-white px-4 py-2 text-xs font-semibold text-zinc-900 hover:bg-zinc-100 disabled:opacity-50"
            >
              <LogIn class="h-3.5 w-3.5" />
              Đăng nhập với Google
            </button>
          {/if}
        </div>

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

            <label
              class="inline-flex cursor-pointer items-center gap-1.5 rounded-xl border border-zinc-700 px-3.5 py-2 text-xs font-medium text-zinc-200 hover:bg-zinc-800"
            >
              <Upload class="h-3.5 w-3.5" />
              Khôi phục từ .json
              <input type="file" accept=".json" onchange={handleImportJson} class="hidden" />
            </label>
          </div>
        </div>

        <!-- WebDAV Sync -->
        <div class="space-y-3 rounded-xl border border-zinc-800 bg-zinc-950/60 p-4">
          <h3 class="text-xs font-semibold text-zinc-200 uppercase">
            WebDAV Cloud Sync (Nextcloud / NAS)
          </h3>

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
      </div>
    </div>
  </div>
{/if}
