use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UserProfileData {
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "nickName")]
    pub nick_name: String,
    #[serde(rename = "account", skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RelationUser {
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "nickName")]
    pub nick_name: String,
    pub avatar: Option<String>,
    #[serde(rename = "remarkName")]
    pub remark_name: Option<String>,
    #[serde(rename = "relationFlags")]
    pub relation_flags: i32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RelationUsers {
    pub users: Vec<RelationUser>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GroupMemberData {
    #[serde(rename = "userId")]
    pub user_id: String,
    pub name: String,
    pub avatar: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UIConversationItem {
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    #[serde(rename = "peerId")]
    pub peer_id: Option<String>,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    #[serde(rename = "lastMessageId")]
    pub last_message_id: Option<String>,
    #[serde(rename = "lastReadMessageId")]
    pub last_read_message_id: Option<String>,
    #[serde(rename = "unreadCount")]
    pub unread_count: i64,
    #[serde(rename = "lastMessage")]
    pub last_message_text: Option<String>,
    #[serde(rename = "lastMessageTimestamp")]
    pub last_message_timestamp: Option<i64>,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "avatar")]
    pub avatar: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UIConversations {
    pub conversations: Vec<UIConversationItem>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MessageItem {
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "senderId")]
    pub sender_id: String,
    #[serde(rename = "receiverId")]
    pub receiver_id: Option<String>,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    #[serde(rename = "sendTimestamp")]
    pub send_timestamp: String,
    #[serde(rename = "messageType")]
    pub message_type: i32,
    #[serde(rename = "text")]
    pub text: Option<String>,
    #[serde(rename = "localMessageId", skip_serializing_if = "Option::is_none")]
    pub local_message_id: Option<String>,
    #[serde(rename = "fileUrl")]
    pub file_url: Option<String>,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(rename = "commandType")]
    pub command_type: i32,
    #[serde(rename = "commandData")]
    pub command_data: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ReadMessagesData {
    pub messages: Vec<MessageItem>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UIMessageItem {
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    #[serde(rename = "senderId")]
    pub sender_id: String,
    #[serde(rename = "groupId", skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    pub content: String,
    pub timestamp: i64,
    #[serde(rename = "messageType")]
    pub message_type: i32,
    #[serde(rename = "commandType", skip_serializing_if = "Option::is_none")]
    pub command_type: Option<i32>,
    #[serde(rename = "commandData", skip_serializing_if = "Option::is_none")]
    pub command_data: Option<String>,
    #[serde(rename = "fileUrl", skip_serializing_if = "Option::is_none")]
    pub file_url: Option<String>,
    #[serde(rename = "fileName", skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MessageUpdateEvent {
    pub action: i32,
    pub message: Option<UIMessageItem>,
}
