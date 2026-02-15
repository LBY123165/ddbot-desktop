<template>
  <div class="config-editor">
    <div class="card">
      <div class="card__header">
        <div>
          <div class="card__title">配置编辑器</div>
          <div class="card__subtitle">编辑 DDBOT-WSa 配置文件</div>
        </div>
        <div class="actions">
          <Button :icon="History" variant="secondary" @click="showBackups = !showBackups">备份历史</Button>
          <Button :icon="Save" :loading="saving" @click="saveConfig">保存配置</Button>
          <Button :icon="RefreshCw" @click="loadConfig">重新加载</Button>
        </div>
      </div>

      <div class="editor-container">
        <div v-if="showBackups" class="backups-panel">
          <div class="backups-header">
            <span>备份历史 ({{ backups.length }})</span>
            <Button size="sm" variant="secondary" @click="showBackups = false">关闭</Button>
          </div>
          <div class="backups-list">
            <div v-if="loadingBackups" class="backups-empty">加载中...</div>
            <div v-else-if="backups.length === 0" class="backups-empty">暂无备份</div>
            <div v-for="backup in backups" :key="backup" class="backup-item">
              <div class="backup-info">
                <div class="backup-name">{{ parseBackupTime(backup) }}</div>
                <div class="backup-file">{{ backup }}</div>
              </div>
              <Button size="sm" variant="secondary" :icon="RotateCcw" @click="restoreBackup(backup)">恢复</Button>
            </div>
          </div>
        </div>

        <div class="editor-main">
          <div class="editor-toolbar">
          <div class="toolbar-item">
            <label class="toolbar-label">文件:</label>
            <select v-model="selectedFile" class="file-select" @change="loadSelectedFile">
              <option value="application.yaml">application.yaml</option>
              <option value="template.yaml">template.yaml</option>
            </select>
          </div>
          <div class="toolbar-item">
            <span class="file-info">{{ fileInfo }}</span>
          </div>
        </div>

        <div class="editor-wrapper">
          <textarea
            v-model="configContent"
            class="config-editor-textarea"
            :placeholder="placeholderText"
            @input="onContentChange"
          ></textarea>
        </div>

          <div class="editor-status">
            <div class="status-item" :class="{ 'status-warning': hasUnsavedChanges }">
              <span v-if="hasUnsavedChanges">🟡 有未保存的更改</span>
              <span v-else>🟢 配置已同步</span>
            </div>
            <div v-if="lastSaved" class="status-item">
              最后保存: {{ lastSaved }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="error" class="config-error">
      <AlertCircle :size="16" />
      <span>{{ error }}</span>
      <Button size="sm" variant="danger" @click="clearError">关闭</Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { Save, RefreshCw, AlertCircle, History, RotateCcw } from 'lucide-vue-next'
import Button from '../components/Button.vue'

const API_BASE = 'http://localhost:3000/api'

const configContent = ref('')
const selectedFile = ref('application.yaml')
const saving = ref(false)
const loadingBackups = ref(false)
const backups = ref<string[]>([])
const showBackups = ref(false)
const error = ref<string | undefined>()
const hasUnsavedChanges = ref(false)
const lastSaved = ref<string | undefined>()
const originalContent = ref('')

const fileInfo = computed(() => {
  const lines = configContent.value.split('\n').length
  const chars = configContent.value.length
  return `${lines} 行, ${chars} 字符`
})

const placeholderText = computed(() => {
  if (selectedFile.value === 'application.yaml') {
    return '# DDBOT-WSa 配置文件\n# 在此处编辑您的配置...'
  }
  return '# 模板配置文件\n# 在此处编辑模板配置...'
})

async function loadConfig() {
  try {
    error.value = undefined
    const response = await fetch(`${API_BASE}/config?filename=${selectedFile.value}`)
    if (!response.ok) throw new Error('无法加载配置')
    const data = await response.json()
    configContent.value = data.content
    originalContent.value = data.content
    hasUnsavedChanges.value = false
    await loadBackups()
  } catch (e) {
    error.value = `加载配置失败: ${e}`
    configContent.value = ''
  }
}

async function loadBackups() {
  try {
    loadingBackups.value = true
    const response = await fetch(`${API_BASE}/config/backups?filename=${selectedFile.value}`)
    if (!response.ok) throw new Error('无法加载备份')
    const data = await response.json()
    backups.value = data.backups
  } catch (e) {
    console.error('Failed to load backups:', e)
  } finally {
    loadingBackups.value = false
  }
}

async function saveConfig() {
  try {
    saving.value = true
    error.value = undefined
    const response = await fetch(`${API_BASE}/config`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        filename: selectedFile.value,
        content: configContent.value
      })
    })
    if (!response.ok) throw new Error('保存失败')
    originalContent.value = configContent.value
    hasUnsavedChanges.value = false
    lastSaved.value = new Date().toLocaleTimeString()
    await loadBackups()
  } catch (e) {
    error.value = `保存配置失败: ${e}`
  } finally {
    saving.value = false
  }
}

async function restoreBackup(backupName: string) {
  if (!confirm(`确定要恢复备份 ${backupName} 吗？当前未保存的更改将丢失。`)) {
    return
  }
  try {
    const response = await fetch(`${API_BASE}/config/restore`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ backup_name: backupName })
    })
    if (!response.ok) throw new Error('恢复失败')
    await loadConfig()
    showBackups.value = false
  } catch (e) {
    error.value = `恢复备份失败: ${e}`
  }
}

function parseBackupTime(name: string) {
  // filename.YYYYMMDD_HHMMSS.bak
  const match = name.match(/\.(\d{8}_\d{6})\.bak$/)
  if (!match) return name
  const t = match[1]
  return `${t.slice(0,4)}-${t.slice(4,6)}-${t.slice(6,8)} ${t.slice(9,11)}:${t.slice(11,13)}:${t.slice(13,15)}`
}

function onContentChange() {
  hasUnsavedChanges.value = configContent.value !== originalContent.value
}

function clearError() {
  error.value = undefined
}

async function loadSelectedFile() {
  await loadConfig()
}

onMounted(() => {
  loadConfig()
})
</script>

<style scoped>
.config-editor {
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
  flex-wrap: wrap;
}

.editor-container {
  display: flex;
  flex-direction: row;
  height: calc(100vh - 200px);
  position: relative;
}

.backups-panel {
  width: 300px;
  background: rgba(0, 0, 0, 0.25);
  border-right: 1px solid rgba(255, 255, 255, 0.07);
  display: flex;
  flex-direction: column;
  animation: slide-in 0.3s ease;
}

@keyframes slide-in {
  from { transform: translateX(-100%); }
  to { transform: translateX(0); }
}

.backups-header {
  padding: 12px 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  font-size: 13px;
  font-weight: 600;
}

.backups-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.backups-empty {
  padding: 20px;
  text-align: center;
  font-size: 12px;
  opacity: 0.5;
}

.backup-item {
  padding: 10px 12px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.03);
  margin-bottom: 8px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
}

.backup-item:hover {
  background: rgba(255, 255, 255, 0.06);
}

.backup-info {
  flex: 1;
  min-width: 0;
}

.backup-name {
  font-size: 12px;
  font-weight: 600;
  margin-bottom: 2px;
}

.backup-file {
  font-size: 10px;
  opacity: 0.5;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.editor-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.editor-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  background: rgba(0, 0, 0, 0.2);
}

.toolbar-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.toolbar-label {
  font-size: 13px;
  opacity: 0.8;
}

.file-select {
  padding: 6px 10px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(0, 0, 0, 0.3);
  color: rgba(232, 234, 240, 0.95);
  font-size: 13px;
  outline: none;
}

.file-select:focus {
  border-color: rgba(124, 92, 255, 0.5);
}

.file-info {
  font-size: 12px;
  opacity: 0.7;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

.editor-wrapper {
  flex: 1;
  overflow: hidden;
}

.config-editor-textarea {
  width: 100%;
  height: 100%;
  padding: 16px;
  border: none;
  background: transparent;
  color: rgba(232, 234, 240, 0.95);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.5;
  resize: none;
  outline: none;
}

.config-editor-textarea:focus {
  background: rgba(0, 0, 0, 0.1);
}

.editor-status {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
  background: rgba(0, 0, 0, 0.15);
  font-size: 12px;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-warning {
  color: #f59e0b;
}

.config-error {
  position: fixed;
  bottom: 24px;
  right: 24px;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 10px;
  background: rgba(239, 68, 68, 0.9);
  color: white;
  font-size: 13px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(10px);
  max-width: 400px;
  z-index: 1000;
}
</style>