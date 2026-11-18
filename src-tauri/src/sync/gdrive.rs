use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rusqlite::Connection;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::db::settings_repo;
use crate::error::{AppError, AppResult};
use crate::http::build_client;
use crate::models::{
    GoogleDriveBackupInfo, GoogleDriveSettings, GoogleDriveStatus, GoogleOAuthStart,
};

const GOOGLE_AUTH_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_ENDPOINT: &str = "https://www.googleapis.com/oauth2/v2/userinfo";
const GOOGLE_DRIVE_API_BASE: &str = "https://www.googleapis.com/drive/v3";
const GOOGLE_DRIVE_UPLOAD_BASE: &str = "https://www.googleapis.com/upload/drive/v3";
const BACKUP_FILENAME: &str = "vlearning-backup.json";
const OAUTH_SCOPES: &str =
    "https://www.googleapis.com/auth/drive.appdata https://www.googleapis.com/auth/userinfo.email";
const OAUTH_TIMEOUT_SECS: u64 = 180;

pub struct PendingOAuthSession {
    listener: TcpListener,
    redirect_uri: String,
    code_verifier: String,
    state: String,
    client_id: String,
    client_secret: String,
    cancelled: Arc<AtomicBool>,
}

#[derive(Default)]
pub struct PendingGoogleOAuth {
    session: Mutex<Option<PendingOAuthSession>>,
    active_cancel: Mutex<Option<Arc<AtomicBool>>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default = "default_expires_in")]
    pub expires_in: i64,
}

fn default_expires_in() -> i64 {
    3600
}

#[derive(Debug, Deserialize)]
struct UserInfoResponse {
    #[serde(default)]
    email: String,
}

#[derive(Debug, Deserialize)]
struct DriveFileListResponse {
    #[serde(default)]
    files: Vec<DriveFileEntry>,
}

#[derive(Debug, Deserialize)]
struct DriveFileEntry {
    id: String,
    #[serde(default, rename = "modifiedTime")]
    modified_time: Option<String>,
    #[serde(default)]
    size: Option<String>,
}

fn now_unix_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(0))
        .unwrap_or(0)
}

fn builtin_client_id() -> &'static str {
    option_env!("VLEARNING_GOOGLE_CLIENT_ID").unwrap_or("")
}

fn builtin_client_secret() -> &'static str {
    option_env!("VLEARNING_GOOGLE_CLIENT_SECRET").unwrap_or("")
}

pub fn resolve_credentials(
    stored: &GoogleDriveSettings,
    input_client_id: Option<&str>,
    input_client_secret: Option<&str>,
) -> AppResult<(String, String)> {
    let client_id = input_client_id
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or_else(|| {
            let s = stored.client_id.trim();
            if s.is_empty() { None } else { Some(s) }
        })
        .or_else(|| {
            let s = builtin_client_id().trim();
            if s.is_empty() { None } else { Some(s) }
        })
        .ok_or_else(|| {
            AppError::Validation(
                "Chưa có Google OAuth Client ID. Vui lòng nhập Client ID trước khi đăng nhập."
                    .into(),
            )
        })?
        .to_string();

    let client_secret = input_client_secret
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or_else(|| {
            let s = stored.client_secret.trim();
            if s.is_empty() { None } else { Some(s) }
        })
        .or_else(|| {
            let s = builtin_client_secret().trim();
            if s.is_empty() { None } else { Some(s) }
        })
        .unwrap_or("")
        .to_string();

    Ok((client_id, client_secret))
}

pub fn status_from_settings(settings: &GoogleDriveSettings) -> GoogleDriveStatus {
    let effective_client_id = if settings.client_id.trim().is_empty() {
        builtin_client_id().trim().to_string()
    } else {
        settings.client_id.trim().to_string()
    };
    let has_secret =
        !settings.client_secret.trim().is_empty() || !builtin_client_secret().trim().is_empty();
    let connected = !settings.refresh_token.trim().is_empty()
        || (!settings.access_token.trim().is_empty() && settings.expires_at > now_unix_secs());

    GoogleDriveStatus {
        connected,
        user_email: settings.user_email.clone(),
        client_id: effective_client_id,
        has_client_secret: has_secret,
        auto_sync: settings.auto_sync,
    }
}

pub fn get_status(conn: &Connection) -> AppResult<GoogleDriveStatus> {
    let settings = settings_repo::get_google_drive_settings(conn)?;
    Ok(status_from_settings(&settings))
}

pub fn set_auto_sync(conn: &Connection, enabled: bool) -> AppResult<GoogleDriveStatus> {
    let mut settings = settings_repo::get_google_drive_settings(conn)?;
    settings.auto_sync = enabled;
    settings_repo::save_google_drive_settings(conn, &settings)?;
    Ok(status_from_settings(&settings))
}

pub fn logout(conn: &Connection) -> AppResult<GoogleDriveStatus> {
    let mut settings = settings_repo::get_google_drive_settings(conn)?;
    settings.access_token.clear();
    settings.refresh_token.clear();
    settings.expires_at = 0;
    settings.user_email.clear();
    settings.last_synced_remote_modified.clear();
    settings.auto_sync = false;
    settings_repo::save_google_drive_settings(conn, &settings)?;
    Ok(status_from_settings(&settings))
}

pub(crate) fn generate_pkce_pair() -> (String, String) {
    let mut bytes = [0u8; 32];
    bytes[..16].copy_from_slice(Uuid::new_v4().as_bytes());
    bytes[16..].copy_from_slice(Uuid::new_v4().as_bytes());
    let verifier = URL_SAFE_NO_PAD.encode(bytes);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    (verifier, challenge)
}

fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                out.push_str(&format!("%{byte:02X}"));
            }
        }
    }
    out
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                if let Ok(hex_str) = std::str::from_utf8(&bytes[i + 1..i + 3])
                    && let Ok(val) = u8::from_str_radix(hex_str, 16)
                {
                    out.push(val);
                    i += 3;
                    continue;
                }
                out.push(bytes[i]);
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub(crate) fn build_auth_url(
    client_id: &str,
    redirect_uri: &str,
    code_challenge: &str,
    state: &str,
) -> String {
    format!(
        "{GOOGLE_AUTH_ENDPOINT}?client_id={}&redirect_uri={}&response_type=code&scope={}&code_challenge={}&code_challenge_method=S256&state={}&access_type=offline&prompt=consent",
        percent_encode(client_id),
        percent_encode(redirect_uri),
        percent_encode(OAUTH_SCOPES),
        percent_encode(code_challenge),
        percent_encode(state),
    )
}

pub(crate) fn parse_oauth_callback_query(
    request_line: &str,
    expected_state: &str,
) -> AppResult<String> {
    let path = request_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| AppError::Validation("Phản hồi OAuth không hợp lệ.".into()))?;
    let query = path
        .split_once('?')
        .map(|(_, q)| q)
        .ok_or_else(|| AppError::Validation("Thiếu tham số callback OAuth.".into()))?;

    let mut code: Option<String> = None;
    let mut state: Option<String> = None;
    let mut error: Option<String> = None;
    let mut error_desc: Option<String> = None;

    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        let key = percent_decode(k);
        let val = percent_decode(v);
        match key.as_str() {
            "code" => code = Some(val),
            "state" => state = Some(val),
            "error" => error = Some(val),
            "error_description" => error_desc = Some(val),
            _ => {}
        }
    }

    if let Some(err) = error {
        let msg = error_desc
            .filter(|d| !d.is_empty())
            .unwrap_or_else(|| err.clone());
        return Err(AppError::Validation(format!(
            "Đăng nhập Google bị từ chối: {msg}"
        )));
    }

    if state.as_deref() != Some(expected_state) {
        return Err(AppError::Validation(
            "Tham số state OAuth không khớp.".into(),
        ));
    }

    code.filter(|c| !c.is_empty())
        .ok_or_else(|| AppError::Validation("Không nhận được mã xác thực từ Google.".into()))
}

pub fn start_login(
    conn: &Connection,
    pending: &PendingGoogleOAuth,
    input_client_id: Option<&str>,
    input_client_secret: Option<&str>,
) -> AppResult<GoogleOAuthStart> {
    let mut settings = settings_repo::get_google_drive_settings(conn)?;
    let (client_id, client_secret) =
        resolve_credentials(&settings, input_client_id, input_client_secret)?;

    if let Some(cid) = input_client_id.map(str::trim).filter(|s| !s.is_empty()) {
        settings.client_id = cid.to_string();
    }
    if let Some(sec) = input_client_secret.map(str::trim)
        && !sec.is_empty()
    {
        settings.client_secret = sec.to_string();
    }
    settings_repo::save_google_drive_settings(conn, &settings)?;

    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| AppError::Internal(format!("Không mở được cổng OAuth cục bộ: {e}")))?;
    let port = listener
        .local_addr()
        .map_err(|e| AppError::Internal(format!("Không lấy được cổng cục bộ: {e}")))?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{port}");
    let (code_verifier, code_challenge) = generate_pkce_pair();
    let state = Uuid::new_v4().simple().to_string();
    let auth_url = build_auth_url(&client_id, &redirect_uri, &code_challenge, &state);

    if let Ok(mut cancel_guard) = pending.active_cancel.lock()
        && let Some(prev) = cancel_guard.take()
    {
        prev.store(true, Ordering::Relaxed);
    }

    let cancelled = Arc::new(AtomicBool::new(false));
    if let Ok(mut cancel_guard) = pending.active_cancel.lock() {
        *cancel_guard = Some(Arc::clone(&cancelled));
    }

    let mut guard = pending
        .session
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    *guard = Some(PendingOAuthSession {
        listener,
        redirect_uri,
        code_verifier,
        state,
        client_id,
        client_secret,
        cancelled,
    });

    Ok(GoogleOAuthStart { auth_url })
}

pub fn cancel_login(pending: &PendingGoogleOAuth) -> AppResult<()> {
    if let Ok(mut cancel_guard) = pending.active_cancel.lock()
        && let Some(flag) = cancel_guard.take()
    {
        flag.store(true, Ordering::Relaxed);
    }
    if let Ok(mut session_guard) = pending.session.lock() {
        *session_guard = None;
    }
    Ok(())
}

fn write_oauth_html_response(stream: &mut std::net::TcpStream, success: bool, message: &str) {
    let title = if success {
        "Đăng nhập thành công"
    } else {
        "Đăng nhập thất bại"
    };
    let status_line = if success { "200 OK" } else { "400 Bad Request" };
    let body = format!(
        "<!doctype html><html lang=\"vi\"><head><meta charset=\"utf-8\"><title>{title}</title>\
        <style>body{{font-family:system-ui,sans-serif;display:flex;align-items:center;justify-content:center;min-height:100vh;margin:0;background:#f8fafc;color:#0f172a;}}\
        .card{{background:#fff;padding:2rem;border-radius:1rem;box-shadow:0 10px 25px rgba(15,23,42,.08);max-width:420px;text-align:center;}}</style></head>\
        <body><div class=\"card\"><h2>{title}</h2><p>{message}</p></div></body></html>"
    );
    let response = format!(
        "HTTP/1.1 {status_line}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn wait_for_oauth_code(
    listener: &TcpListener,
    expected_state: &str,
    timeout_secs: u64,
    cancelled: &AtomicBool,
) -> AppResult<String> {
    listener
        .set_nonblocking(true)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);

    loop {
        if cancelled.load(Ordering::Relaxed) {
            return Err(AppError::Validation("Đã hủy đăng nhập Google.".into()));
        }
        if Instant::now() >= deadline {
            return Err(AppError::Validation(
                "Hết thời gian chờ đăng nhập Google (180 giây). Vui lòng thử lại.".into(),
            ));
        }

        match listener.accept() {
            Ok((mut stream, _)) => {
                let _ = stream.set_nonblocking(false);
                let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                let mut buf = [0u8; 8192];
                let n = stream.read(&mut buf).unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]);
                let first_line = req.lines().next().unwrap_or("");
                if first_line.contains("/favicon.ico") {
                    let _ =
                        stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
                    continue;
                }
                match parse_oauth_callback_query(first_line, expected_state) {
                    Ok(code) => {
                        write_oauth_html_response(
                            &mut stream,
                            true,
                            "Bạn có thể đóng tab này và quay lại ứng dụng V-Learning.",
                        );
                        return Ok(code);
                    }
                    Err(err) => {
                        write_oauth_html_response(&mut stream, false, &err.to_string());
                        return Err(err);
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                return Err(AppError::Internal(format!(
                    "Lỗi lắng nghe callback OAuth: {e}"
                )));
            }
        }
    }
}

fn encode_form(params: &[(&str, &str)]) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

pub(crate) async fn exchange_code_with_endpoint(
    client: &reqwest::Client,
    token_url: &str,
    client_id: &str,
    client_secret: &str,
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
) -> AppResult<TokenResponse> {
    let mut params = vec![
        ("grant_type", "authorization_code"),
        ("client_id", client_id),
        ("code", code),
        ("code_verifier", code_verifier),
        ("redirect_uri", redirect_uri),
    ];
    if !client_secret.is_empty() {
        params.push(("client_secret", client_secret));
    }

    let resp = client
        .post(token_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(encode_form(&params))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Lỗi kết nối Google OAuth: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Validation(format!(
            "Đổi mã OAuth thất bại ({status}): {body}"
        )));
    }

    resp.json::<TokenResponse>()
        .await
        .map_err(|e| AppError::Internal(format!("Không đọc được token từ Google: {e}")))
}

pub(crate) async fn refresh_token_with_endpoint(
    client: &reqwest::Client,
    token_url: &str,
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> AppResult<TokenResponse> {
    let mut params = vec![
        ("grant_type", "refresh_token"),
        ("client_id", client_id),
        ("refresh_token", refresh_token),
    ];
    if !client_secret.is_empty() {
        params.push(("client_secret", client_secret));
    }

    let resp = client
        .post(token_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(encode_form(&params))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Lỗi làm mới token Google: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Validation(format!(
            "Phiên đăng nhập Google đã hết hạn hoặc bị thu hồi ({status}): {body}"
        )));
    }

    resp.json::<TokenResponse>()
        .await
        .map_err(|e| AppError::Internal(format!("Không đọc được token mới: {e}")))
}

pub(crate) async fn fetch_userinfo_with_endpoint(
    client: &reqwest::Client,
    userinfo_url: &str,
    access_token: &str,
) -> AppResult<String> {
    let resp = client
        .get(userinfo_url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Lỗi lấy thông tin tài khoản Google: {e}")))?;

    if !resp.status().is_success() {
        return Ok(String::new());
    }

    let info = resp
        .json::<UserInfoResponse>()
        .await
        .unwrap_or(UserInfoResponse {
            email: String::new(),
        });
    Ok(info.email)
}

pub fn take_pending_session(pending: &PendingGoogleOAuth) -> AppResult<PendingOAuthSession> {
    let mut guard = pending
        .session
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    guard
        .take()
        .ok_or_else(|| AppError::Validation("Không có phiên đăng nhập Google đang chờ.".into()))
}

pub async fn complete_login_flow(
    session: PendingOAuthSession,
) -> AppResult<(TokenResponse, String)> {
    let PendingOAuthSession {
        listener,
        redirect_uri,
        code_verifier,
        state,
        client_id,
        client_secret,
        cancelled,
    } = session;

    let code = tauri::async_runtime::spawn_blocking(move || {
        wait_for_oauth_code(&listener, &state, OAUTH_TIMEOUT_SECS, &cancelled)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Lỗi tiến trình OAuth: {e}")))??;

    let client = build_client(30)?;
    let token = exchange_code_with_endpoint(
        &client,
        GOOGLE_TOKEN_ENDPOINT,
        &client_id,
        &client_secret,
        &code,
        &code_verifier,
        &redirect_uri,
    )
    .await?;

    let email =
        fetch_userinfo_with_endpoint(&client, GOOGLE_USERINFO_ENDPOINT, &token.access_token)
            .await?;
    Ok((token, email))
}

pub fn save_login_tokens(
    conn: &Connection,
    token: &TokenResponse,
    email: &str,
) -> AppResult<GoogleDriveStatus> {
    let mut settings = settings_repo::get_google_drive_settings(conn)?;
    settings.access_token = token.access_token.clone();
    if let Some(rt) = token.refresh_token.as_deref()
        && !rt.trim().is_empty()
    {
        settings.refresh_token = rt.trim().to_string();
    }
    settings.expires_at = now_unix_secs() + token.expires_in.max(60);
    if !email.trim().is_empty() {
        settings.user_email = email.trim().to_string();
    }
    settings_repo::save_google_drive_settings(conn, &settings)?;
    Ok(status_from_settings(&settings))
}

pub(crate) fn needs_token_refresh(settings: &GoogleDriveSettings) -> bool {
    settings.access_token.trim().is_empty() || settings.expires_at <= now_unix_secs() + 60
}

pub(crate) async fn refresh_settings_token_with_endpoint(
    client: &reqwest::Client,
    token_url: &str,
    settings: &GoogleDriveSettings,
) -> AppResult<GoogleDriveSettings> {
    if settings.refresh_token.trim().is_empty() {
        return Err(AppError::Validation(
            "Chưa đăng nhập Google Drive hoặc thiếu refresh token.".into(),
        ));
    }
    let (client_id, client_secret) = resolve_credentials(settings, None, None)?;
    let refreshed = refresh_token_with_endpoint(
        client,
        token_url,
        &client_id,
        &client_secret,
        &settings.refresh_token,
    )
    .await?;

    let mut updated = settings.clone();
    updated.access_token = refreshed.access_token;
    if let Some(rt) = refreshed.refresh_token
        && !rt.trim().is_empty()
    {
        updated.refresh_token = rt;
    }
    updated.expires_at = now_unix_secs() + refreshed.expires_in.max(60);
    Ok(updated)
}

pub async fn refresh_access_token_if_needed(
    settings: GoogleDriveSettings,
) -> AppResult<(GoogleDriveSettings, bool)> {
    if !needs_token_refresh(&settings) {
        return Ok((settings, false));
    }
    let client = build_client(30)?;
    let updated =
        refresh_settings_token_with_endpoint(&client, GOOGLE_TOKEN_ENDPOINT, &settings).await?;
    Ok((updated, true))
}

async fn find_backup_file_meta(
    client: &reqwest::Client,
    api_base: &str,
    access_token: &str,
) -> AppResult<Option<DriveFileEntry>> {
    let query = format!("name='{BACKUP_FILENAME}' and trashed=false");
    let url = format!(
        "{}/files?spaces=appDataFolder&q={}&fields=files(id,name,modifiedTime,size)&pageSize=1",
        api_base.trim_end_matches('/'),
        percent_encode(&query),
    );
    let resp = client
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Lỗi truy vấn Google Drive: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Validation(format!(
            "Google Drive từ chối yêu cầu ({status}): {body}"
        )));
    }

    let list = resp
        .json::<DriveFileListResponse>()
        .await
        .map_err(|e| AppError::Internal(format!("Không đọc được danh sách file Drive: {e}")))?;

    Ok(list.files.into_iter().next())
}

pub(crate) async fn drive_get_backup_info_with_endpoint(
    client: &reqwest::Client,
    api_base: &str,
    access_token: &str,
) -> AppResult<GoogleDriveBackupInfo> {
    match find_backup_file_meta(client, api_base, access_token).await? {
        Some(entry) => Ok(GoogleDriveBackupInfo {
            exists: true,
            modified_time: entry.modified_time,
            size_bytes: entry.size.and_then(|s| s.parse::<u64>().ok()),
        }),
        None => Ok(GoogleDriveBackupInfo {
            exists: false,
            modified_time: None,
            size_bytes: None,
        }),
    }
}

pub async fn drive_get_backup_info(access_token: &str) -> AppResult<GoogleDriveBackupInfo> {
    let client = build_client(30)?;
    drive_get_backup_info_with_endpoint(&client, GOOGLE_DRIVE_API_BASE, access_token).await
}

pub fn check_push_conflict(remote_modified: Option<&str>, last_synced: &str) -> bool {
    match remote_modified {
        Some(rm) if !rm.trim().is_empty() => rm.trim() != last_synced.trim(),
        _ => false,
    }
}

pub(crate) async fn drive_push_with_endpoints(
    client: &reqwest::Client,
    api_base: &str,
    upload_base: &str,
    access_token: &str,
    json_body: &str,
) -> AppResult<Option<String>> {
    let existing_id = find_backup_file_meta(client, api_base, access_token)
        .await?
        .map(|f| f.id);

    let resp = if let Some(file_id) = existing_id {
        let url = format!(
            "{}/files/{}?uploadType=media&fields=id,modifiedTime",
            upload_base.trim_end_matches('/'),
            percent_encode(&file_id),
        );
        client
            .patch(&url)
            .bearer_auth(access_token)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(json_body.to_string())
            .send()
            .await
    } else {
        let boundary = format!("vlearning_{}", Uuid::new_v4().simple());
        let metadata = serde_json::json!({
            "name": BACKUP_FILENAME,
            "parents": ["appDataFolder"],
        })
        .to_string();
        let multipart_body = format!(
            "--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{metadata}\r\n--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{json_body}\r\n--{boundary}--\r\n"
        );
        let url = format!(
            "{}/files?uploadType=multipart&fields=id,modifiedTime",
            upload_base.trim_end_matches('/'),
        );
        client
            .post(&url)
            .bearer_auth(access_token)
            .header(
                "Content-Type",
                format!("multipart/related; boundary={boundary}"),
            )
            .body(multipart_body)
            .send()
            .await
    }
    .map_err(|e| AppError::Internal(format!("Lỗi tải backup lên Google Drive: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Validation(format!(
            "Tải lên Google Drive thất bại ({status}): {body}"
        )));
    }

    let uploaded_meta = resp.json::<DriveFileEntry>().await.ok();
    Ok(uploaded_meta.and_then(|m| m.modified_time))
}

pub(crate) async fn drive_pull_with_endpoints(
    client: &reqwest::Client,
    api_base: &str,
    access_token: &str,
) -> AppResult<(String, Option<String>)> {
    let meta = find_backup_file_meta(client, api_base, access_token)
        .await?
        .ok_or_else(|| AppError::NotFound("Chưa có bản sao lưu nào trên Google Drive.".into()))?;

    let url = format!(
        "{}/files/{}?alt=media",
        api_base.trim_end_matches('/'),
        percent_encode(&meta.id),
    );
    let resp = client
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Lỗi tải backup từ Google Drive: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Validation(format!(
            "Tải từ Google Drive thất bại ({status}): {body}"
        )));
    }

    let body = resp
        .text()
        .await
        .map_err(|e| AppError::Internal(format!("Không đọc được dữ liệu backup: {e}")))?;
    Ok((body, meta.modified_time))
}

pub async fn drive_push_backup(access_token: &str, json_body: &str) -> AppResult<Option<String>> {
    let client = build_client(30)?;
    drive_push_with_endpoints(
        &client,
        GOOGLE_DRIVE_API_BASE,
        GOOGLE_DRIVE_UPLOAD_BASE,
        access_token,
        json_body,
    )
    .await
}

pub async fn drive_pull_backup(access_token: &str) -> AppResult<(String, Option<String>)> {
    let client = build_client(30)?;
    drive_pull_with_endpoints(&client, GOOGLE_DRIVE_API_BASE, access_token).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http_mock::spawn_one_shot;
    use std::io::Write;
    use std::net::TcpStream;

    #[test]
    fn pkce_pair_is_valid_base64url() {
        let (verifier, challenge) = generate_pkce_pair();
        assert_eq!(verifier.len(), 43);
        assert_eq!(challenge.len(), 43);
        assert!(!verifier.contains('='));
        assert!(!challenge.contains('='));
        assert_ne!(verifier, challenge);
    }

    #[test]
    fn parse_oauth_callback_query_extracts_code_and_validates_state() {
        let code = parse_oauth_callback_query(
            "GET /?state=abc123&code=4%2F0AdLx_test&scope=email HTTP/1.1",
            "abc123",
        )
        .expect("parse code");
        assert_eq!(code, "4/0AdLx_test");

        let wrong_state =
            parse_oauth_callback_query("GET /?state=other&code=xyz HTTP/1.1", "abc123");
        assert!(wrong_state.is_err());

        let denied =
            parse_oauth_callback_query("GET /?error=access_denied&state=abc123 HTTP/1.1", "abc123");
        assert!(denied.is_err());
    }

    #[test]
    fn wait_for_oauth_code_accepts_loopback_request() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("addr").port();
        let handle = std::thread::spawn(move || {
            let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).expect("connect");
            stream
                .write_all(
                    b"GET /?code=my_auth_code&state=state_ok HTTP/1.1\r\nHost: localhost\r\n\r\n",
                )
                .expect("write");
            let mut resp = String::new();
            let _ = stream.read_to_string(&mut resp);
            assert!(resp.contains("200 OK"));
        });

        let cancel = AtomicBool::new(false);
        let code = wait_for_oauth_code(&listener, "state_ok", 5, &cancel).expect("wait code");
        assert_eq!(code, "my_auth_code");
        handle.join().expect("join");
    }

    #[test]
    fn wait_for_oauth_code_stops_immediately_when_cancelled() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let cancel = AtomicBool::new(true);
        let err = wait_for_oauth_code(&listener, "state_ok", 60, &cancel)
            .expect_err("should cancel immediately");
        assert!(err.to_string().contains("Đã hủy"));
    }

    #[test]
    fn exchange_and_refresh_token_work_against_mock() {
        tauri::async_runtime::block_on(async {
            let (token_url, rx) = spawn_one_shot(
                "200 OK",
                "application/json",
                br#"{"access_token":"ya29.new","refresh_token":"1//rt","expires_in":3600}"#
                    .to_vec(),
            );
            let client = build_client(5).expect("client");
            let tok = exchange_code_with_endpoint(
                &client,
                &token_url,
                "cid",
                "csec",
                "auth_code",
                "verifier",
                "http://127.0.0.1:1234",
            )
            .await
            .expect("exchange");
            assert_eq!(tok.access_token, "ya29.new");
            assert_eq!(tok.refresh_token.as_deref(), Some("1//rt"));
            let req = rx.recv().expect("captured");
            assert!(req.body.contains("grant_type=authorization_code"));
            assert!(req.body.contains("code_verifier=verifier"));

            let (refresh_url, refresh_rx) = spawn_one_shot(
                "200 OK",
                "application/json",
                br#"{"access_token":"ya29.refreshed","expires_in":3600}"#.to_vec(),
            );
            let settings = GoogleDriveSettings {
                client_id: "cid".into(),
                client_secret: String::new(),
                access_token: "ya29.old".into(),
                refresh_token: "1//rt".into(),
                expires_at: 100,
                user_email: "u@example.com".into(),
                ..Default::default()
            };
            let updated = refresh_settings_token_with_endpoint(&client, &refresh_url, &settings)
                .await
                .expect("refresh");
            assert_eq!(updated.access_token, "ya29.refreshed");
            assert_eq!(updated.refresh_token, "1//rt");
            let r_req = refresh_rx.recv().expect("captured");
            assert!(r_req.body.contains("grant_type=refresh_token"));
        });
    }

    #[test]
    fn drive_push_creates_new_file_and_updates_existing_file() {
        tauri::async_runtime::block_on(async {
            let client = build_client(5).expect("client");

            // 1) Empty file list -> multipart POST create
            let (list_url, list_rx) =
                spawn_one_shot("200 OK", "application/json", br#"{"files":[]}"#.to_vec());
            let (upload_url, upload_rx) = spawn_one_shot(
                "200 OK",
                "application/json",
                br#"{"id":"file_1","modifiedTime":"2026-11-18T12:00:00Z"}"#.to_vec(),
            );

            let mod_time = drive_push_with_endpoints(
                &client,
                &list_url,
                &upload_url,
                "tok1",
                r#"{"decks":[]}"#,
            )
            .await
            .expect("push create");
            assert_eq!(mod_time.as_deref(), Some("2026-11-18T12:00:00Z"));
            let list_req = list_rx.recv().expect("list captured");
            let upload_req = upload_rx.recv().expect("upload captured");
            assert!(list_req.request_line.contains("spaces=appDataFolder"));
            assert!(upload_req.request_line.starts_with("POST "));
            assert!(upload_req.request_line.contains("uploadType=multipart"));
            assert!(upload_req.body.contains("vlearning-backup.json"));

            // 2) Existing file -> PATCH media
            let (list_url2, list_rx2) = spawn_one_shot(
                "200 OK",
                "application/json",
                br#"{"files":[{"id":"file_1"}]}"#.to_vec(),
            );
            let (upload_url2, upload_rx2) =
                spawn_one_shot("200 OK", "application/json", br#"{"id":"file_1"}"#.to_vec());
            drive_push_with_endpoints(&client, &list_url2, &upload_url2, "tok1", r#"{"decks":[]}"#)
                .await
                .expect("push update");
            let _ = list_rx2.recv().expect("list2 captured");
            let patch_req = upload_rx2.recv().expect("patch captured");
            assert!(patch_req.request_line.starts_with("PATCH "));
            assert!(
                patch_req
                    .request_line
                    .contains("/files/file_1?uploadType=media")
            );
        });
    }

    #[test]
    fn drive_pull_downloads_backup_json() {
        tauri::async_runtime::block_on(async {
            let client = build_client(5).expect("client");

            // Multi-response local mock for 2 sequential requests (list + download)
            let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
            let port = listener.local_addr().expect("addr").port();
            let api_base = format!("http://127.0.0.1:{port}");
            let handle = std::thread::spawn(move || {
                let mut requests = Vec::new();
                for (idx, body) in [
                    r#"{"files":[{"id":"file_42","modifiedTime":"2026-11-18T12:30:00Z"}]}"#,
                    r#"{"version":1,"decks":[]}"#,
                ]
                .iter()
                .enumerate()
                {
                    let (mut stream, _) = listener.accept().expect("accept");
                    let mut buf = [0u8; 4096];
                    let n = stream.read(&mut buf).unwrap_or(0);
                    requests.push(String::from_utf8_lossy(&buf[..n]).into_owned());
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    stream.write_all(resp.as_bytes()).expect("write");
                    let _ = idx;
                }
                requests
            });

            let (downloaded, mod_time) = drive_pull_with_endpoints(&client, &api_base, "tok1")
                .await
                .expect("pull ok");
            assert_eq!(downloaded, r#"{"version":1,"decks":[]}"#);
            assert_eq!(mod_time.as_deref(), Some("2026-11-18T12:30:00Z"));
            let reqs = handle.join().expect("join");
            assert!(reqs[0].contains("spaces=appDataFolder"));
            assert!(reqs[1].contains("/files/file_42?alt=media"));

            let (empty_url, empty_rx) =
                spawn_one_shot("200 OK", "application/json", br#"{"files":[]}"#.to_vec());
            let err = drive_pull_with_endpoints(&client, &empty_url, "tok1")
                .await
                .expect_err("not found");
            assert!(err.to_string().contains("Chưa có bản sao lưu"));
            let _ = empty_rx.recv();
        });
    }

    #[test]
    fn drive_backup_info_returns_metadata_or_not_exists() {
        tauri::async_runtime::block_on(async {
            let client = build_client(5).expect("client");

            let (found_url, found_rx) = spawn_one_shot(
                "200 OK",
                "application/json",
                br#"{"files":[{"id":"f1","name":"vlearning-backup.json","modifiedTime":"2026-11-18T12:00:00Z","size":"4096"}]}"#.to_vec(),
            );
            let info = drive_get_backup_info_with_endpoint(&client, &found_url, "tok1")
                .await
                .expect("info ok");
            assert!(info.exists);
            assert_eq!(info.modified_time.as_deref(), Some("2026-11-18T12:00:00Z"));
            assert_eq!(info.size_bytes, Some(4096));
            let _ = found_rx.recv();

            let (empty_url, empty_rx) =
                spawn_one_shot("200 OK", "application/json", br#"{"files":[]}"#.to_vec());
            let empty_info = drive_get_backup_info_with_endpoint(&client, &empty_url, "tok1")
                .await
                .expect("empty ok");
            assert!(!empty_info.exists);
            assert!(empty_info.modified_time.is_none());
            assert!(empty_info.size_bytes.is_none());
            let _ = empty_rx.recv();
        });
    }

    #[test]
    fn resolve_credentials_validates_and_prefers_input_over_stored() {
        let empty = GoogleDriveSettings::default();
        if builtin_client_id().trim().is_empty() {
            assert!(resolve_credentials(&empty, None, None).is_err());
        }

        let stored = GoogleDriveSettings {
            client_id: "stored_cid".into(),
            client_secret: "stored_sec".into(),
            ..Default::default()
        };
        let (cid, csec) = resolve_credentials(&stored, None, None).expect("from stored");
        assert_eq!(cid, "stored_cid");
        assert_eq!(csec, "stored_sec");

        let (cid2, csec2) = resolve_credentials(&stored, Some(" input_cid "), Some(" input_sec "))
            .expect("from input");
        assert_eq!(cid2, "input_cid");
        assert_eq!(csec2, "input_sec");
    }

    #[test]
    fn needs_token_refresh_checks_empty_and_expiry_window() {
        let now = now_unix_secs();
        let empty = GoogleDriveSettings::default();
        assert!(needs_token_refresh(&empty));

        let expiring = GoogleDriveSettings {
            access_token: "ya29.abc".into(),
            expires_at: now + 30,
            ..Default::default()
        };
        assert!(needs_token_refresh(&expiring));

        let fresh = GoogleDriveSettings {
            access_token: "ya29.abc".into(),
            expires_at: now + 3600,
            ..Default::default()
        };
        assert!(!needs_token_refresh(&fresh));
    }

    #[test]
    fn save_login_tokens_and_logout_update_db_and_status() {
        let conn = crate::db::open_in_memory_for_test();
        settings_repo::save_google_drive_settings(
            &conn,
            &GoogleDriveSettings {
                client_id: "cid_1".into(),
                ..Default::default()
            },
        )
        .expect("seed settings");

        let token = TokenResponse {
            access_token: "ya29.token".into(),
            refresh_token: Some("1//refresh".into()),
            expires_in: 3600,
        };
        let status = save_login_tokens(&conn, &token, "learner@example.com").expect("save");
        assert!(status.connected);
        assert_eq!(status.user_email, "learner@example.com");
        assert_eq!(status.client_id, "cid_1");

        let after_logout = logout(&conn).expect("logout");
        assert!(!after_logout.connected);
        assert!(after_logout.user_email.is_empty());
        assert_eq!(after_logout.client_id, "cid_1");
    }

    #[test]
    fn start_login_and_cancel_login_manage_pending_session() {
        let conn = crate::db::open_in_memory_for_test();
        let pending = PendingGoogleOAuth::default();

        let started = start_login(&conn, &pending, Some("my_cid"), Some("my_sec")).expect("start");
        assert!(started.auth_url.contains("client_id=my_cid"));
        assert!(started.auth_url.contains("code_challenge_method=S256"));

        cancel_login(&pending).expect("cancel");
        let missing = take_pending_session(&pending);
        assert!(missing.is_err());

        // Starting again should succeed and allow taking the session
        let _ = start_login(&conn, &pending, None, None).expect("restart");
        let session = take_pending_session(&pending).expect("take");
        assert_eq!(session.client_id, "my_cid");
        assert_eq!(session.client_secret, "my_sec");
    }

    #[test]
    fn check_push_conflict_detects_remote_changes() {
        // No remote file yet -> safe to push
        assert!(!check_push_conflict(None, ""));
        // Matches last synced -> safe to push
        assert!(!check_push_conflict(
            Some("2026-11-18T12:00:00Z"),
            "2026-11-18T12:00:00Z"
        ));
        // Unsynced device vs existing remote file -> conflict
        assert!(check_push_conflict(Some("2026-11-18T12:00:00Z"), ""));
        // Remote newer/different -> conflict
        assert!(check_push_conflict(
            Some("2026-11-18T13:00:00Z"),
            "2026-11-18T12:00:00Z"
        ));
    }
}
