pub mod commands;
pub mod connection;
pub mod dialect;
pub mod error;
pub mod query;
pub mod safety;
pub mod schema;

use crate::commands::AppState;
use crate::connection::config::ConfigStore;
use crate::connection::pool::PoolManager;
use tauri::menu::{Menu, PredefinedMenuItem, Submenu};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            app.manage(AppState {
                store: ConfigStore::new(&config_dir),
                pools: PoolManager::new(),
            });

            // 自定义菜单：不含 Cmd+W Close，让快捷键落到 webview；保留 Edit 使复制粘贴可用
            let app_menu = Submenu::with_items(
                app,
                "DB-Minus",
                true,
                &[
                    &PredefinedMenuItem::about(app, None, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::hide(app, None)?,
                    &PredefinedMenuItem::quit(app, None)?,
                ],
            )?;
            let edit_menu = Submenu::with_items(
                app,
                "Edit",
                true,
                &[
                    &PredefinedMenuItem::undo(app, None)?,
                    &PredefinedMenuItem::redo(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::cut(app, None)?,
                    &PredefinedMenuItem::copy(app, None)?,
                    &PredefinedMenuItem::paste(app, None)?,
                    &PredefinedMenuItem::select_all(app, None)?,
                ],
            )?;
            let menu = Menu::with_items(app, &[&app_menu, &edit_menu])?;
            app.set_menu(menu)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::connections_list,
            commands::connection_save,
            commands::connection_delete,
            commands::connection_test,
            commands::list_namespaces,
            commands::list_tables,
            commands::list_columns,
            commands::fetch_table_page,
            commands::execute_sql,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
