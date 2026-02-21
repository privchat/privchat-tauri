use crate::bridge_types::{
    GroupMemberData, MessageItem, ReadMessagesData, RelationUser, RelationUsers, UIConversationItem,
    UIConversations, UserProfileData,
};
use crate::errors;
use crate::privchat_state::{PrivchatAppState, SavedSession};
use anyhow::anyhow;
use base64::{engine::general_purpose::STANDARD, Engine};
use privchat_sdk_ffi::{ProfileUpdateInput, StoredMessage};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;
use uuid::Uuid;

const RELATION_FRIEND: i32 = 0b0001;
const RELATION_BLOCKED: i32 = 0b0010;

#[derive(Clone, Serialize)]
struct AuthResultEvent {
    success: bool,
    message: String,
}

#[tauri::command]
pub async fn exists_token(state: State<'_, PrivchatAppState>) -> Result<bool, errors::CommandError> {
    Ok(state.load_session()?.is_some())
}

#[tauri::command]
pub async fn password_login(
    app: AppHandle,
    username: String,
    password: String,
    server_url: Option<String>,
    device_id: Option<String>,
) -> Result<(), errors::CommandError> {
    let state = app.state::<PrivchatAppState>();
    let client = state.get_or_create_client().await?;

    if let Some(url) = server_url {
        if url != state.server_url {
            app.emit(
                "auth-result",
                AuthResultEvent {
                    success: false,
                    message: format!("server_url is fixed at startup: {}", state.server_url),
                },
            )
            .ok();
            return Err(anyhow!("server_url mismatch").into());
        }
    }

    client.connect().await.map_err(|e| anyhow!(e.to_string()))?;

    let dev = device_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let login = client
        .login(username.clone(), password, dev.clone())
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    client
        .authenticate(login.user_id, login.token.clone(), dev.clone())
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    client
        .run_bootstrap_sync()
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    let _ = client.sync_all_channels().await;

    state.save_session(&SavedSession {
        user_id: login.user_id,
        token: login.token,
        device_id: dev,
        username: Some(username),
    })?;

    app.emit(
        "auth-result",
        AuthResultEvent {
            success: true,
            message: "ok".to_string(),
        },
    )
    .ok();

    Ok(())
}

#[tauri::command]
pub async fn password_register(
    app: AppHandle,
    username: String,
    password: String,
    server_url: Option<String>,
    device_id: Option<String>,
) -> Result<(), errors::CommandError> {
    let state = app.state::<PrivchatAppState>();
    let client = state.get_or_create_client().await?;

    if let Some(url) = server_url {
        if url != state.server_url {
            app.emit(
                "auth-result",
                AuthResultEvent {
                    success: false,
                    message: format!("server_url is fixed at startup: {}", state.server_url),
                },
            )
            .ok();
            return Err(anyhow!("server_url mismatch").into());
        }
    }

    client.connect().await.map_err(|e| anyhow!(e.to_string()))?;

    let dev = device_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let register = client
        .register(username.clone(), password, dev.clone())
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    client
        .authenticate(register.user_id, register.token.clone(), dev.clone())
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    client
        .run_bootstrap_sync()
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    let _ = client.sync_all_channels().await;

    state.save_session(&SavedSession {
        user_id: register.user_id,
        token: register.token,
        device_id: dev,
        username: Some(username),
    })?;

    app.emit(
        "auth-result",
        AuthResultEvent {
            success: true,
            message: "ok".to_string(),
        },
    )
    .ok();

    Ok(())
}

#[tauri::command]
pub async fn resume_session(app: AppHandle) -> Result<(), errors::CommandError> {
    let state = app.state::<PrivchatAppState>();
    let session = state
        .load_session()?
        .ok_or_else(|| anyhow!("no saved session"))?;

    let client = state.get_or_create_client().await?;
    client.connect().await.map_err(|e| anyhow!(e.to_string()))?;
    client
        .authenticate(session.user_id, session.token, session.device_id)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    client
        .run_bootstrap_sync()
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    let _ = client.sync_all_channels().await;
    Ok(())
}

#[tauri::command]
pub async fn start_server(_app: AppHandle) -> Result<(), errors::CommandError> {
    Ok(())
}

#[tauri::command]
pub async fn stop_server() -> Result<(), errors::CommandError> {
    Ok(())
}

#[tauri::command]
pub fn open_signup_url(app: AppHandle) -> Result<(), errors::CommandError> {
    Ok(app
        .opener()
        .open_url("https://github.com/netonstream/privchat".to_string(), None::<&str>)?)
}

#[tauri::command]
pub fn open_auth_url(app: AppHandle) -> Result<(), errors::CommandError> {
    Ok(app
        .opener()
        .open_url("https://github.com/netonstream/privchat".to_string(), None::<&str>)?)
}

#[tauri::command]
pub async fn get_user_profile(
    state: State<'_, PrivchatAppState>,
) -> Result<UserProfileData, errors::CommandError> {
    let client = state.get_or_create_client().await?;
    let session = client
        .session_snapshot()
        .await
        .map_err(|e| anyhow!(e.to_string()))?
        .ok_or_else(|| anyhow!("session not ready"))?;
    let user = client
        .get_user_by_id(session.user_id)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    Ok(UserProfileData {
        user_id: session.user_id.to_string(),
        nick_name: user
            .as_ref()
            .map(|u| pick_name(u.alias.as_ref(), u.nickname.as_ref(), u.username.as_ref()))
            .unwrap_or_else(|| session.user_id.to_string()),
        account: user
            .as_ref()
            .and_then(|u| u.username.clone().and_then(none_if_empty)),
        avatar: user.and_then(|u| none_if_empty(u.avatar)),
    })
}

#[tauri::command]
pub async fn upload_user_avatar_blob(
    _app: AppHandle,
    image_data: String,
) -> Result<(), errors::CommandError> {
    let _ = STANDARD
        .decode(image_data)
        .map_err(|e| anyhow!("invalid base64 image: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn upload_image_blob(
    _app: AppHandle,
    image_data: String,
) -> Result<String, errors::CommandError> {
    let image_bytes = STANDARD
        .decode(image_data)
        .map_err(|e| anyhow!("invalid base64 image: {e}"))?;
    let b64 = STANDARD.encode(image_bytes);
    Ok(format!("data:image/png;base64,{b64}"))
}

#[tauri::command]
pub async fn upload_group_avatar_blob(
    _app: AppHandle,
    _group_id: String,
    image_data: String,
) -> Result<(), errors::CommandError> {
    let _ = STANDARD
        .decode(image_data)
        .map_err(|e| anyhow!("invalid base64 image: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn update_user_nickname(
    nickname: String,
    state: State<'_, PrivchatAppState>,
) -> Result<(), errors::CommandError> {
    let client = state.get_or_create_client().await?;
    client
        .update_profile(ProfileUpdateInput {
            display_name: Some(nickname),
            avatar_url: None,
            bio: None,
        })
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn remove_user_avatar(
    state: State<'_, PrivchatAppState>,
) -> Result<(), errors::CommandError> {
    let client = state.get_or_create_client().await?;
    client
        .update_profile(ProfileUpdateInput {
            display_name: None,
            avatar_url: Some(String::new()),
            bio: None,
        })
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn add_friend(
    target_user_id: String,
    request_message: Option<String>,
    state: State<'_, PrivchatAppState>,
) -> Result<(), errors::CommandError> {
    let client = state.get_or_create_client().await?;
    let uid = parse_u64(&target_user_id, "target_user_id")?;
    let _ = client
        .send_friend_request(uid, request_message, None, None)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn remove_friend(
    friend_id: String,
    state: State<'_, PrivchatAppState>,
) -> Result<(), errors::CommandError> {
    let client = state.get_or_create_client().await?;
    let uid = parse_u64(&friend_id, "friend_id")?;
    let _ = client
        .delete_friend(uid)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn update_friend_display_name(
    _friend_id: String,
    _remark_name: String,
) -> Result<(), errors::CommandError> {
    Ok(())
}

#[tauri::command]
pub async fn block_user(
    target_user_id: String,
    state: State<'_, PrivchatAppState>,
) -> Result<(), errors::CommandError> {
    let client = state.get_or_create_client().await?;
    let uid = parse_u64(&target_user_id, "target_user_id")?;
    let _ = client
        .add_to_blacklist(uid)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn unblock_user(
    target_user_id: String,
    state: State<'_, PrivchatAppState>,
) -> Result<(), errors::CommandError> {
    let client = state.get_or_create_client().await?;
    let uid = parse_u64(&target_user_id, "target_user_id")?;
    let _ = client
        .remove_from_blacklist(uid)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn hide_blocked_user(
    _target_user_id: String,
) -> Result<(), errors::CommandError> {
    Ok(())
}

#[tauri::command]
pub async fn get_user_profile_by_id(
    user_id: String,
    state: State<'_, PrivchatAppState>,
) -> Result<UserProfileData, errors::CommandError> {
    let client = state.get_or_create_client().await?;
    let direct_uid = user_id.trim().parse::<u64>().ok();
    let (uid, fallback_name, fallback_account, fallback_avatar) = if let Some(uid) = direct_uid {
        (uid, None, None, None)
    } else {
        let keyword = user_id.trim().to_string();
        let mut users = client
            .search_users(keyword.clone())
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        if users.is_empty() {
            return Err(anyhow!("user not found by keyword: {}", keyword).into());
        }

        let exact_hit_idx = users.iter().position(|u| {
            u.username.eq_ignore_ascii_case(&keyword)
                || u.nickname.eq_ignore_ascii_case(&keyword)
                || u.user_id.to_string() == keyword
        });

        let hit = if let Some(idx) = exact_hit_idx {
            users.swap_remove(idx)
        } else {
            users.swap_remove(0)
        };

        (
            hit.user_id,
            Some(if hit.nickname.trim().is_empty() {
                hit.username.clone()
            } else {
                hit.nickname
            }),
            Some(hit.username),
            hit.avatar_url,
        )
    };

    let user = client
        .get_user_by_id(uid)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    Ok(UserProfileData {
        user_id: uid.to_string(),
        nick_name: user
            .as_ref()
            .map(|u| pick_name(u.alias.as_ref(), u.nickname.as_ref(), u.username.as_ref()))
            .or(fallback_name)
            .unwrap_or_else(|| uid.to_string()),
        account: user
            .as_ref()
            .and_then(|u| u.username.clone().and_then(none_if_empty))
            .or(fallback_account.and_then(none_if_empty)),
        avatar: user
            .and_then(|u| none_if_empty(u.avatar))
            .or(fallback_avatar.and_then(none_if_empty)),
    })
}

#[tauri::command]
pub async fn get_relations(
    state: State<'_, PrivchatAppState>,
) -> Result<RelationUsers, errors::CommandError> {
    let client = state.get_or_create_client().await?;
    let _ = client.run_bootstrap_sync().await;

    let mut friends = client
        .get_friends(2000, 0)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    if friends.is_empty() {
        let _ = client.sync_all_channels().await;
        friends = client
            .get_friends(2000, 0)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
    }

    let blacklist = client
        .get_blacklist()
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    let friend_ids: HashSet<u64> = friends.iter().map(|v| v.user_id).collect();
    let blocked_ids: HashSet<u64> = blacklist.iter().map(|v| v.blocked_user_id).collect();
    let mut all_ids: Vec<u64> = friend_ids.union(&blocked_ids).copied().collect();
    all_ids.sort_unstable();

    let users = if all_ids.is_empty() {
        vec![]
    } else {
        client
            .list_users_by_ids(all_ids)
            .await
            .map_err(|e| anyhow!(e.to_string()))?
    };

    let user_map: HashMap<u64, _> = users.into_iter().map(|u| (u.user_id, u)).collect();
    let friend_map: HashMap<u64, _> = friends.into_iter().map(|f| (f.user_id, f)).collect();

    let mut out = Vec::new();
    for uid in friend_ids.union(&blocked_ids).copied() {
        let _friend = friend_map.get(&uid);
        let user = user_map.get(&uid);

        let mut relation_flags = 0;
        if friend_ids.contains(&uid) {
            relation_flags |= RELATION_FRIEND;
        }
        if blocked_ids.contains(&uid) {
            relation_flags |= RELATION_BLOCKED;
        }

        out.push(RelationUser {
            user_id: uid.to_string(),
            nick_name: user
                .map(|u| pick_name(u.alias.as_ref(), u.nickname.as_ref(), u.username.as_ref()))
                .unwrap_or_else(|| uid.to_string()),
            avatar: user.and_then(|u| none_if_empty(u.avatar.clone())),
            remark_name: None,
            relation_flags,
        });
    }

    out.sort_by(|a, b| a.user_id.cmp(&b.user_id));
    Ok(RelationUsers { users: out })
}

#[tauri::command]
pub async fn get_conversations(
    state: State<'_, PrivchatAppState>,
) -> Result<UIConversations, errors::CommandError> {
    let client = state.get_or_create_client().await?;
    let channels = client
        .get_channels(2000, 0)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    let conversations = channels
        .into_iter()
        .map(|c| UIConversationItem {
            conversation_id: c.channel_id.to_string(),
            peer_id: (c.channel_type == 1)
                .then(|| derive_peer_id(&c.channel_name).unwrap_or_else(|| c.channel_id.to_string())),
            group_id: (c.channel_type != 1).then(|| c.channel_id.to_string()),
            last_message_id: (c.last_local_message_id > 0).then(|| c.last_local_message_id.to_string()),
            last_read_message_id: None,
            unread_count: c.unread_count as i64,
            last_message_text: none_if_empty(c.last_msg_content),
            last_message_timestamp: Some(c.last_msg_timestamp),
            name: if c.channel_name.trim().is_empty() {
                c.channel_id.to_string()
            } else {
                c.channel_name
            },
            avatar: none_if_empty(c.avatar),
        })
        .collect();

    Ok(UIConversations { conversations })
}

#[tauri::command]
pub async fn generate_local_message_id(
    state: State<'_, PrivchatAppState>,
) -> Result<String, errors::CommandError> {
    let client = state.get_or_create_client().await?;
    let id = client
        .generate_local_message_id()
        .map_err(|e| anyhow!(e.to_string()))?;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn send_message(
    _app: AppHandle,
    sender_id: String,
    conversation_id: String,
    receiver_id: Option<String>,
    group_id: Option<String>,
    local_message_id: Option<String>,
    text: Option<String>,
    file_url: Option<String>,
    file_name: Option<String>,
    state: State<'_, PrivchatAppState>,
) -> Result<String, errors::CommandError> {
    let client = state.get_or_create_client().await?;
    let channel_id = parse_u64(&conversation_id, "conversation_id")?;
    let channel_type = guess_channel_type(group_id.as_ref(), receiver_id.as_ref(), &client, channel_id).await;
    let from_uid = parse_u64(&sender_id, "sender_id")?;
    let content = text
        .or_else(|| file_name.clone())
        .or_else(|| file_url.clone())
        .unwrap_or_default();

    let local_id = match local_message_id.as_deref() {
        Some(v) if !v.trim().is_empty() => Some(parse_u64(v, "local_message_id")?),
        _ => None,
    };

    let message_id = client
        .enqueue_text_with_local_id(channel_id, channel_type, from_uid, content, local_id)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    Ok(message_id.to_string())
}

#[tauri::command]
pub async fn read_latest_messages(
    conversation_id: String,
    read_size: u32,
    _last_read_message_id: String,
    state: State<'_, PrivchatAppState>,
) -> Result<ReadMessagesData, errors::CommandError> {
    let client = state.get_or_create_client().await?;
    let channel_id = parse_u64(&conversation_id, "conversation_id")?;
    let channel_type = guess_channel_type(None, None, &client, channel_id).await;
    let mut rows = client
        .get_messages(channel_id, channel_type, read_size.min(200) as u64, 0)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    rows.reverse();

    Ok(ReadMessagesData {
        messages: rows.into_iter().map(|m| map_message(&conversation_id, m)).collect(),
    })
}

#[tauri::command]
pub async fn read_messages_before(
    conversation_id: String,
    before_message_id: String,
    read_size: u32,
    state: State<'_, PrivchatAppState>,
) -> Result<ReadMessagesData, errors::CommandError> {
    let client = state.get_or_create_client().await?;
    let channel_id = parse_u64(&conversation_id, "conversation_id")?;
    let channel_type = guess_channel_type(None, None, &client, channel_id).await;
    let before = parse_u64(&before_message_id, "before_message_id")?;

    let rows = client
        .get_messages(channel_id, channel_type, 1000, 0)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    let mut filtered: Vec<_> = rows
        .into_iter()
        .filter(|m| m.message_id < before)
        .take(read_size.min(200) as usize)
        .collect();
    filtered.reverse();

    Ok(ReadMessagesData {
        messages: filtered
            .into_iter()
            .map(|m| map_message(&conversation_id, m))
            .collect(),
    })
}

#[tauri::command]
pub async fn mark_last_read_message_id(
    conversation_id: String,
    message_id: String,
    state: State<'_, PrivchatAppState>,
) -> Result<(), errors::CommandError> {
    let client = state.get_or_create_client().await?;
    let channel_id = parse_u64(&conversation_id, "conversation_id")?;
    let channel_type = guess_channel_type(None, None, &client, channel_id).await;

    let before = client
        .get_channel_by_id(channel_id)
        .await
        .ok()
        .flatten()
        .map(|c| (c.channel_type, c.unread_count, c.last_local_message_id));
    println!(
        "[mark_last_read_message_id] req conversation_id={} message_id={} resolved_channel_type={} before={:?}",
        conversation_id, message_id, channel_type, before
    );

    client
        .mark_channel_read(channel_id, channel_type)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    let after = client
        .get_channel_by_id(channel_id)
        .await
        .ok()
        .flatten()
        .map(|c| (c.channel_type, c.unread_count, c.last_local_message_id));
    println!(
        "[mark_last_read_message_id] done conversation_id={} message_id={} after={:?}",
        conversation_id, message_id, after
    );
    Ok(())
}

#[tauri::command]
pub async fn create_group(
    _sender_id: String,
    group_name: String,
    member_ids: Vec<String>,
    state: State<'_, PrivchatAppState>,
) -> Result<String, errors::CommandError> {
    let client = state.get_or_create_client().await?;
    let ids = member_ids
        .into_iter()
        .map(|v| parse_u64(&v, "member_id"))
        .collect::<Result<Vec<_>, _>>()?;

    let group = client
        .create_group(group_name, None, Some(ids))
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    Ok(group.group_id.to_string())
}

#[tauri::command]
pub async fn invite_members(
    group_id: String,
    _sender_id: String,
    member_ids: Vec<String>,
    _group_name: String,
    _group_avatar: Option<String>,
    state: State<'_, PrivchatAppState>,
) -> Result<(), errors::CommandError> {
    let client = state.get_or_create_client().await?;
    let gid = parse_u64(&group_id, "group_id")?;
    let ids = member_ids
        .into_iter()
        .map(|v| parse_u64(&v, "member_id"))
        .collect::<Result<Vec<_>, _>>()?;
    let _ = client
        .invite_to_group(gid, ids)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn get_group_members(
    group_id: String,
    state: State<'_, PrivchatAppState>,
) -> Result<Vec<GroupMemberData>, errors::CommandError> {
    let client = state.get_or_create_client().await?;
    let gid = parse_u64(&group_id, "group_id")?;
    let members = client
        .get_group_members(gid, 1000, 0)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    let user_ids: Vec<u64> = members.iter().map(|m| m.user_id).collect();
    let users = if user_ids.is_empty() {
        vec![]
    } else {
        client
            .list_users_by_ids(user_ids)
            .await
            .map_err(|e| anyhow!(e.to_string()))?
    };
    let user_map: HashMap<u64, _> = users.into_iter().map(|u| (u.user_id, u)).collect();

    Ok(members
        .into_iter()
        .map(|m| {
            let user = user_map.get(&m.user_id);
            GroupMemberData {
                user_id: m.user_id.to_string(),
                name: user
                    .map(|u| pick_name(u.alias.as_ref(), u.nickname.as_ref(), u.username.as_ref()))
                    .unwrap_or_else(|| m.user_id.to_string()),
                avatar: user.and_then(|u| none_if_empty(u.avatar.clone())),
            }
        })
        .collect())
}

#[tauri::command]
pub async fn update_group_name(
    _group_id: String,
    _sender_id: String,
    _group_name: String,
) -> Result<(), errors::CommandError> {
    Ok(())
}

#[tauri::command]
pub async fn leave_group(
    group_id: String,
    state: State<'_, PrivchatAppState>,
) -> Result<(), errors::CommandError> {
    let client = state.get_or_create_client().await?;
    let gid = parse_u64(&group_id, "group_id")?;
    let _ = client
        .leave_group(gid)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    Ok(())
}

#[derive(Clone, Serialize)]
pub struct UploadAttachmentResponse {
    pub file_url: String,
}

#[tauri::command]
pub async fn upload_attachment(
    _app: AppHandle,
    file_path: String,
) -> Result<UploadAttachmentResponse, errors::CommandError> {
    Ok(UploadAttachmentResponse {
        file_url: format!("file://{}", file_path),
    })
}

#[tauri::command]
pub async fn logout(state: State<'_, PrivchatAppState>) -> Result<(), errors::CommandError> {
    if let Ok(client) = state.get_or_create_client().await {
        let _ = client.logout().await;
    }
    state.clear_session()?;
    state.clear_client().await;
    Ok(())
}

fn derive_peer_id(channel_name: &str) -> Option<String> {
    let candidate = channel_name.trim();
    if candidate.is_empty() {
        return None;
    }
    if candidate.parse::<u64>().is_ok() {
        return Some(candidate.to_string());
    }
    None
}

fn parse_u64(value: &str, field: &str) -> Result<u64, errors::CommandError> {
    value
        .parse::<u64>()
        .map_err(|e| anyhow!("invalid {}: {} ({})", field, value, e).into())
}

fn none_if_empty(v: String) -> Option<String> {
    if v.trim().is_empty() {
        None
    } else {
        Some(v)
    }
}

fn pick_name(alias: Option<&String>, nickname: Option<&String>, username: Option<&String>) -> String {
    alias
        .and_then(|v| if v.trim().is_empty() { None } else { Some(v.clone()) })
        .or_else(|| nickname.and_then(|v| if v.trim().is_empty() { None } else { Some(v.clone()) }))
        .or_else(|| username.and_then(|v| if v.trim().is_empty() { None } else { Some(v.clone()) }))
        .unwrap_or_else(|| "unknown".to_string())
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn map_message(conversation_id: &str, m: StoredMessage) -> MessageItem {
    MessageItem {
        conversation_id: conversation_id.to_string(),
        message_id: m.message_id.to_string(),
        sender_id: m.from_uid.to_string(),
        receiver_id: None,
        group_id: (m.channel_type != 1).then(|| m.channel_id.to_string()),
        send_timestamp: m.created_at.to_string(),
        message_type: if m.message_type == 2 { 2 } else { 1 },
        text: Some(m.content),
        local_message_id: m.local_message_id.map(|v| v.to_string()),
        file_url: None,
        file_name: None,
        command_type: 0,
        command_data: None,
    }
}

async fn guess_channel_type(
    group_id: Option<&String>,
    receiver_id: Option<&String>,
    client: &privchat_sdk_ffi::PrivchatClient,
    channel_id: u64,
) -> i32 {
    if group_id.is_some() {
        return 2;
    }
    if receiver_id.is_some() {
        return 1;
    }
    match client.get_channel_by_id(channel_id).await {
        Ok(Some(ch)) => ch.channel_type,
        _ => 1,
    }
}
