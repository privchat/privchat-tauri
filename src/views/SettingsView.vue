<template>
  <div class="h-full bg-background overflow-y-auto">
    <div class="max-w-4xl mx-auto p-8">
      <div class="mb-8">
        <h1 class="text-2xl font-semibold text-text mb-2">Settings</h1>
        <hr class="mt-4 border-border">
      </div>

      <div class="bg-surface rounded-xl border border-border overflow-hidden">
        <div class="p-6">
          <div class="divide-y divide-border">
            <div
              v-for="item in settingItems"
              :key="item.key"
              class="py-4 flex items-center justify-between"
            >
              <div>
                <div class="text-sm font-medium text-text">{{ item.label }}</div>
                <div class="text-xs text-muted-foreground mt-1">{{ item.description }}</div>
              </div>
              <div class="text-xs px-2 py-1 rounded bg-muted/40 text-muted-foreground">Coming Soon</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({
  name: 'SettingsView'
});

const settingItems = [
  { key: 'language', label: 'Language', description: 'Select app display language' },
  { key: 'theme', label: 'Theme', description: 'Light / Dark / System (placeholder)' },
  { key: 'notifications', label: 'Notifications', description: 'Message sound and desktop notification settings' },
  { key: 'privacy', label: 'Privacy', description: 'Blocked list and privacy options' },
  { key: 'storage', label: 'Storage', description: 'Cache and local data options (placeholder)' },
  { key: 'about', label: 'About', description: 'Version and device information (placeholder)' }
];
</script>
