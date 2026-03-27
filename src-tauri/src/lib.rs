mod profiles;
mod terminal;
mod worktree;

use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State, WindowEvent,
};
use tauri_plugin_autostart::MacosLauncher;

struct TrayState {
    icon: Option<TrayIcon>,
    profile_paths: Vec<(String, String)>, // (id, name)
}

// --- Terminal Commands ---

#[tauri::command]
fn create_terminal(
    cwd: String,
    cmd: Option<String>,
    state: State<'_, terminal::TerminalState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.create(&cwd, cmd.as_deref(), app)
}

#[tauri::command]
fn write_terminal(
    id: String,
    data: String,
    state: State<'_, terminal::TerminalState>,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.write(&id, data.as_bytes())
}

#[tauri::command]
fn resize_terminal(
    id: String,
    rows: u16,
    cols: u16,
    state: State<'_, terminal::TerminalState>,
) -> Result<(), String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    manager.resize(&id, rows, cols)
}

#[tauri::command]
fn close_terminal(
    id: String,
    state: State<'_, terminal::TerminalState>,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.close(&id)
}

// --- Worktree Commands ---

#[tauri::command]
fn list_worktrees(project_path: String) -> Result<Vec<worktree::WorktreeInfo>, String> {
    worktree::list_worktrees(&project_path)
}

#[tauri::command]
fn get_project_info(project_path: String) -> Result<worktree::ProjectInfo, String> {
    worktree::get_project_info(&project_path)
}

#[tauri::command]
fn add_worktree(
    project_path: String,
    path: String,
    branch: String,
) -> Result<String, String> {
    worktree::add_worktree(&project_path, &path, &branch)
}

#[tauri::command]
fn remove_worktree(project_path: String, worktree_path: String) -> Result<(), String> {
    worktree::remove_worktree(&project_path, &worktree_path)
}

// --- Git Commands ---

#[tauri::command]
fn get_diff(worktree_path: String) -> Result<String, String> {
    worktree::get_diff(&worktree_path)
}

#[tauri::command]
fn get_staged_diff(worktree_path: String) -> Result<String, String> {
    worktree::get_staged_diff(&worktree_path)
}

#[tauri::command]
fn get_git_log(worktree_path: String, count: Option<u32>) -> Result<Vec<worktree::LogEntry>, String> {
    worktree::get_log(&worktree_path, count.unwrap_or(50))
}

#[tauri::command]
fn get_git_status(worktree_path: String) -> Result<Vec<worktree::FileStatus>, String> {
    worktree::get_status(&worktree_path)
}

#[tauri::command]
fn get_file_diff(
    worktree_path: String,
    file: String,
    context_lines: u32,
    staged: bool,
) -> Result<String, String> {
    worktree::get_file_diff(&worktree_path, &file, context_lines, staged)
}

#[tauri::command]
fn stage_file(worktree_path: String, file: String) -> Result<(), String> {
    worktree::stage_file(&worktree_path, &file)
}

#[tauri::command]
fn unstage_file(worktree_path: String, file: String) -> Result<(), String> {
    worktree::unstage_file(&worktree_path, &file)
}

#[tauri::command]
fn read_file_base64(worktree_path: String, file: String) -> Result<String, String> {
    worktree::read_file_base64(&worktree_path, &file)
}

#[tauri::command]
fn read_git_file_base64(worktree_path: String, file: String) -> Result<String, String> {
    worktree::read_git_file_base64(&worktree_path, &file)
}

// --- Profile Commands ---

#[tauri::command]
fn list_profiles(state: State<'_, profiles::ProfileState>) -> Result<Vec<profiles::Profile>, String> {
    let mgr = state.lock().map_err(|e| e.to_string())?;
    Ok(mgr.list())
}

#[tauri::command]
fn get_profile(id: String, state: State<'_, profiles::ProfileState>) -> Result<profiles::Profile, String> {
    let mgr = state.lock().map_err(|e| e.to_string())?;
    mgr.get(&id)
}

#[tauri::command]
fn get_default_profile(state: State<'_, profiles::ProfileState>) -> Result<profiles::Profile, String> {
    let mgr = state.lock().map_err(|e| e.to_string())?;
    Ok(mgr.get_default())
}

#[tauri::command]
fn create_profile(name: String, color: String, state: State<'_, profiles::ProfileState>) -> Result<profiles::Profile, String> {
    let mut mgr = state.lock().map_err(|e| e.to_string())?;
    mgr.create(&name, &color)
}

#[tauri::command]
fn update_profile(id: String, name: String, color: String, state: State<'_, profiles::ProfileState>) -> Result<profiles::Profile, String> {
    let mut mgr = state.lock().map_err(|e| e.to_string())?;
    mgr.update(&id, &name, &color)
}

#[tauri::command]
fn update_profile_settings(id: String, settings: profiles::ProfileSettings, state: State<'_, profiles::ProfileState>) -> Result<(), String> {
    let mut mgr = state.lock().map_err(|e| e.to_string())?;
    mgr.update_settings(&id, settings)
}

#[tauri::command]
fn delete_profile(id: String, state: State<'_, profiles::ProfileState>) -> Result<(), String> {
    let mut mgr = state.lock().map_err(|e| e.to_string())?;
    mgr.delete(&id)
}

#[tauri::command]
fn set_default_profile(id: String, state: State<'_, profiles::ProfileState>) -> Result<(), String> {
    let mut mgr = state.lock().map_err(|e| e.to_string())?;
    mgr.set_default(&id)
}

#[tauri::command]
async fn open_profile_window(
    profile_id: String,
    profile_name: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use tauri::WebviewWindowBuilder;
    use tauri::WebviewUrl;

    let window_label = format!("profile-{}", &profile_id[..8.min(profile_id.len())]);

    if let Some(window) = app.get_webview_window(&window_label) {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let url = format!("index.html?profile={}", profile_id);

    WebviewWindowBuilder::new(
        &app,
        &window_label,
        WebviewUrl::App(url.into()),
    )
    .title(format!("Agentic IDE - {}", profile_name))
    .inner_size(1440.0, 900.0)
    .min_inner_size(1024.0, 600.0)
    .decorations(true)
    .title_bar_style(tauri::TitleBarStyle::Overlay)
    .hidden_title(true)
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn sync_tray_profiles(app: tauri::AppHandle) -> Result<(), String> {
    let profiles = {
        let mgr = app.state::<profiles::ProfileState>();
        let mgr = mgr.lock().map_err(|e| e.to_string())?;
        mgr.list()
    };

    let mut items: Vec<MenuItem<tauri::Wry>> = Vec::new();

    let open_item = MenuItem::with_id(&app, "open", "Open Agentic IDE", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    items.push(open_item);

    let sep = MenuItem::with_id(&app, "sep", "──────────", false, None::<&str>)
        .map_err(|e| e.to_string())?;
    items.push(sep);

    let mut profile_entries = Vec::new();
    for profile in &profiles {
        let id = format!("profile_{}", profile.id);
        let label = if profile.is_default {
            format!("{} (Default)", profile.name)
        } else {
            profile.name.clone()
        };
        let item = MenuItem::with_id(&app, &id, &label, true, None::<&str>)
            .map_err(|e| e.to_string())?;
        items.push(item);
        profile_entries.push((profile.id.clone(), profile.name.clone()));
    }

    let sep2 = MenuItem::with_id(&app, "sep2", "──────────", false, None::<&str>)
        .map_err(|e| e.to_string())?;
    items.push(sep2);

    let quit_item = MenuItem::with_id(&app, "quit", "Quit", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    items.push(quit_item);

    let item_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = items
        .iter()
        .map(|i| i as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
        .collect();
    let menu = Menu::with_items(&app, &item_refs).map_err(|e| e.to_string())?;

    let tray_state = app.state::<Mutex<TrayState>>();
    let mut state = tray_state.lock().map_err(|e| e.to_string())?;
    if let Some(ref tray) = state.icon {
        tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
    }
    state.profile_paths = profile_entries;

    Ok(())
}

// --- App Setup ---

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(Mutex::new(terminal::TerminalManager::new()) as terminal::TerminalState)
        .manage(Mutex::new(TrayState { icon: None, profile_paths: Vec::new() }))
        .invoke_handler(tauri::generate_handler![
            create_terminal,
            write_terminal,
            resize_terminal,
            close_terminal,
            list_worktrees,
            get_project_info,
            add_worktree,
            remove_worktree,
            get_diff,
            get_staged_diff,
            get_git_log,
            get_git_status,
            get_file_diff,
            stage_file,
            unstage_file,
            read_file_base64,
            read_git_file_base64,
            list_profiles,
            get_profile,
            get_default_profile,
            create_profile,
            update_profile,
            update_profile_settings,
            delete_profile,
            set_default_profile,
            open_profile_window,
            sync_tray_profiles,
        ])
        .setup(|app| {
            // Initialize profile manager
            let app_data_dir = app.path().app_data_dir().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            let profile_manager = profiles::ProfileManager::load(app_data_dir);
            app.manage(Mutex::new(profile_manager) as profiles::ProfileState);

            // Build tray menu
            let open_item = MenuItem::with_id(app, "open", "Open Agentic IDE", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_item, &quit_item])?;

            // Create tray icon (monochrome template for macOS)
            let tray_icon_bytes = include_bytes!("../icons/tray.png");
            let tray_image = tauri::image::Image::from_bytes(tray_icon_bytes)?.to_owned();

            let tray = TrayIconBuilder::new()
                .icon(tray_image)
                .icon_as_template(true)
                .tooltip("Agentic IDE")
                .menu(&menu)
                .on_menu_event(|app, event| {
                    let event_id = event.id.as_ref().to_string();
                    match event_id.as_str() {
                        "open" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                            let app_handle = app.clone();
                            std::thread::spawn(move || {
                                std::thread::sleep(std::time::Duration::from_secs(2));
                                let _ = app_handle.emit("check-for-updates", ());
                            });
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        id if id.starts_with("profile_") => {
                            let profile_id = id.strip_prefix("profile_").unwrap_or("").to_string();
                            // Look up profile name from tray state
                            let profile_name = {
                                let tray_state = app.state::<Mutex<TrayState>>();
                                tray_state.lock().ok()
                                    .and_then(|g| g.profile_paths.iter()
                                        .find(|(pid, _)| *pid == profile_id)
                                        .map(|(_, name)| name.clone()))
                                    .unwrap_or_else(|| "Profile".to_string())
                            };
                            // Open profile in new window
                            let app_clone = app.clone();
                            tauri::async_runtime::spawn(async move {
                                let _ = open_profile_window(profile_id, profile_name, app_clone).await;
                            });
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Store tray handle for dynamic menu updates
            if let Ok(mut tray_state) = app.state::<Mutex<TrayState>>().lock() {
                tray_state.icon = Some(tray);
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
