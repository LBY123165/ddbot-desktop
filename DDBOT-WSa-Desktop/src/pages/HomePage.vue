<template>
  <div class="home">
    <div class="home__content">
      <!-- 状态概览卡片 -->
      <div class="card card--wide">
        <div class="card__header">
          <div>
            <div class="card__title">运行状态</div>
            <div class="card__subtitle">DDBOT-WSa 本体与 OneBot 连接概览</div>
          </div>
          <div class="actions">
            <Button :icon="RefreshCw" :loading="appStore.loading" @click="appStore.loadStatus">刷新</Button>
            <Button variant="primary" :icon="Download" :loading="appStore.loading" @click="handleInstall">安装/更新</Button>
          </div>
        </div>

        <div class="kpis">
          <div class="kpi">
            <div class="kpi__icon">
              <Activity :size="20" />
            </div>
            <div>
              <div class="kpi__label">本体进程</div>
              <div class="kpi__value" :class="{ 'kpi__value--active': appStore.isRunning }">
                {{ appStore.isRunning ? '运行中' : '已停止' }}
              </div>
              <div v-if="appStore.isRunning" class="kpi__detail">{{ appStore.uptime }} · PID: {{ appStore.status.pid }}</div>
            </div>
          </div>

          <div class="kpi">
            <div class="kpi__icon">
              <Link :size="20" />
            </div>
            <div>
              <div class="kpi__label">OneBot 连接</div>
              <div class="kpi__value" :class="{ 'kpi__value--active': appStore.onebotStatus.connected }">
                {{ appStore.onebotStatus.connected ? '已连接' : '未连接' }}
              </div>
              <div v-if="appStore.onebotStatus.selfId" class="kpi__detail">{{ appStore.onebotStatus.selfId }}</div>
            </div>
          </div>

          <div class="kpi">
            <div class="kpi__icon">
              <Database :size="20" />
            </div>
            <div>
              <div class="kpi__label">订阅统计</div>
              <div class="kpi__value">{{ appStore.subsSummary.total }}</div>
              <div class="kpi__detail">活跃: {{ appStore.subsSummary.active }} · 暂停: {{ appStore.subsSummary.paused }}</div>
            </div>
          </div>

          <div class="kpi">
            <div class="kpi__icon">
              <Tag :size="20" />
            </div>
            <div>
              <div class="kpi__label">当前版本</div>
              <div class="kpi__value">{{ appStore.version }}</div>
              <div class="kpi__detail mono">{{ appStore.platform }}</div>
            </div>
          </div>
        </div>
      </div>

      <!-- 快速操作卡片 -->
      <div class="card">
        <div class="card__header">
          <div>
            <div class="card__title">快速操作</div>
            <div class="card__subtitle">控制 DDBOT-WSa 进程</div>
          </div>
        </div>
        <div class="stack">
          <Button
            v-if="!appStore.isRunning"
            variant="success"
            :icon="Play"
            :loading="appStore.loading"
            size="lg"
            @click="appStore.start"
          >
            启动服务
          </Button>
          <Button
            v-else
            variant="danger"
            :icon="Square"
            :loading="appStore.loading"
            size="lg"
            @click="appStore.stop"
          >
            停止服务
          </Button>
          <Button variant="secondary" :icon="RefreshCw" :loading="appStore.loading" @click="appStore.restart">
            重启服务
          </Button>
        </div>
      </div>

      <!-- 信息卡片 -->
      <div class="card">
        <div class="card__header">
          <div>
            <div class="card__title">安装信息</div>
            <div class="card__subtitle">GitHub Releases 自动更新</div>
          </div>
        </div>
        <div class="stack">
          <div class="row">
            <div class="label">当前平台</div>
            <div class="value mono">{{ appStore.platform }}</div>
          </div>
          <div class="row">
            <div class="label">下载源</div>
            <div class="value mono">cnxysoft/DDBOT-WSa</div>
          </div>
          <div v-if="!appStore.isUserApproved" class="warning">
            <AlertTriangle :size="16" />
            <span>尚未授权 Admin API，部分功能不可用</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="appStore.error" class="home__error">
      <AlertCircle :size="16" />
      <span>{{ appStore.error }}</span>
      <Button size="sm" variant="danger" @click="appStore.clearError">关闭</Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { Activity, AlertCircle, AlertTriangle, Database, Download, Link, Play, RefreshCw, Square, Tag } from 'lucide-vue-next'
import Button from '../components/Button.vue'
import { useAppStore } from '../stores/app'

const appStore = useAppStore()

let statusInterval: number | null = null

onMounted(async () => {
  await appStore.init()

  statusInterval = window.setInterval(() => {
    appStore.loadStatus()
  }, 5000) as unknown as number
})

onUnmounted(() => {
  if (statusInterval) {
    clearInterval(statusInterval)
  }
})

async function handleInstall() {
  try {
    await appStore.install()
  } catch (e) {
    console.error('Install failed:', e)
  }
}
</script>

<style scoped>
.home {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.home__content {
  display: grid;
  gap: 16px;
  grid-template-columns: 2fr 1fr;
}

@media (max-width: 980px) {
  .home__content {
    grid-template-columns: 1fr;
  }
}

.card {
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: rgba(255, 255, 255, 0.04);
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.35);
  overflow: hidden;
}

.card--wide {
  grid-column: 1 / -1;
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

.kpis {
  display: grid;
  gap: 12px;
  padding: 16px;
  grid-template-columns: repeat(4, 1fr);
}

@media (max-width: 980px) {
  .kpis {
    grid-template-columns: repeat(2, 1fr);
  }
}

.kpi {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  border-radius: 14px;
  padding: 14px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(0, 0, 0, 0.18);
}

.kpi__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: rgba(124, 92, 255, 0.15);
  color: #7c5cff;
  flex-shrink: 0;
}

.kpi__label {
  font-size: 12px;
  opacity: 0.7;
}

.kpi__value {
  margin-top: 4px;
  font-size: 16px;
  font-weight: 700;
}

.kpi__value--active {
  color: #10b981;
}

.kpi__detail {
  margin-top: 2px;
  font-size: 11px;
  opacity: 0.6;
}

.actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.stack {
  padding: 16px;
  display: grid;
  gap: 8px;
}

.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 8px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.row:last-child {
  border-bottom: none;
}

.label {
  opacity: 0.7;
  font-size: 12px;
}

.value {
  font-weight: 600;
  font-size: 13px;
}

.mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
  font-size: 11px;
  opacity: 0.9;
}

.warning {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-radius: 10px;
  background: rgba(245, 158, 11, 0.15);
  border: 1px solid rgba(245, 158, 11, 0.3);
  color: #fbbf24;
  font-size: 12px;
}

.home__error {
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
