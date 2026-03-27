mod terminal;
mod worktree;

use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State, WindowEvent,
};
use tauri_plugin_autostart::MacosLauncher;

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
        ])
        .setup(|app| {
            // Build tray menu
            let open_item = MenuItem::with_id(app, "open", "Open Agentic IDE", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_item, &quit_item])?;

            // Create tray icon (monochrome template for macOS)
            let tray_icon_bytes = include_bytes!("../icons/tray.png");
            let tray_image = tauri::image::Image::from_bytes(tray_icon_bytes)?.to_owned();

            TrayIconBuilder::new()
                .icon(tray_image)
                .icon_as_template(true)
                .tooltip("Agentic IDE")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
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
                    _ => {}
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

            Ok(())
        })
        .on_window_event(|window, event| {
            // Hide to tray instead of closing
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
