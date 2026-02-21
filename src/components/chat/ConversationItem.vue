<template>
  <div
    class="flex items-center gap-2.5 px-4 py-2.5 hover:bg-gray-50 cursor-pointer transition-colors"
    @click="handleClick"
  >
    <!-- Avatar -->
    <img
      :src="avatarUrl"
      @error="onImageError"
      alt="Avatar"
      class="w-10 h-10 rounded-lg flex-shrink-0 object-cover"
    />

    <!-- Content -->
    <div class="flex-1 min-w-0">
      <div class="flex items-center justify-between">
        <!-- Name -->
        <div class="font-medium text-sm text-gray-900 truncate">
          {{ displayName }}
        </div>

        <!-- Time -->
        <div class="text-[11px] text-gray-500 flex-shrink-0 ml-2">
          {{ formattedTime }}
        </div>
      </div>

      <div class="flex items-center justify-between">
        <!-- Last Message Preview -->
        <div class="text-xs text-gray-500 truncate flex-1">
          {{ conversation.lastMessage || '暂无消息' }}
        </div>

        <!-- Unread Badge -->
        <div
          v-if="conversation.unreadCount > 0"
          class="ml-2 flex-shrink-0 bg-blue-500 text-white text-[10px] font-medium rounded-full min-w-4 h-4 px-1 flex items-center justify-center"
        >
          {{ conversation.unreadCount > 99 ? '99+' : conversation.unreadCount }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { ConversationDisplay } from '../../types/chat';
import { getConversationDisplayName, getConversationAvatar, isGroupChat } from '../../types/chat';
import { formatConversationTime } from '../../utils/dateFormat';
import { useGroupMembersCache } from '../../composables/chat/useGroupMembersCache';
import defaultAvatarUrl from '../../assets/default-avatar.svg';

const props = defineProps<{
  conversation: ConversationDisplay;
}>();

const emit = defineEmits<{
  click: [conversation: ConversationDisplay];
}>();

const { getGroupMemberCount } = useGroupMembersCache();

const displayName = computed(() => {
  const baseName = getConversationDisplayName(props.conversation);

  // For group chats, append member count if available
  if (isGroupChat(props.conversation) && props.conversation.groupId) {
    const memberCount = getGroupMemberCount(props.conversation.groupId);
    if (memberCount !== undefined) {
      return `${baseName}(${memberCount})`;
    }
  }

  return baseName;
});

const avatarUrl = computed(() => {
  const avatar = getConversationAvatar(props.conversation);
  if (!avatar) return defaultAvatarUrl;
  if (avatar.startsWith('http://') || avatar.startsWith('https://')) {
    return avatar;
  }
  return `asset://localhost/${avatar}`;
});

const formattedTime = computed(() => {
  if (!props.conversation.lastMessageTimestamp) {
    return '';
  }
  return formatConversationTime(props.conversation.lastMessageTimestamp);
});

function onImageError(event: Event) {
  const img = event.target as HTMLImageElement;
  img.src = defaultAvatarUrl;
}

function handleClick() {
  emit('click', props.conversation);
}
</script>
