import { ref } from 'vue';
import type { Message, MessageUpdateEvent, UIMessageItem } from '../../types/chat';
import { MessageAction, sortMessagesByMessageId } from '../../types/chat';

const sharedMessagesByConversation: Record<string, Message[]> = {};

/**
 * Composable for managing messages state
 *
 * Maintains messages grouped by conversationId
 * Handles message update events from backend
 *
 * @returns State and update handler
 */
export function useMessagesState() {
  // Messages grouped by conversationId
  const messagesByConversation = ref<Record<string, Message[]>>(sharedMessagesByConversation);

  /**
   * Convert UIMessageItem from WebSocket to Message format for storage
   */
  function convertUIMessageToMessage(uiMessage: UIMessageItem): Message {
    return {
      messageId: uiMessage.messageId,
      localMessageId: uiMessage.localMessageId,
      conversationId: uiMessage.conversationId,
      senderId: uiMessage.senderId,
      sendTimestamp: uiMessage.timestamp,
      messageType: uiMessage.messageType ?? 1, // Default to single message if not provided
      text: uiMessage.content,
      commandType: uiMessage.commandType,
      commandData: uiMessage.commandData,
      fileUrl: uiMessage.fileUrl,
      fileName: uiMessage.fileName,
    };
  }

  /**
   * Handle a message update event from the backend
   */
  function handleEvent(event: MessageUpdateEvent): void {
    const { action, message: uiMessage } = event;

    // CLEAR action - reset everything
    if (action === MessageAction.CLEAR) {
      messagesByConversation.value = {};
      console.log('[useMessagesState] Cleared all messages');
      return;
    }

    // All other actions require message data
    if (!uiMessage) {
      console.warn('[useMessagesState] Received event without message data:', action);
      return;
    }

    // Convert UIMessageItem to Message format
    const message = convertUIMessageToMessage(uiMessage);

    switch (action) {
      case MessageAction.ADD:
        addMessage(message);
        console.log('[useMessagesState] Added message:', message.messageId);
        break;

      case MessageAction.UPDATE:
        updateMessage(message);
        console.log('[useMessagesState] Updated message:', message.messageId);
        break;

      case MessageAction.DELETE:
        deleteMessage(message.conversationId, message.messageId);
        console.log('[useMessagesState] Deleted message:', message.messageId);
        break;

      default:
        console.warn('[useMessagesState] Unknown action:', action);
    }
  }

  /**
   * Load messages for a specific conversation
   */
  function loadMessages(conversationId: string, messages: Message[]): void {
    const existing = messagesByConversation.value[conversationId] || [];
    messagesByConversation.value[conversationId] = sortMessagesByMessageId(
      mergeMessages(existing, messages)
    );
    console.log(
      `[useMessagesState] Loaded ${messagesByConversation.value[conversationId].length} messages for ${conversationId}`
    );
  }

  /**
   * Get messages for a specific conversation
   */
  function getMessages(conversationId: string): Message[] {
    return messagesByConversation.value[conversationId] || [];
  }

  /**
   * Add a new message to its conversation
   */
  function addMessage(message: Message): void {
    const { conversationId } = message;
    const messages = messagesByConversation.value[conversationId] || [];

    // Replace optimistic placeholder with the real row id when localMessageId is available.
    if (
      message.localMessageId &&
      message.localMessageId !== message.messageId
    ) {
      const placeholderIndex = messages.findIndex(
        (m) => m.messageId === message.localMessageId
      );
      if (placeholderIndex >= 0) {
        messages.splice(placeholderIndex, 1);
      }
    }

    // Check if message already exists
    const exists = messages.some((m) => m.messageId === message.messageId);
    if (exists) {
      console.warn('[useMessagesState] Message already exists:', message.messageId);
      return;
    }

    // Guard against backend double-delivery of the same outbound message
    // with different message IDs in a very short time window.
    const probableEchoExists = messages.some((m) => isProbableEchoDuplicate(m, message));
    if (probableEchoExists) {
      console.warn('[useMessagesState] Probable duplicate echo ignored:', message.messageId);
      return;
    }

    // Add and sort
    messages.push(message);
    messagesByConversation.value[conversationId] = sortMessagesByMessageId(messages);
  }

  /**
   * Update an existing message
   */
  function updateMessage(message: Message): void {
    const { conversationId, messageId } = message;
    const messages = messagesByConversation.value[conversationId];

    if (!messages) {
      console.warn('[useMessagesState] Conversation not found:', conversationId);
      return;
    }

    const index = messages.findIndex((m) => m.messageId === messageId);
    if (index >= 0) {
      messages.splice(index, 1, message);
    } else {
      console.warn('[useMessagesState] Message not found:', messageId);
    }
  }

  /**
   * Delete a message
   */
  function deleteMessage(conversationId: string, messageId: string): void {
    const messages = messagesByConversation.value[conversationId];

    if (!messages) {
      console.warn('[useMessagesState] Conversation not found:', conversationId);
      return;
    }

    const index = messages.findIndex((m) => m.messageId === messageId);
    if (index >= 0) {
      messages.splice(index, 1);
    }
  }

  /**
   * Prepend older messages to the beginning of the message list
   * Used for pagination when loading historical messages
   */
  function prependMessages(conversationId: string, messages: Message[]): void {
    const existing = messagesByConversation.value[conversationId] || [];

    // Filter out duplicates
    const existingIds = new Set(existing.map((m) => m.messageId));
    const newMessages = messages.filter((m) => !existingIds.has(m.messageId));

    if (newMessages.length === 0) {
      console.log(`[useMessagesState] No new messages to prepend for ${conversationId}`);
      return;
    }

    // Combine and sort, then run dedupe again for backend echo duplicates.
    const combined = mergeMessages(existing, newMessages);
    messagesByConversation.value[conversationId] = sortMessagesByMessageId(combined);

    console.log(
      `[useMessagesState] Prepended ${newMessages.length} older messages for ${conversationId}`
    );
  }

  /**
   * Get the oldest message ID for a conversation (used for pagination)
   */
  function getOldestMessageId(conversationId: string): string | null {
    const messages = messagesByConversation.value[conversationId];
    if (!messages || messages.length === 0) {
      return null;
    }
    // Messages are sorted from old to new, so first message is oldest
    return messages[0].messageId;
  }

  /**
   * Clear messages for a specific conversation
   */
  function clearConversationMessages(conversationId: string): void {
    delete messagesByConversation.value[conversationId];
  }

  function addOptimisticMessage(message: Message): void {
    addMessage(message);
  }

  function removeMessageById(conversationId: string, messageId: string): void {
    deleteMessage(conversationId, messageId);
  }

  function replaceMessageId(
    conversationId: string,
    fromMessageId: string,
    toMessageId: string
  ): void {
    const messages = messagesByConversation.value[conversationId];
    if (!messages || fromMessageId === toMessageId) return;

    const fromIdx = messages.findIndex((m) => m.messageId === fromMessageId);
    if (fromIdx < 0) return;

    const targetExists = messages.some((m) => m.messageId === toMessageId);
    if (targetExists) {
      messages.splice(fromIdx, 1);
      messagesByConversation.value[conversationId] = sortMessagesByMessageId(messages);
      return;
    }

    const replaced: Message = {
      ...messages[fromIdx],
      messageId: toMessageId,
      localMessageId: fromMessageId,
    };
    messages.splice(fromIdx, 1, replaced);
    messagesByConversation.value[conversationId] = sortMessagesByMessageId(messages);
  }

  return {
    messagesByConversation,
    handleEvent,
    loadMessages,
    getMessages,
    prependMessages,
    getOldestMessageId,
    clearConversationMessages,
    addOptimisticMessage,
    removeMessageById,
    replaceMessageId,
  };
}

const ECHO_DEDUPE_WINDOW_MS = 3000;

function isProbableEchoDuplicate(existing: Message, incoming: Message): boolean {
  if (existing.messageId === incoming.messageId) return true;
  if (existing.conversationId !== incoming.conversationId) return false;
  if (existing.senderId !== incoming.senderId) return false;
  if ((existing.text || '') !== (incoming.text || '')) return false;
  if ((existing.fileUrl || '') !== (incoming.fileUrl || '')) return false;
  if ((existing.fileName || '') !== (incoming.fileName || '')) return false;
  if ((existing.messageType || 1) !== (incoming.messageType || 1)) return false;

  const delta = Math.abs((existing.sendTimestamp || 0) - (incoming.sendTimestamp || 0));
  return delta <= ECHO_DEDUPE_WINDOW_MS;
}

function dedupeProbableEchoMessages(messages: Message[]): Message[] {
  const deduped: Message[] = [];
  for (const msg of messages) {
    const duplicate = deduped.some((existing) => isProbableEchoDuplicate(existing, msg));
    if (!duplicate) {
      deduped.push(msg);
    }
  }
  return deduped;
}

function mergeMessages(existing: Message[], incoming: Message[]): Message[] {
  const merged = new Map<string, Message>();

  // First pass: if incoming includes real message id with localMessageId, drop placeholder.
  for (const msg of incoming) {
    if (msg.localMessageId && msg.localMessageId !== msg.messageId) {
      merged.delete(msg.localMessageId);
    }
  }

  for (const msg of [...existing, ...incoming]) {
    if (msg.localMessageId && msg.localMessageId !== msg.messageId) {
      merged.delete(msg.localMessageId);
    }
    const old = merged.get(msg.messageId);
    if (!old) {
      merged.set(msg.messageId, msg);
      continue;
    }
    // Prefer the newer/richer version when same messageId appears.
    merged.set(msg.messageId, choosePreferredMessage(old, msg));
  }

  return dedupeProbableEchoMessages(Array.from(merged.values()));
}

function choosePreferredMessage(a: Message, b: Message): Message {
  const score = (m: Message): number => {
    let s = 0;
    if (m.text && m.text.trim().length > 0) s += 2;
    if (m.fileUrl && m.fileUrl.trim().length > 0) s += 2;
    if (m.fileName && m.fileName.trim().length > 0) s += 1;
    if (m.commandData && m.commandData.trim().length > 0) s += 1;
    if (m.sendTimestamp > 0) s += 1;
    return s;
  };

  const scoreA = score(a);
  const scoreB = score(b);
  if (scoreA !== scoreB) {
    return scoreA > scoreB ? a : b;
  }
  return a.sendTimestamp >= b.sendTimestamp ? a : b;
}
