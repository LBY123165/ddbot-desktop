import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const isTauriApp = () => window.__TAURI__ !== undefined

const invokeTauri = async (command: string, args?: any) => {
  if (!isTauriApp()) {
    console.warn(`[Mock] invoke: ${command}`, args)
    throw new Error('Not running in Tauri app')
  }
  return invoke(command, args)
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
}

export interface SubsSummary {
  total: number
  active: number
  paused: number
}

export const useAppStore = defineStore('app', () => {
  const status = ref<DDBOTStatus>({ running: false })
  const onebotStatus = ref<OneBotStatus>({ connected: false })
  const subsSummary = ref<SubsSummary>({ total: 0, active: 0, paused: 0 })
  const version = ref<string>('-')
  const platform = ref<string>('loading...')
  const loading = ref(false)
  const error = ref<string | undefined>()
  const isUserApproved = ref(false)
  const initialized = ref(false)

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
    if (!isTauriApp()) {
      platform.value = 'Browser (Dev Mode)'
      initialized.value = true
      return
    }
    await Promise.all([
      loadStatus(),
      checkApproval(),
      loadPlatform(),
    ])
    initialized.value = true
  }

  async function loadStatus() {
    if (!isTauriApp()) return
    try {
      loading.value = true
      const processStatus = await invokeTauri<string>('process_status_text')
      status.value = parseProcessStatus(processStatus)

      const onebotText = await invokeTauri<string>('onebot_status_text')
      onebotStatus.value = parseOneBotStatus(onebotText)

      const subsText = await invokeTauri<string>('subs_summary_text')
      subsSummary.value = parseSubsSummary(subsText)

      version.value = await invokeTauri<string>('installed_version_text')
    } catch (e) {
      console.error('Failed to load status:', e)
    } finally {
      loading.value = false
    }
  }

  async function loadPlatform() {
    if (!isTauriApp()) return
    try {
      platform.value = await invokeTauri<string>('get_platform_triple')
    } catch (e) {
      platform.value = 'unknown'
    }
  }

  async function checkApproval() {
    if (!isTauriApp()) {
      isUserApproved.value = false
      return
    }
    try {
      isUserApproved.value = await invokeTauri<boolean>('is_user_approved')
    } catch (e) {
      console.error('Failed to check approval:', e)
    }
  }

  async function approve() {
    if (!isTauriApp()) {
      error.value = '此功能仅在 Tauri 应用中可用'
      throw error.value
    }
    try {
      await invokeTauri('set_user_approved', { approved: true })
      isUserApproved.value = true
    } catch (e) {
      error.value = `授权失败: ${e}`
      throw e
    }
  }

  async function importDeployment(srcDir: string) {
    if (!isTauriApp()) {
      error.value = '此功能仅在 Tauri 应用中可用'
      throw error.value
    }
    try {
      loading.value = true
      error.value = undefined
      await invokeTauri('import_existing_deployment', { srcDir })
      await loadStatus()
    } catch (e) {
      error.value = `导入失败: ${e}`
      throw e
    } finally {
      loading.value = false
    }
  }

  async function install() {
    if (!isTauriApp()) {
      error.value = '此功能仅在 Tauri 应用中可用'
      throw error.value
    }
    try {
      loading.value = true
      error.value = undefined
      await invokeTauri('ensure_ddbot_installed')
      version.value = await invokeTauri<string>('installed_version_text')
    } catch (e) {
      error.value = `安装失败: ${e}`
      throw e
    } finally {
      loading.value = false
    }
  }

  async function start() {
    if (!isTauriApp()) {
      error.value = '此功能仅在 Tauri 应用中可用'
      throw error.value
    }
    try {
      loading.value = true
      error.value = undefined
      await invokeTauri('process_start')
      await loadStatus()
    } catch (e) {
      error.value = `启动失败: ${e}`
      throw e
    } finally {
      loading.value = false
    }
  }

  async function stop() {
    if (!isTauriApp()) {
      error.value = '此功能仅在 Tauri 应用中可用'
      throw error.value
    }
    try {
      loading.value = true
      error.value = undefined
      await invokeTauri('process_stop')
      status.value = { running: false }
      onebotStatus.value = { connected: false }
    } catch (e) {
      error.value = `停止失败: ${e}`
      throw e
    } finally {
      loading.value = false
    }
  }

  async function restart() {
    await stop()
    await start()
  }

  function clearError() {
    error.value = undefined
  }

  function parseProcessStatus(text: string): DDBOTStatus {
    if (text.includes('运行中')) {
      const pidMatch = text.match(/pid=(\d+)/)
      return {
        running: true,
        pid: pidMatch?.[1],
        startTime: Date.now(),
      }
    }
    return { running: false }
  }

  function parseOneBotStatus(text: string): OneBotStatus {
    if (text.includes('已连接') || text.includes('在线')) {
      return { connected: true, protocol: 'OneBot v11' }
    }
    return { connected: false }
  }

  function parseSubsSummary(text: string): SubsSummary {
    if (text === '待实现') return { total: 0, active: 0, paused: 0 }
    const match = text.match(/(\d+)\/(\d+)/)
    if (match) {
      return {
        total: parseInt(match[2]),
        active: parseInt(match[1]),
        paused: 0,
      }
    }
    return { total: 0, active: 0, paused: 0 }
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
    isRunning,
    uptime,
    init,
    loadStatus,
    loadPlatform,
    checkApproval,
    approve,
    importDeployment,
    install,
    start,
    stop,
    restart,
    clearError,
  }
})
