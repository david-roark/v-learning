use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub fn toggle_quick_lookup(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("quick-lookup") {
        if win.is_visible().unwrap_or(false) {
            let _ = win.hide();
        } else {
            let _ = win.show();
            let _ = win.set_focus();
        }
        return;
    }

    let builder = WebviewWindowBuilder::new(
        app,
        "quick-lookup",
        WebviewUrl::App("quick-lookup".into()),
    )
    .title("V-Learning Quick Lookup")
    .inner_size(460.0, 440.0)
    .resizable(false)
    .always_on_top(true)
    .center();

    if let Ok(win) = builder.build() {
        let _ = win.set_focus();
    }
}

pub fn show_main_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

pub fn setup_desktop_features(app: &AppHandle) -> tauri::Result<()> {
    let open_item = MenuItem::with_id(app, "open", "Mở V-Learning", true, None::<&str>)?;
    let lookup_item = MenuItem::with_id(
        app,
        "lookup",
        "Tra từ nhanh (Ctrl+Shift+D)",
        true,
        None::<&str>,
    )?;
    let quit_item = MenuItem::with_id(app, "quit", "Thoát", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_item, &lookup_item, &quit_item])?;

    let mut tray_builder = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("V-Learning")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_main_window(app),
            "lookup" => toggle_quick_lookup(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        tray_builder = tray_builder.icon(icon);
    }
    tray_builder.build(app)?;

    let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyD);
    let _ = app.global_shortcut().on_shortcut(shortcut, |app, _sc, event| {
        if event.state() == ShortcutState::Pressed {
            toggle_quick_lookup(app);
        }
    });

    Ok(())
}
