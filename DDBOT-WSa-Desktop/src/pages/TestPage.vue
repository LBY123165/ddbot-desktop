<template>
  <div style="padding: 20px; color: white;">
    <h1>测试页面</h1>
    <p>如果你能看到这个页面，说明基本的 Vue + Tauri 环境是正常的。</p>
    <div style="margin-top: 20px;">
      <p>Store 状态: {{ initialized ? '已初始化' : '未初始化' }}</p>
      <p>平台: {{ platform }}</p>
      <p v-if="error" style="color: red;">错误: {{ error }}</p>
    </div>
    <button @click="testInvoke" style="margin-top: 20px; padding: 10px;">测试 Tauri 调用</button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const initialized = ref(false)
const platform = ref('未知')
const error = ref<string | null>(null)

async function testInvoke() {
  try {
    error.value = null
    const triple = await window.__TAURI__.core.invoke('get_platform_triple')
    platform.value = triple as string
    initialized.value = true
  } catch (e: any) {
    error.value = String(e)
    console.error('Tauri invoke error:', e)
  }
}
</script>
