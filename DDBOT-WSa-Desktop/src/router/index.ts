import { createRouter, createWebHashHistory } from 'vue-router'

import HomePage from '../pages/HomePage.vue'
import SettingsPage from '../pages/SettingsPage.vue'
import SetupWizard from '../pages/SetupWizard.vue'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: HomePage,
    meta: { title: '概览' },
  },
  {
    path: '/settings',
    name: 'Settings',
    component: SettingsPage,
    meta: { title: '设置' },
  },
  {
    path: '/setup',
    name: 'Setup',
    component: SetupWizard,
    meta: { title: '初始设置' },
  },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

export default router
