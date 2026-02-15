<template>
  <div class="app">
    <Navigation />
    <main class="main">
      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { RouterView } from 'vue-router'
import { useRouter } from 'vue-router'
import Navigation from './components/NavigationPage.vue'
import { useAppStore } from './stores/app'

const appStore = useAppStore()
const router = useRouter()

onMounted(async () => {
  await appStore.init()
  // WebUI 模式下不需要检查授权状态
})
</script>

<style scoped>
.app {
  display: flex;
  min-height: 100vh;
  color: #e8eaf0;
  background: radial-gradient(1200px 700px at 10% 0%, rgba(124, 92, 255, 0.18), transparent 60%),
    radial-gradient(900px 600px at 90% 10%, rgba(0, 214, 160, 0.14), transparent 55%),
    linear-gradient(180deg, #0b1020, #070a14);
}

.main {
  flex: 1;
  padding: 22px;
  max-width: 1400px;
  margin: 0 auto;
  width: 100%;
}
</style>
