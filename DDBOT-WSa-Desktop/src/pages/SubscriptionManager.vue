<template>
  <div class="subscription-manager">
    <div class="header">
      <div>
        <h2 class="title">订阅管理</h2>
        <p class="subtitle">查看和编辑 Bot 当前的所有订阅</p>
      </div>
      <Button :icon="Plus" variant="primary" @click="showAddModal = true">添加订阅</Button>
      <Button :icon="RefreshCw" @click="fetchSubscriptions">刷新列表</Button>
    </div>

    <!-- Error message -->
    <div v-if="error" class="error-banner">
      {{ error }}
      <Button size="sm" variant="danger" @click="error = ''">关闭</Button>
    </div>

    <div class="card">
      <div v-if="loading" class="empty-state">
        <div>正在加载订阅信息...</div>
      </div>
      <div v-else-if="subscriptions.length === 0" class="empty-state">
        <div>暂无订阅数据或无法连接到 Bot。</div>
      </div>
      <div class="table-container" v-else>
        <table class="subs-table">
          <thead>
            <tr>
              <th>群组 (Group Code)</th>
              <th>站点 (Site)</th>
              <th>账号 ID</th>
              <th>账号名称</th>
              <th>订阅类型 (Type)</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(sub, index) in subscriptions" :key="index">
              <td class="mono">{{ sub.groupCode }}</td>
              <td>{{ sub.site }}</td>
              <td class="mono font-semibold">{{ sub.id }}</td>
              <td>{{ sub.name || '-' }}</td>
              <td>
                <span class="type-badge">{{ sub.type }}</span>
              </td>
              <td>
                <Button size="sm" variant="secondary" @click="openSettings(sub)" class="mr-2">设置</Button>
                <Button size="sm" variant="danger" @click="removeSub(sub)">删除</Button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Add Subscription Modal -->
    <div v-if="showAddModal" class="modal-overlay">
      <div class="modal">
        <h3 class="modal-title">添加订阅</h3>
        <div class="form-group">
          <label>群组 (Group Code)</label>
          <input type="number" v-model.number="newSub.groupCode" class="input" placeholder="例如: 123456789" />
        </div>
        <div class="form-group">
          <label>站点</label>
          <select v-model="newSub.site" class="input">
            <option value="bilibili">bilibili</option>
            <option value="acfun">acfun</option>
            <option value="douyin">douyin</option>
            <option value="weibo">weibo</option>
            <option value="youtube">youtube</option>
            <option value="twitter">twitter</option>
          </select>
        </div>
        <div class="form-group">
          <label>账号 ID</label>
          <input type="text" v-model="newSub.id" class="input" placeholder="例如: 226257459" />
        </div>
        <div class="form-group">
          <label>订阅类型</label>
          <select v-model="newSub.type" class="input">
            <option value="news">动态/文章/视频/博文 (news)</option>
            <option value="live">直播 (live)</option>
          </select>
        </div>
        
        <div class="modal-actions">
          <Button variant="secondary" @click="showAddModal = false">取消</Button>
          <Button variant="primary" @click="addSub">确定添加</Button>
        </div>
      </div>
    </div>

    <!-- Subscription Settings Modal -->
    <div v-if="showSettingsModal" class="modal-overlay">
      <div class="modal modal-lg">
        <h3 class="modal-title">订阅推送设置</h3>
        
        <div class="info-box mb-4">
          <strong>配置填写说明：</strong><br/>
          - <strong>推送场景</strong>由多个类型组成，如 <code>news</code>(动态/文章) 或 <code>live</code>(直播)。若要同时匹配，用 <code>/</code> 分隔：<code>news/live</code>。<br/>
          - <strong>@特定人</strong>：填写QQ号，多个使用半角逗号 <code>,</code> 隔开。<br/>
          - <strong>文本过滤器</strong>必须符合严格的 JSON 格式结构。支持 <code>text</code> (包含至少某个词) 和 <code>not_text</code> (不包含任何这些词)。示例：<br/>
          <pre class="code-sm"><code>[
  {"type":"not_text", "config":"{\"text\":[\"屏蔽关键词1\", \"广告\"]}"},
  {"type":"text", "config":"{\"text\":[\"必须包含的词\"]}"}
]</code></pre>
        </div>

        <div v-if="configLoading" class="empty-state">加载中...</div>
        <div v-else class="settings-grid">
          <div class="form-group">
            <label>@全体 推送场景 (如: news/live, news, live等)</label>
            <input type="text" v-model="editConfig.group_concern_at.at_all" class="input" placeholder="留空为不艾特" />
          </div>
          
          <div class="form-group">
            <label>@特定人 QQ号 (多个用逗号隔开)</label>
            <input type="text" v-model="atSomeoneText" class="input" placeholder="例如: 12345,67890" />
            <p class="text-xs mt-1 opacity-70">在哪些场景艾特他们，由群管理员指令额外调整</p>
          </div>
          
          <div class="form-group">
            <label>直播间改标题时 推送场景</label>
            <input type="text" v-model="editConfig.group_concern_notify.title_change_notify" class="input" placeholder="例如: live 或 news 或者空" />
          </div>
          
          <div class="form-group">
            <label>主播下播时 推送场景</label>
            <input type="text" v-model="editConfig.group_concern_notify.offline_notify" class="input" placeholder="例如: live 或 news 或者空" />
          </div>
          
          <div class="form-group">
            <label>特别提醒 推送场景 (extend_notify)</label>
            <input type="text" v-model="editConfig.group_concern_notify.extend_notify" class="input" placeholder="例如: news" />
          </div>

          <div class="form-group span-full">
            <label>文本过滤器 (仅允许包含或不包含某关键字时的推送) [JSON模式]</label>
            <textarea v-model="filterRulesText" class="input" rows="4" placeholder='[{"type":"not_text","config":"{\"text\":[\"黑名单词\"]}"}]'></textarea>
            <p class="text-xs mt-1 opacity-70 text-right"><a href="https://github.com/cnxysoft/DDBOT-WSa" target="_blank" class="underline">查阅过滤器编写规范</a></p>
          </div>
        </div>
        
        <div class="modal-actions">
          <Button variant="secondary" @click="showSettingsModal = false">取消</Button>
          <Button variant="primary" @click="saveSettings">保存配置</Button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Plus, RefreshCw } from 'lucide-vue-next'
import Button from '../components/Button.vue'

interface SubInfo {
  id: any
  name?: string
  type: string
  site: string
  groupCode: number
}

const subscriptions = ref<SubInfo[]>([])
const loading = ref(false)
const error = ref('')
const showAddModal = ref(false)

const newSub = ref<SubInfo>({
  id: '',
  site: 'bilibili',
  type: 'news',
  groupCode: 0
})

const showSettingsModal = ref(false)
const configLoading = ref(false)
const activeSubForConfig = ref<SubInfo | null>(null)
const editConfig = ref<any>({
  group_concern_at: { at_all: '', at_someone: [] },
  group_concern_notify: { title_change_notify: '', offline_notify: '', extend_notify: '' },
  group_concern_filter: { type: '', config: '', rules: [] }
})
const atSomeoneText = ref('')
const filterRulesText = ref('[]')

const API_BASE = '/api/v1'
const ADMIN_API_BASE = '/api/admin'

const fetchSubscriptions = async () => {
  try {
    loading.value = true
    error.value = ''
    const res = await fetch(`${API_BASE}/subs/list`)
    if (res.ok) {
      const data = await res.json()
      subscriptions.value = Array.isArray(data) ? data : []
    } else {
      throw new Error(`Failed to load subscriptions: ${res.statusText}`)
    }
  } catch (e: any) {
    error.value = `请求失败, 也许 DDBOT 未运行或者管理员 API 未启用。错误: ${e.message}`
    subscriptions.value = []
  } finally {
    loading.value = false
  }
}

const addSub = async () => {
  if (!newSub.value.id || !newSub.value.groupCode) {
    alert("请输入各项信息")
    return
  }
  try {
    const res = await fetch(`${API_BASE}/subs/add`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
         site: newSub.value.site,
         id: isNaN(Number(newSub.value.id)) ? newSub.value.id : Number(newSub.value.id),
         type: newSub.value.type,
         groupCode: newSub.value.groupCode
      })
    })
    
    if (res.ok) {
      showAddModal.value = false
      newSub.value.id = ''
      await fetchSubscriptions()
    } else {
      const errData = await res.json()
      alert(`添加失败: ${errData.error || res.statusText}`)
    }
  } catch (e: any) {
    alert(`网络错误: ${e.message}`)
  }
}

const removeSub = async (sub: SubInfo) => {
  if (!confirm(`确定要删除位于群 ${sub.groupCode} 的 ${sub.site} 订阅 (ID:${sub.id}) 吗？`)) return;
  try {
    const res = await fetch(`${API_BASE}/subs/remove`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
         site: sub.site,
         id: sub.id,
         type: sub.type,
         groupCode: sub.groupCode
      })
    })
    
    if (res.ok) {
      await fetchSubscriptions()
    } else {
      const errData = await res.json()
      alert(`删除失败: ${errData.error || res.statusText}`)
    }
  } catch (e: any) {
    alert(`网络错误: ${e.message}`)
  }
}

const openSettings = async (sub: SubInfo) => {
  activeSubForConfig.value = sub
  showSettingsModal.value = true
  configLoading.value = true
  
  try {
    const url = new URL(`${ADMIN_API_BASE}/sub/config`)
    url.searchParams.append('site', sub.site)
    url.searchParams.append('id', typeof sub.id === 'string' || typeof sub.id === 'number' ? String(sub.id) : '')
    url.searchParams.append('groupCode', String(sub.groupCode))

    const res = await fetch(url.toString())
    if (res.ok) {
      const data = await res.json()
      editConfig.value = {
        group_concern_at: { at_all: '', at_someone: [], ...(data.group_concern_at || {}) },
        group_concern_notify: { title_change_notify: '', offline_notify: '', extend_notify: '', ...(data.group_concern_notify || {}) },
        group_concern_filter: { type: '', config: '', rules: [], ...(data.group_concern_filter || {}) },
      }
      
      // Parse map fields for UI
      if (editConfig.value.group_concern_at.at_someone && Array.isArray(editConfig.value.group_concern_at.at_someone)) {
        atSomeoneText.value = editConfig.value.group_concern_at.at_someone.join(',')
      } else {
        atSomeoneText.value = ''
      }
      
      if (editConfig.value.group_concern_filter.rules) {
        filterRulesText.value = JSON.stringify(editConfig.value.group_concern_filter.rules, null, 2)
      } else {
        filterRulesText.value = '[]'
      }
    } else {
      throw new Error(`无法获取配置: ${res.statusText}`)
    }
  } catch (e: any) {
    alert(e.message)
    showSettingsModal.value = false
  } finally {
    configLoading.value = false
  }
}

const saveSettings = async () => {
  if (!activeSubForConfig.value) return
  
  // pack text items into struct array
  const someoneArr = atSomeoneText.value.split(',').map(s => s.trim()).filter(Boolean).map(Number)
  editConfig.value.group_concern_at.at_someone = someoneArr
  
  try {
    editConfig.value.group_concern_filter.rules = JSON.parse(filterRulesText.value || '[]')
  } catch(e) {
    alert("过滤器 JSON 格式错误")
    return
  }

  try {
    const res = await fetch(`${ADMIN_API_BASE}/sub/config`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
         site: activeSubForConfig.value.site,
         id: activeSubForConfig.value.id,
         type: activeSubForConfig.value.type,
         groupCode: activeSubForConfig.value.groupCode,
         config: editConfig.value
      })
    })
    
    if (res.ok) {
      showSettingsModal.value = false
      alert("配置保存成功")
    } else {
      const errData = await res.json()
      alert(`保存失败: ${errData.error || res.statusText}`)
    }
  } catch (e: any) {
    alert(`网络错误: ${e.message}`)
  }
}

onMounted(() => {
  fetchSubscriptions()
})
</script>

<style scoped>
.subscription-manager {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.header {
  display: flex;
  align-items: center;
  gap: 16px;
  background: var(--bg-card);
  padding: 20px;
  border-radius: 16px;
  border: 1px solid var(--border-color);
}

.header > div {
  flex: 1;
}

.title {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
}

.subtitle {
  margin: 4px 0 0 0;
  font-size: 13px;
  opacity: 0.7;
}

.card {
  background: var(--bg-card);
  border-radius: 16px;
  border: 1px solid var(--border-color);
  padding: 0;
  overflow: hidden;
}

.empty-state {
  padding: 40px;
  text-align: center;
  color: var(--text-secondary);
}

.table-container {
  overflow-x: auto;
}

.settings-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  max-height: 60vh;
  overflow-y: auto;
  padding: 4px;
}
.span-full {
  grid-column: 1 / -1;
}
.modal-lg {
  max-width: 900px !important;
  width: 95%;
}
.mr-2 {
  margin-right: 8px;
}

.subs-table {
  width: 100%;
  border-collapse: collapse;
}

.subs-table th,
.subs-table td {
  padding: 12px 16px;
  text-align: left;
  border-bottom: 1px solid var(--border-color);
}

.subs-table th {
  background: var(--bg-card);
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
}

.subs-table td {
  font-size: 14px;
}

.mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}
.font-semibold {
  font-weight: 600;
}

.type-badge {
  display: inline-block;
  padding: 4px 8px;
  background: rgba(124, 92, 255, 0.15);
  color: #c9bcff;
  border-radius: 6px;
  font-size: 12px;
}

.error-banner {
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #fca5a5;
  padding: 12px 16px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--bg-secondary);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  width: 90%;
  max-width: 400px;
  border-radius: 16px;
  padding: 24px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
}

.modal-title {
  margin: 0 0 20px 0;
  font-size: 18px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  font-size: 13px;
  opacity: 0.8;
  margin-bottom: 6px;
}

.input {
  width: 100%;
  padding: 10px 12px;
  border-radius: 8px;
  border: 1px solid var(--border-color);
  background: var(--bg-card);
  color: var(--text-primary);
  font-family: inherit;
  font-size: 14px;
  box-sizing: border-box;
}

.input:focus {
  outline: none;
  border-color: var(--accent-color);
}

.info-box {
  background: rgba(124, 92, 255, 0.1);
  border-left: 4px solid var(--accent-color);
  padding: 12px;
  border-radius: 4px;
  font-size: 13px;
  color: var(--text-secondary);
  word-wrap: break-word;
  white-space: normal;
}

.info-box strong {
  color: var(--text-primary);
}

.code-sm {
  background: var(--bg-card);
  padding: 8px;
  border-radius: 4px;
  margin-top: 4px;
  font-family: ui-monospace, monospace;
  font-size: 11px;
  white-space: pre-wrap;
  word-break: break-all;
  overflow-x: auto;
}
.mb-4 {
  margin-bottom: 16px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 24px;
}
</style>
