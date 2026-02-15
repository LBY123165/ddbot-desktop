<template>
  <div class="graphical-config-editor">
    <div class="card">
      <div class="card__header">
        <div>
          <div class="card__title">图形化配置编辑器</div>
          <div class="card__subtitle">可视化编辑 DDBOT-WSa 配置文件</div>
        </div>
        <div class="actions">
          <Button :icon="History" variant="secondary" @click="showBackups = !showBackups">备份历史</Button>
          <Button :icon="Save" :loading="saving" @click="saveConfig">保存配置</Button>
          <Button :icon="RefreshCw" @click="loadConfig">重新加载</Button>
          <Button :icon="Code" variant="secondary" @click="switchToTextEditor">文本编辑</Button>
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
              <Button size="sm" variant="secondary" :icon="RotateCcw" @click="restoreBackup(backup)">
                恢复
              </Button>
            </div>
          </div>
        </div>

        <div class="editor-main">
          <div v-if="loading" class="loading-container">
            <div class="loading-spinner"></div>
            <div>加载配置中...</div>
          </div>
          <div v-else-if="error" class="error-container">
            <AlertCircle :size="24" />
            <div>{{ error }}</div>
            <Button variant="secondary" @click="loadConfig">重试</Button>
          </div>
          <div v-else class="config-form">
            <!-- Bot 设置 -->
            <div class="config-section">
              <div class="section-header" @click="toggleSection('bot')">
                <div class="section-title">
                  <Bot :size="18" />
                  <span>Bot 设置</span>
                </div>
                <div class="section-toggle">{{ sections.bot ? '▼' : '▶' }}</div>
              </div>
              <div v-if="sections.bot" class="section-content">
                <div class="config-group">
                  <h4>加入群组设置</h4>
                  <div class="config-item">
                    <label class="config-label">自动改名</label>
                    <div class="config-control">
                      <Input v-model="config.bot.onJoinGroup.rename" placeholder="留空则不自动改名" />
                      <div class="config-hint">BOT进群后自动改名，默认改名为"【bot】"</div>
                    </div>
                  </div>
                </div>

                <div class="config-group">
                  <h4>失败提醒</h4>
                  <div class="config-item">
                    <label class="config-label">启用失败提醒</label>
                    <div class="config-control">
                      <Toggle v-model="config.bot.sendFailureReminder.enable" />
                    </div>
                  </div>
                  <div class="config-item" v-if="config.bot.sendFailureReminder.enable">
                    <label class="config-label">失败次数阈值</label>
                    <div class="config-control">
                      <InputNumber v-model="config.bot.sendFailureReminder.times" :min="1" :max="10" />
                      <div class="config-hint">发送失败达到一定次数后触发提醒</div>
                    </div>
                  </div>
                </div>

                <div class="config-group">
                  <h4>离线缓存</h4>
                  <div class="config-item">
                    <label class="config-label">启用离线缓存</label>
                    <div class="config-control">
                      <Toggle v-model="config.bot.offlineQueue.enable" />
                    </div>
                  </div>
                  <div class="config-item" v-if="config.bot.offlineQueue.enable">
                    <label class="config-label">离线消息有效期</label>
                    <div class="config-control">
                      <Input v-model="config.bot.offlineQueue.expire" placeholder="例如：30m" />
                      <div class="config-hint">离线消息有效期，如 30m 表示30分钟</div>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Bilibili 设置 -->
            <div class="config-section">
              <div class="section-header" @click="toggleSection('bilibili')">
                <div class="section-title">
                  <Video :size="18" />
                  <span>Bilibili 设置</span>
                </div>
                <div class="section-toggle">{{ sections.bilibili ? '▼' : '▶' }}</div>
              </div>
              <div v-if="sections.bilibili" class="section-content">
                <div class="config-group">
                  <div class="config-item">
                    <label class="config-label">SESSDATA</label>
                    <div class="config-control">
                      <Input v-model="config.bilibili.SESSDATA" :type="showBiliCookies ? 'text' : 'password'" />
                      <Button size="sm" variant="secondary" @click="showBiliCookies = !showBiliCookies">
                        {{ showBiliCookies ? '隐藏' : '显示' }}
                      </Button>
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">bili_jct</label>
                    <div class="config-control">
                      <Input v-model="config.bilibili.bili_jct" :type="showBiliCookies ? 'text' : 'password'" />
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">启用二维码登录</label>
                    <div class="config-control">
                      <Toggle v-model="config.bilibili.qrlogin" />
                      <div class="config-hint">Cookies失效时只需要清空SESSDATA和bili_jct重启即可再次登录</div>
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">检测间隔</label>
                    <div class="config-control">
                      <Input v-model="config.bilibili.interval" placeholder="例如：25s" />
                      <div class="config-hint">直播状态和动态检测间隔，过快可能导致ip被暂时封禁</div>
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">图片合并模式</label>
                    <div class="config-control">
                      <Select v-model="config.bilibili.imageMergeMode">
                        <Option value="auto">自动</Option>
                        <Option value="only9">仅9张</Option>
                        <Option value="off">关闭</Option>
                      </Select>
                      <div class="config-hint">auto: 存在刷屏图片时合并；only9: 仅9张时合并；off: 不合并</div>
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">使用悄悄关注</label>
                    <div class="config-control">
                      <Toggle v-model="config.bilibili.hiddenSub" />
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">自动取消关注</label>
                    <div class="config-control">
                      <Toggle v-model="config.bilibili.unsub" />
                      <div class="config-hint">默认不取消，如果您的b站账号有多个bot同时使用，取消可能导致推送丢失</div>
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">最小粉丝数</label>
                    <div class="config-control">
                      <InputNumber v-model="config.bilibili.minFollowerCap" :min="-1" />
                      <div class="config-hint">设置订阅的b站用户需要满足至少有多少个粉丝，设为-1表示无限制</div>
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">禁止自动关注</label>
                    <div class="config-control">
                      <Toggle v-model="config.bilibili.disableSub" />
                      <div class="config-hint">禁止ddbot去b站关注帐号，这意味着只能订阅帐号已关注的用户</div>
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">仅在线通知</label>
                    <div class="config-control">
                      <Toggle v-model="config.bilibili.onlyOnlineNotify" />
                      <div class="config-hint">是否不推送Bot离线期间的动态和直播</div>
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">自动解析专栏</label>
                    <div class="config-control">
                      <Toggle v-model="config.bilibili.autoParsePosts" />
                      <div class="config-hint">将发送专栏动态改为发送专栏内容</div>
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">动态二次解析</label>
                    <div class="config-control">
                      <Toggle v-model="config.bilibili.secAnalysis" />
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- 其他平台设置 -->
            <div class="config-section">
              <div class="section-header" @click="toggleSection('otherPlatforms')">
                <div class="section-title">
                  <Globe :size="18" />
                  <span>其他平台设置</span>
                </div>
                <div class="section-toggle">{{ sections.otherPlatforms ? '▼' : '▶' }}</div>
              </div>
              <div v-if="sections.otherPlatforms" class="section-content">
                <!-- AcFun 设置 -->
                <div class="config-group">
                  <h4>AcFun</h4>
                  <div class="config-item">
                    <label class="config-label">账号</label>
                    <div class="config-control">
                      <Input v-model="config.acfun.account" />
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">密码</label>
                    <div class="config-control">
                      <Input v-model="config.acfun.password" type="password" />
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">自动取消关注</label>
                    <div class="config-control">
                      <Toggle v-model="config.acfun.unsub" />
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">检测间隔</label>
                    <div class="config-control">
                      <Input v-model="config.acfun.interval" placeholder="例如：25s" />
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">仅在线通知</label>
                    <div class="config-control">
                      <Toggle v-model="config.acfun.onlyOnlineNotify" />
                    </div>
                  </div>
                </div>

                <!-- Twitter 设置 -->
                <div class="config-group">
                  <h4>Twitter</h4>
                  <div class="config-item">
                    <label class="config-label">Nitter 镜像</label>
                    <div class="config-control">
                      <div v-for="(url, index) in config.twitter.baseUrl" :key="index" class="array-item">
                        <Input v-model="config.twitter.baseUrl[index]" />
                        <Button size="sm" variant="danger" @click="removeTwitterUrl(index)">删除</Button>
                      </div>
                      <Button variant="secondary" @click="addTwitterUrl">添加镜像</Button>
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">检测间隔</label>
                    <div class="config-control">
                      <Input v-model="config.twitter.interval" placeholder="例如：30s" />
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">User Agent</label>
                    <div class="config-control">
                      <Input v-model="config.twitter.userAgent" />
                    </div>
                  </div>
                </div>

                <!-- 其他平台简要设置 -->
                <div class="config-group">
                  <h4>其他平台</h4>
                  <div class="config-item">
                    <label class="config-label">抖音 - 仅在线通知</label>
                    <div class="config-control">
                      <Toggle v-model="config.douyin.onlyOnlineNotify" />
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">微博 - 仅在线通知</label>
                    <div class="config-control">
                      <Toggle v-model="config.weibo.onlyOnlineNotify" />
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">YouTube - 仅在线通知</label>
                    <div class="config-control">
                      <Toggle v-model="config.youtube.onlyOnlineNotify" />
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- 系统设置 -->
            <div class="config-section">
              <div class="section-header" @click="toggleSection('system')">
                <div class="section-title">
                  <Settings :size="18" />
                  <span>系统设置</span>
                </div>
                <div class="section-toggle">{{ sections.system ? '▼' : '▶' }}</div>
              </div>
              <div v-if="sections.system" class="section-content">
                <div class="config-group">
                  <h4>基本设置</h4>
                  <div class="config-item">
                    <label class="config-label">日志等级</label>
                    <div class="config-control">
                      <Select v-model="config.logLevel">
                        <Option value="trace">Trace</Option>
                        <Option value="debug">Debug</Option>
                        <Option value="info">Info</Option>
                        <Option value="warn">Warn</Option>
                        <Option value="error">Error</Option>
                      </Select>
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">启用模板功能</label>
                    <div class="config-control">
                      <Toggle v-model="config.template.enable" />
                    </div>
                  </div>
                </div>

                <!-- WebSocket 设置 -->
                <div class="config-group">
                  <h4>WebSocket</h4>
                  <div class="config-item">
                    <label class="config-label">模式</label>
                    <div class="config-control">
                      <Select v-model="config.websocket.mode">
                        <Option value="ws-server">正向 (ws-server)</Option>
                        <Option value="ws-reverse">反向 (ws-reverse)</Option>
                      </Select>
                    </div>
                  </div>
                  <div class="config-item">
                    <label class="config-label">Token</label>
                    <div class="config-control">
                      <Input v-model="config.websocket.token" :type="showWsToken ? 'text' : 'password'" />
                      <Button size="sm" variant="secondary" @click="showWsToken = !showWsToken">
                        {{ showWsToken ? '隐藏' : '显示' }}
                      </Button>
                    </div>
                  </div>
                  <div class="config-item" v-if="config.websocket.mode === 'ws-server'">
                    <label class="config-label">监听地址</label>
                    <div class="config-control">
                      <Input v-model="config.websocket['ws-server']" placeholder="例如：0.0.0.0:15630" />
                    </div>
                  </div>
                  <div class="config-item" v-if="config.websocket.mode === 'ws-reverse'">
                    <label class="config-label">反向地址</label>
                    <div class="config-control">
                      <Input v-model="config.websocket['ws-reverse']" placeholder="例如：ws://localhost:3001" />
                    </div>
                  </div>
                </div>

                <!-- 管理员设置 -->
                <div class="config-group">
                  <h4>管理员</h4>
                  <div class="config-item">
                    <label class="config-label">启用管理员</label>
                    <div class="config-control">
                      <Toggle v-model="config.admin.enable" />
                    </div>
                  </div>
                  <div class="config-item" v-if="config.admin.enable">
                    <label class="config-label">地址</label>
                    <div class="config-control">
                      <Input v-model="config.admin.addr" placeholder="例如：127.0.0.1:15631" />
                    </div>
                  </div>
                  <div class="config-item" v-if="config.admin.enable">
                    <label class="config-label">Token</label>
                    <div class="config-control">
                      <Input v-model="config.admin.token" :type="showAdminToken ? 'text' : 'password'" />
                      <Button size="sm" variant="secondary" @click="showAdminToken = !showAdminToken">
                        {{ showAdminToken ? '隐藏' : '显示' }}
                      </Button>
                    </div>
                  </div>
                </div>

                <!-- 延迟加载设置 -->
                <div class="config-group">
                  <h4>延迟加载</h4>
                  <div class="config-item">
                    <label class="config-label">启用数据延迟加载</label>
                    <div class="config-control">
                      <Toggle v-model="config.reloadDelay.enable" />
                    </div>
                  </div>
                  <div class="config-item" v-if="config.reloadDelay.enable">
                    <label class="config-label">延迟时间</label>
                    <div class="config-control">
                      <Input v-model="config.reloadDelay.time" placeholder="例如：3s" />
                    </div>
                  </div>
                </div>

                <!-- 扩展数据库设置 -->
                <div class="config-group">
                  <h4>扩展数据库</h4>
                  <div class="config-item">
                    <label class="config-label">启用扩展数据库</label>
                    <div class="config-control">
                      <Toggle v-model="config.extDb.enable" />
                    </div>
                  </div>
                  <div class="config-item" v-if="config.extDb.enable">
                    <label class="config-label">数据库路径</label>
                    <div class="config-control">
                      <Input v-model="config.extDb.path" placeholder=".ext.db" />
                    </div>
                  </div>
                </div>

                <!-- Telegram 设置 -->
                <div class="config-group">
                  <h4>Telegram</h4>
                  <div class="config-item">
                    <label class="config-label">启用Telegram</label>
                    <div class="config-control">
                      <Toggle v-model="config.telegram.enable" />
                    </div>
                  </div>
                  <div class="config-item" v-if="config.telegram.enable">
                    <label class="config-label">Bot Token</label>
                    <div class="config-control">
                      <Input v-model="config.telegram.token" />
                    </div>
                  </div>
                  <div class="config-item" v-if="config.telegram.enable">
                    <label class="config-label">启用代理</label>
                    <div class="config-control">
                      <Toggle v-model="config.telegram.proxy.enable" />
                    </div>
                  </div>
                  <div class="config-item" v-if="config.telegram.enable && config.telegram.proxy.enable">
                    <label class="config-label">代理地址</label>
                    <div class="config-control">
                      <Input v-model="config.telegram.proxy.url" placeholder="例如：http://127.0.0.1:7890" />
                    </div>
                  </div>
                  <div class="config-item" v-if="config.telegram.enable">
                    <label class="config-label">自定义 API Endpoint</label>
                    <div class="config-control">
                      <Input v-model="config.telegram.endpoint" />
                      <div class="config-hint">留空使用默认</div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="card__footer">
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

    <div v-if="error" class="config-error">
      <AlertCircle :size="16" />
      <span>{{ error }}</span>
      <Button size="sm" variant="danger" @click="clearError">关闭</Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { Save, RefreshCw, AlertCircle, History, RotateCcw, Bot, Video, Globe, Settings, Code } from 'lucide-vue-next'
import Button from '../components/Button.vue'
import Input from '../components/Input.vue'
import Toggle from '../components/Toggle.vue'
import Select from '../components/Select.vue'
import Option from '../components/Option.vue'
import InputNumber from '../components/InputNumber.vue'
import { TauriAPI } from '../api/tauri'
import { useRouter } from 'vue-router'

const router = useRouter()

// 状态管理
const config = ref<any>({
  bot: {
    onJoinGroup: {
      rename: "【bot】"
    },
    sendFailureReminder: {
      enable: false,
      times: 3
    },
    offlineQueue: {
      enable: false,
      expire: "30m"
    }
  },
  bilibili: {
    SESSDATA: "",
    bili_jct: "",
    qrlogin: true,
    interval: "25s",
    imageMergeMode: "auto",
    hiddenSub: false,
    unsub: false,
    minFollowerCap: 0,
    disableSub: false,
    onlyOnlineNotify: false,
    autoParsePosts: false,
    secAnalysis: false
  },
  acfun: {
    account: "",
    password: "",
    unsub: false,
    interval: "25s",
    onlyOnlineNotify: false
  },
  twitter: {
    baseUrl: [
      "https://nitter.net/",
      "https://nitter.privacyredirect.com/",
      "https://nitter.tiekoetter.com/",
      "https://nitter.poast.org/"
    ],
    interval: "30s",
    userAgent: ""
  },
  douyin: {
    acSignature: "",
    acNonce: "",
    sessionId: "",
    userAgent: "",
    interval: "30s",
    onlyOnlineNotify: false
  },
  weibo: {
    onlyOnlineNotify: true,
    sub: ""
  },
  youtube: {
    onlyOnlineNotify: true
  },
  concern: {
    emitInterval: "5s"
  },
  template: {
    enable: true
  },
  autoreply: {
    group: {
      command: ["签到"]
    },
    private: {
      command: []
    }
  },
  customCommandPrefix: {
    "签到": ""
  },
  logLevel: "info",
  websocket: {
    mode: "ws-server",
    token: "",
    "ws-server": "0.0.0.0:15630",
    "ws-reverse": "ws://localhost:3001"
  },
  admin: {
    enable: false,
    addr: "127.0.0.1:15631",
    token: ""
  },
  reloadDelay: {
    enable: true,
    time: "3s"
  },
  extDb: {
    enable: false,
    path: ".ext.db"
  },
  telegram: {
    enable: false,
    token: "",
    proxy: {
      enable: false,
      url: ""
    },
    endpoint: ""
  }
})

const originalConfig = ref<any>({...config.value})
const loading = ref(false)
const saving = ref(false)
const error = ref<string | undefined>()
const hasUnsavedChanges = ref(false)
const lastSaved = ref<string | undefined>()
const showBackups = ref(false)
const loadingBackups = ref(false)
const backups = ref<string[]>([])

// 显示控制
const showBiliCookies = ref(false)
const showWsToken = ref(false)
const showAdminToken = ref(false)

// 展开/折叠状态
const sections = ref({
  bot: true,
  bilibili: true,
  otherPlatforms: false,
  system: false
})

// 切换到文本编辑器
function switchToTextEditor() {
  router.push('/config')
}

// 切换配置节
function toggleSection(section: string) {
  sections.value[section] = !sections.value[section]
}

// 添加Twitter URL
function addTwitterUrl() {
  if (!config.value.twitter.baseUrl) {
    config.value.twitter.baseUrl = []
  }
  config.value.twitter.baseUrl.push('')
}

// 删除Twitter URL
function removeTwitterUrl(index: number) {
  if (config.value.twitter.baseUrl && config.value.twitter.baseUrl.length > 1) {
    config.value.twitter.baseUrl.splice(index, 1)
  }
}

// 加载配置
async function loadConfig() {
  try {
    loading.value = true
    error.value = undefined
    
    const content = await TauriAPI.ddbot.readConfigFile('application.yaml')
    config.value = parseYAML(content)
    originalConfig.value = JSON.parse(JSON.stringify(config.value))
    hasUnsavedChanges.value = false
    
    await loadBackups()
  } catch (e) {
    error.value = `加载配置失败: ${e}`
    console.error('Failed to load config:', e)
  } finally {
    loading.value = false
  }
}

// 保存配置
async function saveConfig() {
  try {
    saving.value = true
    error.value = undefined
    
    const content = generateYAML(config.value)
    await TauriAPI.ddbot.writeConfigFile('application.yaml', content)
    
    originalConfig.value = JSON.parse(JSON.stringify(config.value))
    hasUnsavedChanges.value = false
    lastSaved.value = new Date().toLocaleTimeString()
    
    await loadBackups()
  } catch (e) {
    error.value = `保存配置失败: ${e}`
    console.error('Failed to save config:', e)
  } finally {
    saving.value = false
  }
}

// 加载备份
async function loadBackups() {
  try {
    loadingBackups.value = true
    backups.value = await TauriAPI.ddbot.listConfigBackups('application.yaml')
  } catch (e) {
    console.error('Failed to load backups:', e)
  } finally {
    loadingBackups.value = false
  }
}

// 恢复备份
async function restoreBackup(backupName: string) {
  if (!confirm(`确定要恢复备份 ${backupName} 吗？当前未保存的更改将丢失。`)) {
    return
  }
  try {
    await TauriAPI.ddbot.restoreConfigBackup(backupName)
    await loadConfig()
    showBackups.value = false
  } catch (e) {
    error.value = `恢复备份失败: ${e}`
  }
}

// 解析备份时间
function parseBackupTime(name: string) {
  const match = name.match(/\.(\d{8}_\d{6})\.bak$/)
  if (!match) return name
  const t = match[1]
  return `${t.slice(0,4)}-${t.slice(4,6)}-${t.slice(6,8)} ${t.slice(9,11)}:${t.slice(11,13)}:${t.slice(13,15)}`
}

// 清除错误
function clearError() {
  error.value = undefined
}

// 解析YAML
function parseYAML(content: string): any {
  // 简单的YAML解析，实际应用中可能需要更复杂的解析
  const lines = content.split('\n')
  const result: any = {}
  let currentObj: any = result
  const stack: any[] = []
  
  for (const line of lines) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('#')) continue
    
    const indent = line.length - trimmed.length
    const keyMatch = trimmed.match(/^([^:]+):\s*(.*)$/)
    if (keyMatch) {
      const key = keyMatch[1]
      const value = keyMatch[2]
      
      // 调整对象层级
      while (stack.length > 0 && stack[stack.length - 1].indent >= indent) {
        stack.pop()
        currentObj = stack.length > 0 ? stack[stack.length - 1].obj : result
      }
      
      if (value === '') {
        // 嵌套对象
        const newObj = {}
        currentObj[key] = newObj
        stack.push({ indent, obj: currentObj })
        currentObj = newObj
      } else {
        // 直接值
        currentObj[key] = parseValue(value)
      }
    }
  }
  
  // 确保返回的对象结构完整，合并默认值
  return mergeWithDefaults(result)
}

// 合并默认值
function mergeWithDefaults(config: any): any {
  // 默认配置结构
  const defaults = {
    bot: {
      onJoinGroup: {
        rename: "【bot】"
      },
      sendFailureReminder: {
        enable: false,
        times: 3
      },
      offlineQueue: {
        enable: false,
        expire: "30m"
      }
    },
    bilibili: {
      SESSDATA: "",
      bili_jct: "",
      qrlogin: true,
      interval: "25s",
      imageMergeMode: "auto",
      hiddenSub: false,
      unsub: false,
      minFollowerCap: 0,
      disableSub: false,
      onlyOnlineNotify: false,
      autoParsePosts: false,
      secAnalysis: false
    },
    acfun: {
      account: "",
      password: "",
      unsub: false,
      interval: "25s",
      onlyOnlineNotify: false
    },
    twitter: {
      baseUrl: [
        "https://nitter.net/",
        "https://nitter.privacyredirect.com/",
        "https://nitter.tiekoetter.com/",
        "https://nitter.poast.org/"
      ],
      interval: "30s",
      userAgent: ""
    },
    douyin: {
      acSignature: "",
      acNonce: "",
      sessionId: "",
      userAgent: "",
      interval: "30s",
      onlyOnlineNotify: false
    },
    weibo: {
      onlyOnlineNotify: true,
      sub: ""
    },
    youtube: {
      onlyOnlineNotify: true
    },
    concern: {
      emitInterval: "5s"
    },
    template: {
      enable: true
    },
    autoreply: {
      group: {
        command: ["签到"]
      },
      private: {
        command: []
      }
    },
    customCommandPrefix: {
      "签到": ""
    },
    logLevel: "info",
    websocket: {
      mode: "ws-server",
      token: "",
      "ws-server": "0.0.0.0:15630",
      "ws-reverse": "ws://localhost:3001"
    },
    admin: {
      enable: false,
      addr: "127.0.0.1:15631",
      token: ""
    },
    reloadDelay: {
      enable: true,
      time: "3s"
    },
    extDb: {
      enable: false,
      path: ".ext.db"
    },
    telegram: {
      enable: false,
      token: "",
      proxy: {
        enable: false,
        url: ""
      },
      endpoint: ""
    }
  }
  
  // 递归合并配置
  function deepMerge(target: any, source: any): any {
    const result = { ...target }
    
    for (const key in source) {
      if (source.hasOwnProperty(key)) {
        if (source[key] !== null && typeof source[key] === 'object') {
          if (Array.isArray(source[key])) {
            // 对于数组，直接使用源数组
            result[key] = source[key]
          } else {
            // 对于对象，递归合并
            if (!result[key]) {
              result[key] = {}
            }
            result[key] = deepMerge(result[key], source[key])
          }
        } else {
          // 对于基本类型，直接使用源值
          result[key] = source[key]
        }
      }
    }
    
    return result
  }
  
  return deepMerge(defaults, config)
}

// 解析值
function parseValue(value: string): any {
  value = value.trim()
  
  if (value === 'true') return true
  if (value === 'false') return false
  if (!isNaN(Number(value))) return Number(value)
  if (value.startsWith('[') && value.endsWith(']')) {
    // 简单数组解析
    return value.slice(1, -1).split(',').map(item => item.trim())
  }
  if (value.startsWith('"') && value.endsWith('"')) {
    return value.slice(1, -1)
  }
  return value
}

// 生成YAML
function generateYAML(obj: any, indent: number = 0): string {
  const spaces = '  '.repeat(indent)
  let result = ''
  
  for (const [key, value] of Object.entries(obj)) {
    if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
      result += `${spaces}${key}:\n`
      result += generateYAML(value, indent + 1)
    } else if (Array.isArray(value)) {
      result += `${spaces}${key}:\n`
      for (const item of value) {
        result += `${spaces}  - ${item}\n`
      }
    } else {
      result += `${spaces}${key}: ${value}\n`
    }
  }
  
  return result
}

// 监控配置变化
watch(
  () => config.value,
  () => {
    hasUnsavedChanges.value = JSON.stringify(config.value) !== JSON.stringify(originalConfig.value)
  },
  { deep: true }
)

// 初始化
onMounted(() => {
  loadConfig()
})
</script>

<style scoped>
.graphical-config-editor {
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

.card__footer {
  padding: 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
  background: rgba(0, 0, 0, 0.15);
}

.actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.editor-container {
  display: flex;
  flex-direction: row;
  min-height: 600px;
  max-height: calc(100vh - 240px);
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
  overflow: hidden;
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

.config-form {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.config-section {
  margin-bottom: 20px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.07);
  background: rgba(255, 255, 255, 0.02);
  overflow: hidden;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: rgba(255, 255, 255, 0.04);
  cursor: pointer;
  transition: background 0.2s;
}

.section-header:hover {
  background: rgba(255, 255, 255, 0.06);
}

.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  font-size: 14px;
}

.section-toggle {
  font-size: 10px;
  opacity: 0.6;
}

.section-content {
  padding: 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
}

.config-group {
  margin-bottom: 24px;
}

.config-group h4 {
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  color: rgba(232, 234, 240, 0.9);
}

.config-item {
  margin-bottom: 16px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.config-label {
  font-size: 13px;
  font-weight: 500;
  color: rgba(232, 234, 240, 0.8);
}

.config-control {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.config-hint {
  font-size: 11px;
  color: rgba(232, 234, 240, 0.5);
  margin-top: 2px;
}

.array-item {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 8px;
}

.array-item Input {
  flex: 1;
}

.editor-status {
  display: flex;
  justify-content: space-between;
  align-items: center;
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