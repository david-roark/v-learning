<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import {
    CloudDownload,
    CloudUpload,
    Download,
    History,
    LogIn,
    LogOut,
    RotateCcw,
    ShieldAlert,
    Upload,
    X,
  } from "@lucide/svelte";
  import { toast } from "$lib/shared/toast.svelte";
  import type { GoogleDriveBackupInfo, LocalBackupEntry } from "$lib/shared/types";
  import { syncApi } from "./api";
  import { getPendingDriveConflict, setPendingDriveConflict } from "./autoBackup.svelte";

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
  let gdriveAutoSync = $state(false);
  let driveConflictTime = $state<string | null>(null);
  let localBackups = $state<LocalBackupEntry[]>([]);
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

  async function refreshLocalBackups() {
    try {
      localBackups = await syncApi.listLocalBackups();
    } catch {
      localBackups = [];
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
      driveConflictTime = getPendingDriveConflict();
      void syncApi.getWebDavSettings().then((s) => {
        url = s.url;
        username = s.username;
        password = s.password;
      });
      void syncApi.getGoogleDriveStatus().then((st) => {
        gdriveConnected = st.connected;
        gdriveEmail = st.userEmail;
        gdriveClientId = st.clientId;
        gdriveAutoSync = st.autoSync;
        showGdriveConfig = !st.clientId;
        if (st.connected) {
          void refreshGoogleBackupInfo();
        } else {
          gdriveBackupInfo = null;
        }
      });
      void refreshLocalBackups();
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
      gdriveAutoSync = st.autoSync;
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
      gdriveAutoSync = st.autoSync;
      gdriveBackupInfo = null;
      driveConflictTime = null;
      toast.info("Đã đăng xuất Google Drive.");
    } catch (e) {
      toast.error(e);
    } finally {
      busy = false;
    }
  }

  async function handleToggleAutoSync() {
    try {
      const st = await syncApi.setGoogleDriveAutoSync(!gdriveAutoSync);
      gdriveAutoSync = st.autoSync;
      toast.success(
        st.autoSync
          ? "Đã bật tự động sao lưu lên Google Drive khi có thay đổi."
          : "Đã tắt tự động sao lưu Google Drive.",
      );
    } catch (e) {
      toast.error(e);
    }
  }

  async function handleGooglePush(force = false) {
    busy = true;
    try {
      const res = await syncApi.pushGoogleDrive(force);
      if (res.conflict) {
        driveConflictTime = res.remoteModifiedTime;
        setPendingDriveConflict(res.remoteModifiedTime);
        toast.warning("Google Drive đang có bản sao lưu mới hơn từ thiết bị khác.");
        return;
      }
      driveConflictTime = null;
      setPendingDriveConflict(null);
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
      driveConflictTime = null;
      setPendingDriveConflict(null);
      toast.success(
        `Đã khôi phục từ Google Drive: ${report.decks} bộ thẻ, ${report.notes} ghi chú.`,
      );
      void refreshGoogleBackupInfo();
      void refreshLocalBackups();
      onRestored();
    } catch (e) {
      toast.error(e);
    } finally {
      busy = false;
    }
  }

  async function handleGoogleMerge() {
    busy = true;
    try {
      const report = await syncApi.mergeGoogleDrive();
      driveConflictTime = null;
      setPendingDriveConflict(null);
      toast.success(
        `Đã gộp thông minh 2 máy & đẩy lên Drive: ${report.decks} bộ thẻ, ${report.notes} ghi chú.`,
      );
      void refreshGoogleBackupInfo();
      void refreshLocalBackups();
      onRestored();
    } catch (e) {
      toast.error(e);
    } finally {
      busy = false;
    }
  }

  async function handleCreateLocalBackup() {
    busy = true;
    try {
      const created = await syncApi.createLocalBackup();
      if (created) {
        toast.success("Đã tạo bản sao lưu an toàn trên máy!");
      } else {
        toast.info("Chưa có dữ liệu thẻ/ghi chú để sao lưu.");
      }
      void refreshLocalBackups();
    } catch (e) {
      toast.error(e);
    } finally {
      busy = false;
    }
  }

  async function handleRestoreLocalBackup(filename: string) {
    busy = true;
    try {
      const report = await syncApi.restoreLocalBackup(filename);
      toast.success(`Đã khôi phục snapshot: ${report.decks} bộ thẻ, ${report.notes} ghi chú.`);
      void refreshLocalBackups();
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
            {#if driveConflictTime}
              <div class="space-y-2 rounded-xl border border-amber-500/40 bg-amber-500/10 p-3">
                <div class="flex items-start gap-2 text-xs text-amber-200">
                  <ShieldAlert class="mt-0.5 h-4 w-4 shrink-0 text-amber-400" />
                  <p>
                    Google Drive có bản sao lưu mới hơn (lúc
                    <strong>{formatBackupTime(driveConflictTime)}</strong>) từ thiết bị khác. Để
                    tránh ghi đè mất dữ liệu, hãy chọn hành động:
                  </p>
                </div>
                <div class="flex flex-wrap gap-2">
                  <button
                    type="button"
                    disabled={busy}
                    onclick={handleGoogleMerge}
                    class="rounded-lg bg-indigo-600 px-3 py-1.5 text-[11px] font-semibold text-white hover:bg-indigo-500 disabled:opacity-50"
                  >
                    Gộp thông minh (Merge 2 máy)
                  </button>
                  <button
                    type="button"
                    disabled={busy}
                    onclick={handleGooglePull}
                    class="rounded-lg bg-emerald-600 px-3 py-1.5 text-[11px] font-semibold text-white hover:bg-emerald-500 disabled:opacity-50"
                  >
                    Lấy bản Drive (Ghi đè máy)
                  </button>
                  <button
                    type="button"
                    disabled={busy}
                    onclick={() => handleGooglePush(true)}
                    class="rounded-lg border border-amber-500/50 px-3 py-1.5 text-[11px] font-medium text-amber-200 hover:bg-amber-500/20 disabled:opacity-50"
                  >
                    Đẩy bản máy (Ghi đè Drive)
                  </button>
                </div>
              </div>
            {/if}

            <label class="flex cursor-pointer items-center justify-between gap-2 pt-0.5 text-xs">
              <span class="text-zinc-300">Tự động sao lưu lên Drive khi có thay đổi</span>
              <input
                type="checkbox"
                checked={gdriveAutoSync}
                onchange={handleToggleAutoSync}
                class="h-4 w-4 rounded border-zinc-700 bg-zinc-900 accent-indigo-500"
              />
            </label>

            <div class="flex flex-wrap gap-2.5 pt-1">
              <button
                type="button"
                disabled={busy}
                onclick={() => handleGooglePush(false)}
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

        <!-- Local Safety Snapshots -->
        <div class="space-y-2.5 rounded-xl border border-zinc-800 bg-zinc-950/60 p-4">
          <div class="flex items-center justify-between gap-2">
            <div class="flex items-center gap-1.5">
              <History class="h-3.5 w-3.5 text-indigo-400" />
              <h3 class="text-xs font-semibold text-zinc-200 uppercase">
                Bản sao lưu an toàn trên máy
              </h3>
            </div>
            <button
              type="button"
              disabled={busy}
              onclick={handleCreateLocalBackup}
              class="text-[11px] font-medium text-indigo-400 hover:text-indigo-300 disabled:opacity-50"
            >
              + Chụp ngay
            </button>
          </div>
          <p class="text-[11px] text-zinc-400">
            Tự động chụp sau khi học/sửa thẻ và ngay trước mỗi lần khôi phục (giữ 10 bản mới nhất).
          </p>

          {#if localBackups.length === 0}
            <p class="text-[11px] text-zinc-500">Chưa có bản snapshot nào trên máy.</p>
          {:else}
            <div class="max-h-36 space-y-1.5 overflow-y-auto pr-1">
              {#each localBackups as snap (snap.filename)}
                <div
                  class="flex items-center justify-between gap-2 rounded-lg border border-zinc-800 bg-zinc-900/70 px-2.5 py-1.5 text-[11px]"
                >
                  <div class="min-w-0">
                    <span class="font-medium text-zinc-200">
                      {new Date(snap.createdAt).toLocaleString("vi-VN")}
                    </span>
                    <span class="ml-1.5 text-zinc-400">
                      ({snap.reason === "pre-restore" ? "Trước khôi phục" : "Tự động"} · {formatBytes(
                        snap.sizeBytes,
                      )})
                    </span>
                  </div>
                  <button
                    type="button"
                    disabled={busy}
                    onclick={() => handleRestoreLocalBackup(snap.filename)}
                    class="inline-flex shrink-0 items-center gap-1 rounded-md border border-zinc-700 px-2 py-1 text-[11px] text-zinc-200 hover:bg-zinc-800 disabled:opacity-50"
                  >
                    <RotateCcw class="h-3 w-3" />
                    Khôi phục
                  </button>
                </div>
              {/each}
            </div>
          {/if}
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
