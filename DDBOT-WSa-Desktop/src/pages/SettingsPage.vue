<template>
  <div class="settings">
    <div class="card">
      <div class="card__header">
        <div>
          <div class="card__title">设置</div>
          <div class="card__subtitle">配置 DDBOT-WSa Desktop</div>
        </div>
      </div>

      <div class="card__body">
        <div class="section">
          <div class="section__title">安装选项</div>
          <div class="row">
            <div class="label">安装通道</div>
            <select class="select" v-model="channel" disabled>
              <option value="release">Release（GitHub Releases）</option>
              <option value="action">Action（测试版，后续支持）</option>
            </select>
          </div>
        </div>

        <div class="section">
          <div class="section__title">路径信息</div>
          <div class="row">
            <div class="label">工作目录</div>
            <div class="value mono">{{ workdir }}</div>
          </div>
          <div class="row">
            <div class="label">本体路径</div>
            <div class="value mono">{{ binaryPath }}</div>
          </div>
          <div class="row">
            <div class="label">数据目录</div>
            <div class="value mono">{{ managedDir }}</div>
          </div>
        </div>

        <div class="section">
          <div class="section__title">操作</div>
          <div class="actions">
            <Button :icon="Folder" @click="openWorkDir">打开工作目录</Button>
            <Button :icon="Download" @click="install">安装/更新</Button>
          </div>
        </div>
      </div>
    </div>

    <div v-if="appStore.error" class="settings__error">
      <AlertCircle :size="16" />
      <span>{{ appStore.error }}</span>
      <Button size="sm" variant="danger" @click="appStore.clearError">关闭</Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-shell'
import { AlertCircle, Download, Folder } from 'lucide-vue-next'
import Button from '../components/Button.vue'
import { useAppStore } from '../stores/app'

const appStore = useAppStore()

const channel = ref<'release' | 'action'>('release')
const workdir = ref('loading...')
const binaryPath = ref('loading...')
const managedDir = ref('loading...')

async function loadPaths() {
  try {
    workdir.value = await invoke<string>('get_workdir')
    binaryPath.value = await invoke<string>('get_binary_path')
    managedDir.value = await invoke<string>('get_managed_ddbot_dir')
  } catch (e) {
    appStore.error = String(e)
  }
}

async function install() {
  try {
    appStore.loading = true
    appStore.error = undefined
    await invoke('ensure_ddbot_installed')
    appStore.version = await invoke<string>('installed_version_text')
  } catch (e) {
    appStore.error = `安装失败: ${e}`
  } finally {
    appStore.loading = false
  }
}

async function openWorkDir() {
  try {
    const dir = await invoke<string>('get_workdir')
    await open(dir)
  } catch (e) {
    appStore.error = `打开目录失败: ${e}`
  }
}

onMounted(() => {
  loadPaths()
})
</script>

<style scoped>
.settings {
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

.card__body {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section__title {
  font-size: 14px;
  font-weight: 600;
  color: rgba(232, 234, 240, 0.9);
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 8px 0;
}

.label {
  opacity: 0.7;
  font-size: 13px;
}

.value {
  font-weight: 600;
  font-size: 13px;
}

.mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
  font-size: 12px;
  opacity: 0.85;
  text-align: right;
  max-width: 50%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.select {
  padding: 8px 12px;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(0, 0, 0, 0.25);
  color: rgba(232, 234, 240, 0.95);
  font-size: 13px;
  outline: none;
}

.select:focus {
  border-color: rgba(124, 92, 255, 0.5);
}

.actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.settings__error {
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
