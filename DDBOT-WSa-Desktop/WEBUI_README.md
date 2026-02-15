# DDBOT-WSa WebUI

一个现代化的 Web 管理界面，用于管理 DDBOT-WSa 机器人服务。

## 架构说明

本项目采用前后端分离架构：

- **前端**: Vue 3 + TypeScript + Vite
- **后端**: Rust + Axum (RESTful API)
- **部署**: 可注册为系统服务自动运行

## 特性

✅ 纯 Web 架构，跨平台访问
✅ 系统服务自动启动
✅ 进程管理 (启动/停止/重启)
✅ 实时状态监控
✅ 配置文件编辑
✅ 响应式设计

## 开发环境

### 前置要求

- Node.js 18+
- Rust 1.70+
- Cargo

### 开发模式

```bash
# 同时启动前后端开发服务器
npm run dev:both

# 或分别启动
npm run dev          # 前端开发服务器 (http://localhost:5173)
npm run dev:backend  # 后端API服务器 (http://localhost:3000)
```

### 生产构建

```bash
# 构建前后端
npm run build

# 分别构建
npm run build:frontend
npm run build:backend
```

## 系统服务部署

### Windows

```powershell
# 以管理员身份运行
npm run service:install

# 卸载服务
npm run service:uninstall
```

### Linux

```bash
# 复制服务文件
sudo cp service/ddbot-webui.service /etc/systemd/system/

# 启用并启动服务
sudo systemctl enable ddbot-webui
sudo systemctl start ddbot-webui

# 查看状态
sudo systemctl status ddbot-webui
```

## API 接口

### 健康检查
```
GET /api/health
```

### 进程管理
```
GET /api/process/status     # 获取进程状态
POST /api/process/control   # 控制进程 (start/stop/restart)
```

### DDBOT 状态
```
GET /api/onebot/status      # OneBot 连接状态
GET /api/subs/summary       # 订阅统计信息
```

## 目录结构

```
DDBOT-WSa-Desktop/
├── backend/              # Rust 后端服务
│   ├── src/
│   │   └── main.rs       # 主程序入口
│   └── Cargo.toml        # Rust 依赖配置
├── src/                  # Vue 前端源码
│   ├── components/       # Vue 组件
│   ├── pages/           # 页面组件
│   ├── stores/          # 状态管理
│   └── router/          # 路由配置
├── service/              # 系统服务配置
│   ├── ddbot-webui.service      # Linux systemd 配置
│   └── ddbot-webui-windows.xml  # Windows 服务配置
├── scripts/              # 部署脚本
│   ├── install-service.ps1      # Windows 安装脚本
│   └── uninstall-service.ps1    # Windows 卸载脚本
├── dist/                 # 前端构建输出
└── package.json          # Node.js 配置
```

## 访问地址

服务启动后可通过以下地址访问：
- WebUI: `http://localhost:3000`
- API 文档: `http://localhost:3000/docs` (开发中)

## 安全说明

- 默认只监听本地地址 (127.0.0.1)
- 建议在生产环境中配置反向代理和SSL
- 可通过环境变量配置监听地址和端口

## 故障排除

### 服务无法启动
1. 检查端口 3000 是否被占用
2. 确认 Rust 环境配置正确
3. 查看服务日志文件

### WebUI 无法访问
1. 确认后端服务正在运行
2. 检查防火墙设置
3. 验证网络连接

## 许可证

MIT License