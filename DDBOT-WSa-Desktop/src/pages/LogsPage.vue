<template>
  <div class="logs-page">
    <div class="card">
      <div class="card__header">
        <div>
          <div class="card__title">日志查看器</div>
          <div class="card__subtitle">查看 DDBOT-WSa 运行日志</div>
        </div>
        <div class="actions">
          <div class="toolbar-group">
            <label class="toolbar-label">日志来源:</label>
            <Select v-model="selectedSource" @change="loadLogs">
              <Option value="ddbot">DDBOT</Option>
              <Option value="app">应用</Option>
              <Option value="all">全部</Option>
            </Select>
          </div>
          <div class="toolbar-group">
            <label class="toolbar-label">显示行数:</label>
            <InputNumber v-model="logLines" :min="50" :max="1000" :step="50" @change="loadLogs" />
          </div>
          <Button :icon="RefreshCw" :loading="refreshing" @click="loadLogs">刷新</Button>
          <Button :icon="Trash2" variant="danger" @click="clearLogs">清除日志</Button>
          <Button :icon="Eye" :loading="following" @click="toggleFollow">
            {{ following ? '停止跟随' : '跟随日志' }}
          </Button>
        </div>
      </div>

      <div class="logs-container">
        <div v-if="loading" class="loading-container">
          <div class="loading-spinner"></div>
          <div>加载日志中...</div>
        </div>
        <div v-else-if="error" class="error-container">
          <AlertCircle :size="24" />
          <div>{{ error }}</div>
          <Button variant="secondary" @click="loadLogs">重试</Button>
        </div>
        <div v-else class="logs-content">
          <div class="logs-header">
            <div class="logs-info">
              <span>当前来源: {{ selectedSource === 'all' ? '全部' : selectedSource === 'ddbot' ? 'DDBOT' : '应用' }}</span>
              <span>显示行数: {{ logLines }}</span>
              <span v-if="lastUpdated">最后更新: {{ lastUpdated }}</span>
            </div>
          </div>
          <div class="logs-body" ref="logsContainer">
            <div v-if="logs.length === 0" class="logs-empty">暂无日志</div>
            <pre class="logs-text" v-else>{{ logs.join('\n') }}</pre>
          </div>
        </div>
      </div>

      <div class="card__footer">
        <div class="footer-info">
          <span v-if="following" class="follow-indicator">📝 正在跟随日志</span>
          <span class="log-stats">{{ logs.length }} 行日志</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { RefreshCw, Trash2, AlertCircle, Eye } from 'lucide-vue-next'
import Button from '../components/Button.vue'
import Select from '../components/Select.vue'
import Option from '../components/Option.vue'
import InputNumber from '../components/InputNumber.vue'
import { TauriAPI } from '../api/tauri'

// 状态管理
const logs = ref<string[]>([])
const loading = ref(false)
const refreshing = ref(false)
const error = ref<string | undefined>()
const selectedSource = ref<'ddbot' | 'app' | 'all'>('ddbot')
const logLines = ref(200)
const following = ref(true)
const lastUpdated = ref<string | undefined>()
const logsContainer = ref<HTMLElement | null>(null)

// 自动刷新定时器
let autoRefreshTimer: number | undefined

// 加载日志
async function loadLogs() {
  try {
    if (refreshing.value) return
    
    refreshing.value = true
    error.value = undefined
    
    // 使用 readLogsTail 获取指定行数的日志
    const logContent = await TauriAPI.ddbot.readLogsTail(logLines.value)
    logs.value = logContent
    
    lastUpdated.value = new Date().toLocaleTimeString()
    
    // 如果启用了跟随，滚动到底部
    if (following.value) {
      scrollToBottom()
    }
  } catch (e) {
    error.value = `加载日志失败: ${e}`
    console.error('Failed to load logs:', e)
  } finally {
    loading.value = false
    refreshing.value = false
  }
}

// 清除日志
async function clearLogs() {
  if (!confirm('确定要清除当前来源的日志吗？此操作不可恢复。')) {
    return
  }
  
  try {
    loading.value = true
    error.value = undefined
    
    await TauriAPI.logs.clear(selectedSource.value)
    logs.value = []
    lastUpdated.value = new Date().toLocaleTimeString()
  } catch (e) {
    error.value = `清除日志失败: ${e}`
    console.error('Failed to clear logs:', e)
  } finally {
    loading.value = false
  }
}

// 切换跟随日志
function toggleFollow() {
  following.value = !following.value
  if (following.value) {
    scrollToBottom()
  }
}

// 滚动到底部
function scrollToBottom() {
  setTimeout(() => {
    if (logsContainer.value) {
      logsContainer.value.scrollTop = logsContainer.value.scrollHeight
    }
  }, 100)
}

// 自动刷新日志
function startAutoRefresh() {
  if (autoRefreshTimer) {
    clearInterval(autoRefreshTimer)
  }
  
  autoRefreshTimer = window.setInterval(() => {
    if (following.value) {
      loadLogs()
    }
  }, 3000) // 每3秒刷新一次
}

// 停止自动刷新
function stopAutoRefresh() {
  if (autoRefreshTimer) {
    clearInterval(autoRefreshTimer)
    autoRefreshTimer = undefined
  }
}

// 监听跟随状态变化
watch(following, (newValue) => {
  if (newValue) {
    startAutoRefresh()
  } else {
    stopAutoRefresh()
  }
})

// 组件挂载
onMounted(() => {
  loadLogs()
  if (following.value) {
    startAutoRefresh()
  }
})

// 组件卸载
onUnmounted(() => {
  stopAutoRefresh()
})
</script>

<style scoped>
.logs-page {
  display: flex;
  flex-direction: column;
  gap: 16px;
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
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.07);
  flex-wrap: wrap;
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

.card__footer {
  padding: 12px 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
  background: rgba(0, 0, 0, 0.15);
}

.actions {
  display: flex;
  gap: 12px;
  align-items: center;
  flex-wrap: wrap;
}

.toolbar-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.toolbar-label {
  font-size: 13px;
  opacity: 0.8;
  white-space: nowrap;
}

.logs-container {
  height: calc(100vh - 240px);
  min-height: 400px;
  display: flex;
  flex-direction: column;
}

.loading-container,
.error-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 40px;
  text-align: center;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid rgba(255, 255, 255, 0.1);
  border-top-color: #7c5cff;
  border-radius: 50%;
  animation: spin 1s infinite linear;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.logs-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.logs-header {
  padding: 12px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  background: rgba(0, 0, 0, 0.1);
  font-size: 12px;
}

.logs-info {
  display: flex;
  gap: 20px;
  flex-wrap: wrap;
}

.logs-body {
  flex: 1;
  overflow: auto;
  padding: 16px;
  background: rgba(0, 0, 0, 0.2);
}

.logs-empty {
  text-align: center;
  padding: 40px;
  opacity: 0.5;
  font-size: 14px;
}

.logs-text {
  margin: 0;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.4;
  color: rgba(232, 234, 240, 0.9);
  white-space: pre-wrap;
  word-wrap: break-word;
}

.footer-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
}

.follow-indicator {
  color: #10b981;
  font-weight: 500;
}

.log-stats {
  opacity: 0.7;
}

/* 滚动条样式 */
.logs-body::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.logs-body::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 4px;
}

.logs-body::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 4px;
}

.logs-body::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.3);
}

/* 响应式调整 */
@media (max-width: 768px) {
  .card__header {
    flex-direction: column;
    align-items: stretch;
  }
  
  .actions {
    flex-direction: column;
    align-items: stretch;
  }
  
  .toolbar-group {
    justify-content: space-between;
  }
  
  .logs-container {
    height: 500px;
  }
}
</style>