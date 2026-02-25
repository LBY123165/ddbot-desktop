# DDBOT-WSa WebUI

一个现代化的 Web 管理界面，用于管理 DDBOT-WSa 机器人服务。

## 架构说明

本项目采用纯前端架构，直接与 DDBOT-WSa 的 Admin API 交互：

- **前端**: Vue 3 + TypeScript + Vite
- **后端**: DDBOT-WSa 自带的 Admin API (默认端口 15631)
- **零依赖**: 摒弃了原有的 Rust 中间件，直接通过前端浏览器与机器人本体通信。

## 特性

✅ 纯 Web 架构，跨平台访问
✅ 实时状态监控 (OneBot 连接状态、订阅信息等)
✅ 配置文件编辑与热重载无缝集成
✅ 可视化订阅管理 (添加、编辑群组订阅)
✅ 响应式设计

## 开发环境

### 前置要求

- Node.js 18+

### 开发模式

```bash
# 安装依赖
npm install

# 启动前端开发服务器 (监听在 http://localhost:3000)
npm run dev
```

### 生产部署 (发布)

完整的 WebUI 是由“前端静态构建” + “Node.js 守护进程”两部分组成的，在生产环境中，请按照以下步骤启动：

1. **编译打包前端静态网页：**
```bash
# 在项目根目录下执行
npm run build
```

2. **启动 Node.js 服务端服务：**
```bash
cd backend-js
# 安装后端所需依赖 (首次运行需要)
npm install
# 直接启动
node index.js
```

> **推荐方案**：使用 `pm2` 来守护后台进程，确保 WebUI 就算关掉终端也能持久运行并在开机自启：
```bash
npm install -g pm2
cd backend-js
pm2 start index.js --name "ddbot-webui"
```

启动后，无论是内网还是公网，直接访问 `http://服务器IP:3000` 即可全栈使用。

## 目录结构

```
DDBOT-WSa-Desktop/
├── src/                  # Vue 前端源码
│   ├── api/             # API 交互层 (tauri.ts 封装了 Go Admin API)
│   ├── components/       # Vue 组件
│   ├── pages/           # 页面组件
│   ├── stores/          # 状态管理
│   └── router/          # 路由配置
├── dist/                 # 前端构建输出 (npm run build 生成)
├── vite.config.ts        # Vite 构建/服务配置
└── package.json          # Node.js 配置
```

## 访问地址

开发服务启动后可通过以下地址访问 WebUI：
- `http://localhost:3000`
- `http://127.0.0.1:3000`

若要允许局域网设备访问，Vite 的 `host: true` 配置已默认开启，可通过主机的局域网 IP 访问。

## 常见问题 / 故障排除

### WebUI 无法获取数据 / 一直显示未连接
1. 确认 DDBOT-WSa 主程序的 `application.yaml` 中已开启 `admin.enable: true`。
2. 确认主程序 Admin API 端口配置正确 (默认应为 `127.0.0.1:15631`)。
3. 检查并确认 DDBOT-WSa 后台进程是否处于运行状态。

### npm run dev 无法访问
1. 如果端口 3000 被其他程序占用，Vite 将会报错，请换用其他端口或结束占用进程。
2. Windows 下如果提示 `npm.ps1 无法运行`，请尝试在 `cmd` (而非 PowerShell) 下运行，或在 PowerShell 中执行 `Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy RemoteSigned` 解除安全限制。

## 许可证

MIT License