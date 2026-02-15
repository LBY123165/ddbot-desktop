<template>
  <div class="logs">
    <div class="card">
      <div class="card__header">
        <div>
          <div class="card__title">日志查看器</div>
          <div class="card__subtitle">DDBOT-WSa 运行日志</div>
        </div>
        <div class="actions">
          <Button :icon="RefreshCw" :loading="loading" @click="loadLogs">刷新</Button>
          <Button variant="secondary" :icon="Trash" @click="clearLogs">清空</Button>
        </div>
      </div>

      <div class="card__body">
        <div v-if="logs.length === 0 && !loading" class="empty">
          <div class="empty__icon">📝</div>
          <div class="empty__text">暂无日志</div>
        </div>

        <div v-else class="logs__container">
          <div
            v-for="(log, index) in logs"
            :key="index"
            class="log-entry"
            :class="`log-entry--${log.level}`"
          >
            <div class="log-entry__time">{{ formatTime(log.time) }}</div>
            <div class="log-entry__level">{{ log.level }}</div>
            <div class="log-entry__content">{{ log.message }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { RefreshCw, Trash } from 'lucide-vue-next'
import Button from '../components/Button.vue'
import { useAppStore } from '../stores/app'

const appStore = useAppStore()
const loading = ref(false)

interface LogEntry {
  time: Date
  level: 'info' | 'warn' | 'error'
  message: string
}

const logs = ref<LogEntry[]>([])

onMounted(() => {
  loadLogs()
})

async function loadLogs() {
  loading.value = true
  try {
    // TODO: Implement actual log reading from DDBOT-WSa log files
    // For now, show placeholder logs
    logs.value = [
      {
        time: new Date(),
        level: 'info',
        message: 'DDBOT-WSa 桌面管理器已启动'
      },
      {
        time: new Date(Date.now() - 60000),
        level: 'info',
        message: '正在检查运行状态...'
      },
    ]
  } catch (e) {
    console.error('Failed to load logs:', e)
  } finally {
    loading.value = false
  }
}

function clearLogs() {
  logs.value = []
}

function formatTime(date: Date): string {
  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}
</script>

<style scoped>
.logs {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 22px;
  max-width: 1400px;
  margin: 0 auto;
  width: 100%;
}

.card {
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: rgba(255, 255, 255, 0.04);
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.35);
  overflow: hidden;
}

.card__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.07);
}

.card__title {
  font-weight: 700;
  font-size: 15px;
}

.card__subtitle {
  margin-top: 4px;
  font-size: 12px;
  opacity: 0.7;
}

.actions {
  display: flex;
  gap: 8px;
}

.card__body {
  padding: 0;
  max-height: 600px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px;
  color: rgba(232, 234, 240, 0.6);
}

.empty__icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.empty__text {
  font-size: 14px;
}

.logs__container {
  overflow-y: auto;
  padding: 16px;
  flex: 1;
  max-height: 600px;
}

.logs__container::-webkit-scrollbar {
  width: 8px;
}

.logs__container::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.2);
}

.logs__container::-webkit-scrollbar-thumb {
  background: rgba(124, 92, 255, 0.4);
  border-radius: 4px;
}

.log-entry {
  display: grid;
  grid-template-columns: 80px 70px 1fr;
  gap: 12px;
  padding: 8px 0;
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
  font-size: 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.log-entry__time {
  color: rgba(232, 234, 240, 0.5);
}

.log-entry__level {
  text-transform: uppercase;
  font-size: 11px;
  font-weight: 600;
}

.log-entry--info .log-entry__level {
  color: rgba(0, 214, 160, 0.9);
}

.log-entry--warn .log-entry__level {
  color: rgba(251, 191, 36, 0.9);
}

.log-entry--error .log-entry__level {
  color: rgba(239, 68, 68, 0.9);
}

.log-entry__content {
  color: rgba(232, 234, 240, 0.9);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
