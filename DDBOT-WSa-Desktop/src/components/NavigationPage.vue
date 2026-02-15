<template>
  <nav class="nav">
    <div class="nav__header">
      <div class="nav__title">DDBOT-WSa</div>
      <div class="nav__subtitle">Desktop</div>
    </div>

    <div class="nav__links">
      <RouterLink
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        class="nav__link"
        :class="{ 'nav__link--active': $route.path === item.path }"
      >
        <component :is="item.icon" :size="18" />
        <span>{{ item.name }}</span>
        <div v-if="item.badge" class="nav__badge">{{ item.badge }}</div>
      </RouterLink>
    </div>

    <div class="nav__footer">
      <div class="nav__version">v{{ appStore.version }}</div>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import { Activity, Home, Settings, Zap, FileText, ScrollText, LayoutTemplate } from 'lucide-vue-next'
import { useAppStore } from '../stores/app'

const appStore = useAppStore()

const navItems = computed(() => [
  {
    name: '概览',
    path: '/',
    icon: Home,
  },
  {
    name: '配置',
    path: '/config',
    icon: FileText,
  },
  {
    name: '日志',
    path: '/logs',
    icon: ScrollText,
  },
  {
    name: '模板',
    path: '/templates',
    icon: LayoutTemplate,
  },
  {
    name: '设置',
    path: '/settings',
    icon: Settings,
  },
])
</script>

<style scoped>
.nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 16px 12px;
  border-right: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(11, 16, 32, 0.5);
  backdrop-filter: blur(10px);
  min-width: 200px;
}

.nav__header {
  padding: 8px 12px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  margin-bottom: 8px;
}

.nav__title {
  font-weight: 700;
  font-size: 16px;
  letter-spacing: 0.2px;
}

.nav__subtitle {
  margin-top: 4px;
  font-size: 12px;
  opacity: 0.6;
}

.nav__links {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.nav__link {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 10px;
  color: rgba(232, 234, 240, 0.8);
  text-decoration: none;
  border: 1px solid transparent;
  transition: all 0.2s ease;
  position: relative;
}

.nav__link:hover {
  background: rgba(255, 255, 255, 0.05);
  color: rgba(232, 234, 240, 1);
}

.nav__link--active {
  border-color: rgba(124, 92, 255, 0.35);
  background: rgba(124, 92, 255, 0.12);
  color: rgba(232, 234, 240, 1);
}

.nav__badge {
  margin-left: auto;
  background: rgba(16, 185, 129, 0.2);
  color: #10b981;
  font-size: 10px;
  padding: 2px 6px;
  border-radius: 10px;
  font-weight: 600;
}

.nav__footer {
  padding-top: 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.nav__version {
  font-size: 11px;
  opacity: 0.5;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}
</style>
