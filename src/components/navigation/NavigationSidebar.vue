<template>
  <div class="h-full w-sidebar bg-sidebar border-r border-border-sidebar flex flex-col">
    <!-- Logo/Header -->
    <div class="h-header flex items-center justify-center border-b border-border-sidebar px-2">
      <button
        class="w-10 h-10 rounded-xl overflow-hidden border border-border-sidebar/60 hover:border-primary/70 transition-colors"
        :title="userProfile?.nickName || 'Profile'"
        @click="handleOpenProfile"
      >
        <img
          :src="avatarUrl"
          @error="onAvatarError"
          alt="Avatar"
          class="w-full h-full object-cover"
        />
      </button>
    </div>

    <!-- Navigation Items -->
    <nav class="flex-1 px-2 py-4 space-y-3">
      <button
        v-for="item in navigationItems"
        :key="item.id"
        @click="handleNavigationClick(item)"
        :title="item.label"
        :disabled="!item.enabled"
        :class="[
          'w-full flex items-center justify-center px-3 py-3 rounded-xl transition-all duration-200',
          activeItem === item.id 
            ? 'bg-primary text-white shadow-lg'
            : 'text-text-sidebar-secondary hover:text-text-sidebar hover:bg-sidebar-hover',
          !item.enabled && 'opacity-50 cursor-not-allowed'
        ]"
      >
        <HeroIcon :name="item.icon" className="w-6 h-6" />
      </button>
    </nav>

    <!-- Bottom Menu -->
    <div v-if="settings.showLogoutButton" class="p-2 border-t border-border-sidebar relative" ref="menuRoot">
      <button
        @click.stop="toggleMenu"
        title="Menu"
        class="w-full flex items-center justify-center px-3 py-3 rounded-xl transition-all duration-200 text-text-sidebar-secondary hover:text-text-sidebar hover:bg-sidebar-hover"
      >
        <HeroIcon name="bars-3" className="w-7 h-7" />
      </button>

      <div
        v-if="menuOpen"
        class="absolute bottom-14 left-full -ml-1 w-40 bg-slate-800 border border-slate-700 rounded-xl shadow-xl py-1.5 z-20"
      >
        <button
          @click="handleOpenSettings"
          class="w-full flex items-center justify-center gap-2.5 px-3 py-2.5 text-base text-slate-100 hover:bg-slate-700 transition-colors"
        >
          <HeroIcon name="cog-6-tooth" className="w-5 h-5" />
          <span>设置</span>
        </button>
        <button
          @click="handleLogout"
          class="w-full flex items-center justify-center gap-2.5 px-3 py-2.5 text-base text-red-300 hover:bg-slate-700 transition-colors"
        >
          <HeroIcon name="arrow-left-on-rectangle" className="w-5 h-5" />
          <span>退出</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import router from '../../router/router';
import HeroIcon from '../shared/HeroIcon.vue';
import {
  NavigationItem,
  NAVIGATION_ITEMS,
  NAVIGATION_SETTINGS
} from '../../types/navigation';
import { useUserProfileDisplay } from '../../composables/useUserProfileDisplay';
import defaultAvatarUrl from '../../assets/default-avatar.svg';

const navigationItems = ref<NavigationItem[]>(NAVIGATION_ITEMS.filter((item) => item.id !== 'settings'));
const settings = ref(NAVIGATION_SETTINGS);
const menuOpen = ref(false);
const menuRoot = ref<HTMLElement | null>(null);

// User profile state (auto-updates when profile changes)
const { userProfile } = useUserProfileDisplay();

const avatarUrl = computed(() => {
  const avatar = userProfile.value?.avatar;
  if (!avatar) return defaultAvatarUrl;
  if (avatar.startsWith('http://') || avatar.startsWith('https://')) {
    return avatar;
  }
  return `asset://localhost/${avatar}`;
});

function onAvatarError(event: Event) {
  const img = event.target as HTMLImageElement;
  img.src = defaultAvatarUrl;
}

// Track active item based on current route
const activeItem = computed(() => {
  const currentRoute = router.currentRoute.value;
  return (currentRoute.name as string) || settings.value.defaultActive;
});

const handleNavigationClick = (item: NavigationItem) => {
  if (!item.enabled) return;
  
  router.replace(`/${item.id}`);
};

const handleOpenProfile = () => {
  router.replace('/account');
};

const handleOpenSettings = () => {
  menuOpen.value = false;
  router.replace('/settings');
};

const toggleMenu = () => {
  menuOpen.value = !menuOpen.value;
};

const handleLogout = async () => {
  menuOpen.value = false;
  try {
    await invoke('logout');
    router.push({ name: 'login' });
  } catch (error) {
    console.error('Failed to logout:', error);
    // Still navigate to login even if logout fails
    router.push({ name: 'login' });
  }
};

const onWindowClick = (event: MouseEvent) => {
  if (!menuOpen.value || !menuRoot.value) return;
  const target = event.target as Node | null;
  if (target && !menuRoot.value.contains(target)) {
    menuOpen.value = false;
  }
};

onMounted(() => {
  window.addEventListener('click', onWindowClick);
});

onBeforeUnmount(() => {
  window.removeEventListener('click', onWindowClick);
});
</script>
