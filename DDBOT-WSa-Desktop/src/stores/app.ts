import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { TauriAPI } from '../api/tauri'

const API_BASE = 'http://localhost:3000/api'

// HTTP API 调用函数
async function callApi<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
  const response = await fetch(`${API_BASE}${endpoint}`, {
    headers: {
      'Content-Type': 'application/json',
    },
    ...options,
  })

  if (!response.ok) {
    const errorText = await response.text().catch(() => response.statusText)
    throw new Error(errorText || `API 调用失败: ${response.status}`)
  }

  try {
    return await response.json()
  } catch (e) {
    const errorText = await response.text().catch(() => '无效的响应格式')
    throw new Error(`API 响应解析失败: ${errorText}`)
  }
}

export interface DDBOTStatus {
  running: boolean
  pid?: string
  startTime?: number
}

export interface OneBotStatus {
  connected: boolean
  protocol?: string
  selfId?: string
  online: boolean
  good: boolean
}

export interface SubsSummary {
  total: number
  active: number
  paused: number
  bySite: Record<string, number>
}

export const useAppStore = defineStore('app', () => {
  const status = ref<DDBOTStatus>({ running: false })
  const onebotStatus = ref<OneBotStatus>({ connected: false, online: false, good: false })
  const subsSummary = ref<SubsSummary>({ total: 0, active: 0, paused: 0, bySite: {} })
  const version = ref<string>('-')
  const platform = ref<string>('WebUI')
  const loading = ref(false)
  const error = ref<string | undefined>()
  const isUserApproved = ref(true) // WebUI 架构下默认授权
  const initialized = ref(false)
  const firewallStatus = ref(false)
  const updateAvailable = ref(false)
  const latestVersion = ref<string>('-')
  const lastUpdateCheck = ref<string | undefined>()
  const dataDir = ref<string | undefined>()

  const isRunning = computed(() => status.value.running)
  const uptime = computed(() => {
    if (!status.value.startTime) return '-'
    const seconds = Math.floor((Date.now() - status.value.startTime) / 1000)
    const hours = Math.floor(seconds / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    if (hours > 0) return `${hours}h ${minutes}m`
    return `${minutes}m`
  })

  async function init() {
    if (initialized.value) return
    try {
      await loadStatus()
      await loadFirewallStatus()
      await loadDataDir()
      initialized.value = true
    } catch (e) {
      console.error('Failed to init app store:', e)
    }
  }

  async function loadStatus() {
    try {
      loading.value = true
      
      const [processRes, onebot, subs, health] = await Promise.all([
        callApi<any>('/process/status'),
        callApi<any>('/onebot/status').catch(() => null),
        callApi<any>('/subs/summary').catch(() => null),
        callApi<any>('/health').catch(() => null)
      ])

      status.value = {
        running: processRes.running,
        pid: processRes.pid?.toString(),
        startTime: processRes.running ? Date.now() : undefined // 简化的启动时间
      }
      
      if (health) {
        version.value = health.version
      }

      if (onebot) {
        onebotStatus.value = {
          connected: onebot.connected,
          online: onebot.online,
          good: onebot.good,
          protocol: onebot.protocol || 'OneBot v11',
          selfId: onebot.self_id
        }
      }

      if (subs) {
        subsSummary.value = {
          total: subs.total || 0,
          active: subs.active || 0,
          paused: subs.paused || 0,
          bySite: subs.bySite || {}
        }
      }
    } catch (e) {
      console.error('Failed to load status:', e)
    } finally {
      loading.value = false
    }
  }

  async function loadFirewallStatus() {
    try {
      firewallStatus.value = await TauriAPI.firewall.check()
    } catch (e) {
      console.error('Failed to load firewall status:', e)
      firewallStatus.value = false
    }
  }

  async function loadDataDir() {
    try {
      dataDir.value = await TauriAPI.util.defaultDataDdbotDir()
    } catch (e) {
      console.error('Failed to load data dir:', e)
    }
  }

  async function openDataDir() {
    try {
      await TauriAPI.files.openDDBotDataDir()
    } catch (e) {
      console.error('Failed to open data dir:', e)
    }
  }

  async function approve() {
    // WebUI 架构下不需要前端授权逻辑，由后端处理
    isUserApproved.value = true
  }

  async function importDeployment(srcDir: string) {
    try {
      loading.value = true
      error.value = undefined
      // 如果有对应的 API，则调用
      // await callApi('/deployment/import', { method: 'POST', body: JSON.stringify({ srcDir }) })
      await loadStatus()
    } catch (e) {
      error.value = `导入失败: ${e}`
      throw e
    } finally {
      loading.value = false
    }
  }

  async function install() {
    try {
      loading.value = true
      error.value = undefined
      await callApi('/install', { method: 'POST' })
      await loadStatus()
    } catch (e) {
      error.value = `安装失败: ${e}`
      throw e
    } finally {
      loading.value = false
    }
  }

  async function start() {
    try {
      loading.value = true
      error.value = undefined
      await callApi('/process/control', { 
        method: 'POST', 
        body: JSON.stringify({ action: 'start' }) 
      })
      await loadStatus()
    } catch (e) {
      error.value = `启动失败: ${e}`
      throw e
    } finally {
      loading.value = false
    }
  }

  async function stop() {
    try {
      loading.value = true
      error.value = undefined
      await callApi('/process/control', { 
        method: 'POST', 
        body: JSON.stringify({ action: 'stop' }) 
      })
      status.value = { running: false }
      onebotStatus.value = { connected: false, online: false, good: false }
    } catch (e) {
      error.value = `停止失败: ${e}`
      throw e
    } finally {
      loading.value = false
    }
  }

  async function restart() {
    try {
      loading.value = true
      await callApi('/process/control', { 
        method: 'POST', 
        body: JSON.stringify({ action: 'restart' }) 
      })
      await loadStatus()
    } catch (e) {
      error.value = `重启失败: ${e}`
      throw e
    } finally {
      loading.value = false
    }
  }

  function clearError() {
    error.value = undefined
  }

  return {
    status,
    onebotStatus,
    subsSummary,
    version,
    platform,
    loading,
    error,
    isUserApproved,
    initialized,
    firewallStatus,
    updateAvailable,
    latestVersion,
    lastUpdateCheck,
    dataDir,
    isRunning,
    uptime,
    init,
    loadStatus,
    loadFirewallStatus,
    loadDataDir,
    openDataDir,
    approve,
    importDeployment,
    install,
    start,
    stop,
    restart,
    clearError,
  }
})
