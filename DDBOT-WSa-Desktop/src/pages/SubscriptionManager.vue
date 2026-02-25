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

const API_BASE = 'http://localhost:15631/api/v1'

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
  background: rgba(255, 255, 255, 0.04);
  padding: 20px;
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.09);
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
  background: rgba(255, 255, 255, 0.04);
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  padding: 0;
  overflow: hidden;
}

.empty-state {
  padding: 40px;
  text-align: center;
  color: rgba(255, 255, 255, 0.5);
}

.table-container {
  overflow-x: auto;
}

.subs-table {
  width: 100%;
  border-collapse: collapse;
}

.subs-table th,
.subs-table td {
  padding: 12px 16px;
  text-align: left;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.subs-table th {
  background: rgba(0, 0, 0, 0.2);
  font-size: 13px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.7);
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
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: #1a1b1e;
  border: 1px solid #2c2d33;
  width: 400px;
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
  border: 1px solid rgba(255, 255, 255, 0.15);
  background: rgba(0, 0, 0, 0.2);
  color: white;
  font-family: inherit;
  font-size: 14px;
  box-sizing: border-box;
}

.input:focus {
  outline: none;
  border-color: #7c5cff;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 24px;
}
</style>
