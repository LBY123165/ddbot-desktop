import { createRouter, createWebHashHistory } from 'vue-router'

import HomePage from '../pages/HomePage.vue'
import SettingsPage from '../pages/SettingsPage.vue'
import SetupWizard from '../pages/SetupWizard.vue'
import ConfigEditor from '../pages/ConfigEditor.vue'
import GraphicalConfigEditor from '../pages/GraphicalConfigEditor.vue'
import LogsPage from '../pages/LogsPage.vue'
import TemplateCenter from '../pages/TemplateCenter.vue'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: HomePage,
    meta: { title: '概览' },
  },
  {
    path: '/config',
    name: 'Config',
    component: GraphicalConfigEditor,
    meta: { title: '配置编辑' },
  },
  {
    path: '/config/text',
    name: 'TextConfig',
    component: ConfigEditor,
    meta: { title: '文本配置编辑' },
  },
  {
    path: '/logs',
    name: 'Logs',
    component: LogsPage,
    meta: { title: '日志查看' },
  },
  {
    path: '/templates',
    name: 'Templates',
    component: TemplateCenter,
    meta: { title: '模板中心' },
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
