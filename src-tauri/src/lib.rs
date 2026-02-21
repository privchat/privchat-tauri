mod bridge_types;
mod commands;
mod errors;
mod privchat_state;

use crate::privchat_state::PrivchatAppState;
use std::fs;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(move |app| {
            let app_data_dir = app.path().app_data_dir()?;
            fs::create_dir_all(&app_data_dir)?;

            let server_url = std::env::var("PRIVCHAT_SERVER_URL")
                .unwrap_or_else(|_| "tcp://127.0.0.1:9001".to_string());

            app.manage(PrivchatAppState::new(app_data_dir, server_url));
            Ok(())
        })
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::exists_token,
            commands::password_login,
            commands::password_register,
            commands::resume_session,
            commands::start_server,
            commands::stop_server,
            commands::open_signup_url,
            commands::open_auth_url,
            commands::get_user_profile,
            commands::get_user_profile_by_id,
            commands::get_relations,
            commands::get_conversations,
            commands::upload_user_avatar_blob,
            commands::upload_image_blob,
            commands::upload_group_avatar_blob,
            commands::update_user_nickname,
            commands::remove_user_avatar,
            commands::add_friend,
            commands::remove_friend,
            commands::update_friend_display_name,
            commands::block_user,
            commands::unblock_user,
            commands::hide_blocked_user,
            commands::generate_local_message_id,
            commands::send_message,
            commands::read_latest_messages,
            commands::read_messages_before,
            commands::mark_last_read_message_id,
            commands::create_group,
            commands::invite_members,
            commands::get_group_members,
            commands::update_group_name,
            commands::leave_group,
            commands::upload_attachment,
            commands::logout,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
