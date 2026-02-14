# DDBOT-WSa Desktop WebUI — Project Status

## 目标（Objective）
- 构建一个类似 openlist-desktop 的 **桌面端 WebUI**（Tauri v2 + Rust 后端 + Vue3 前端）。
- 桌面端可以：
  - 启动/停止/重启 DDBOT-WSa
  - 管理配置文件（application.yaml 等）
  - 展示运行状态：
    - 本体进程状态
    - OneBot 连接状态
    - 运行时长
    - 订阅统计/推送统计/模板加载状态（逐步补齐）
- 核心数据策略：
  - **在线（本体运行）只走本地只读 Admin API**
  - **离线（本体不运行）才读 DB**，并且必须拿到锁后只读打开，避免破坏数据。

---

## 当前架构/约定（Key Design）

### 1) Desktop 数据隔离（强约束）
- Desktop 永远在自己的数据目录运行，避免破坏用户原部署。
- Desktop 会把用户选择的“已有部署目录”复制到 Desktop 数据目录（**不复制原 exe**），然后下载最新版 DDBOT-WSa 可执行文件运行。

### 2) Admin API
- 本体侧提供本地只读 HTTP API，默认只监听 `127.0.0.1`。
- 默认配置中 `admin.enable=false`，Desktop 在用户“批准/授权”后自动开启并生成随机 token。

---

## 已完成内容（Done）

### A. DDBOT-WSa 本体侧
- 已新增本地只读 Admin API：
  - 文件：`DDBOT-WSa/admin/server.go`
  - 端点：
    - `GET /api/v1/health`
    - `GET /api/v1/onebot/status`
    - `GET /api/v1/subs/summary`
  - Token 鉴权：`Authorization: Bearer <token>`（当 `admin.token` 非空时启用）。

- 启动时拉起 Admin API：
  - 文件：`DDBOT-WSa/bot.go`
  - 位置：`bot.StartService()` 之后调用：
    - `admin.Start(&bot.Instance.Online, nil)`

- 默认配置（exampleConfig）已包含 admin 配置段（默认关闭）：
  - 文件：`DDBOT-WSa/bot.go` 中 `exampleConfig` 字符串
  - 配置：
    - `admin.enable: false`
    - `admin.addr: "127.0.0.1:15631"`
    - `admin.token: ""`

### B. Desktop（Tauri Rust 后端）

#### 1) 数据目录与路径约定
- 文件：`DDBOT-WSa-Desktop/src-tauri/src/ddbot.rs`
- 约定：
  - `workdir()`：`%AppData%/DDBOT-WSa-Desktop/`
  - `data_dir()`：`workdir()/data/`
  - `managed_ddbot_dir()`：`data_dir()/ddbot/`（运行时 cwd）
  - `config_path()`：`managed_ddbot_dir()/application.yaml`
  - `approval_marker_path()`：`data_dir()/approved.json`（批准文件）

#### 2) 运行时 cwd 已固定到 managed 目录
- 文件：`DDBOT-WSa-Desktop/src-tauri/src/process.rs`
- `process::start()`：
  - `cmd.current_dir(ddbot::managed_ddbot_dir())`

#### 3) “已部署过”导入逻辑（校验 + 复制排除 exe）
- 文件：`DDBOT-WSa-Desktop/src-tauri/src/ddbot.rs`
- 校验规则（必须存在）：
  - `DDBOT.exe`（或非 Windows 下 `DDBOT`）
  - `application.yaml`
  - `.lsp.db`
- 复制规则：
  - 递归复制用户选择目录到 `managed_ddbot_dir()`
  - **跳过原本体可执行文件（不复制 exe）**

#### 4) 导入后自动下载最新版 DDBOT-WSa
- 文件：`DDBOT-WSa-Desktop/src-tauri/src/main.rs`
- 命令：`import_existing_deployment(src_dir)`
  - 先复制（排除 exe）
  - 再 `ensure_installed()` 下载最新版 release 资产并安装到 Desktop `binary/`

#### 5) 批准文件机制 + 自动开启 admin + 生成随机 token
- 文件：`DDBOT-WSa-Desktop/src-tauri/src/ddbot.rs`
- 逻辑：
  - `enable_admin_in_config_if_approved()`
    - 仅当 `approved.json` 存在才修改 `application.yaml`
    - 写入 `admin.enable=true`、`admin.addr=127.0.0.1:15631`
    - 若 `admin.token` 为空则生成 48 位随机 token 并写回
  - `read_admin_token()`：只读解析 token
- `process::start()` 启动前会调用 `enable_admin_in_config_if_approved()`（若已批准）。

#### 6) Tauri 命令暴露
- 文件：`DDBOT-WSa-Desktop/src-tauri/src/main.rs`
- 已新增：
  - `get_managed_ddbot_dir()`
  - `is_user_approved()`
  - `set_user_approved(approved)`
  - `import_existing_deployment(src_dir)`（已 async）

#### 7) Rust 依赖补齐
- 文件：`DDBOT-WSa-Desktop/src-tauri/Cargo.toml`
- 增加：`serde_yaml`、`rand`、`walkdir`

---

## 当前未完成（Not Done Yet）

### 1) 前端“首次运行向导”（非常关键）
需要实现 UI 流程：
- 是否已部署过 DDBOT-WSa？
  - 已部署：选择目录 -> 展示提醒文案 -> 调用 `import_existing_deployment`
  - 未部署：仅下载最新版到 Desktop 数据目录并创建最小配置（待定具体策略）
- 用户授权/批准步骤：
  - 写入 `approved.json`
  - 然后 Desktop 自动开启 admin 并生成 token

### 2) Desktop 真数据：用 token 调 Admin API
目前 `onebot_status_text()` / `subs_summary_text()` 仍为占位 `"待实现"`。
需要改为：
- 用 `reqwest` 请求：
  - `/api/v1/onebot/status`
  - `/api/v1/subs/summary`
- Header：`Authorization: Bearer <token>`
- token 获取：
  - 优先内存缓存（本次 start 时生成/读取）
  - 其次从 `application.yaml` 解析（`read_admin_token()`）

### 3) “离线读 DB” fallback（在线不碰 DB）
尚未实现：
- Desktop 离线时读取 `.lsp.db` 的只读方案
- 必须先拿到 `.lsp.db.lock`（同一把锁）
- 拿不到锁：不读，避免破坏

### 4) 其它 MVP
- 配置编辑器、日志查看、扫码 cookie、模板中心

---

## 需要你确认的事项（Decisions Needed From You）

### 1) 批准/授权的触发时机
- A：导入后弹窗明确授权，用户确认才写 `approved.json` 并开启 admin（更安全）
- B：用户选择目录并点击“导入”就视为授权，立即写 `approved.json` 并开启 admin（更省一步）

### 2) 未部署用户的默认配置策略
当用户选择“未部署过”时：
- 是否由 Desktop 自动生成一个包含 `admin` 段（默认关闭）的最小 `application.yaml`？
- 还是直接复制 DDBOT-WSa 上游 `exampleConfig` 作为初始配置？

### 3) 导入覆盖策略
当 `managed_ddbot_dir()` 已存在内容时：
- 覆盖导入
- 或创建一个带时间戳的新目录并切换（更安全）

---

## 关键文件一览（Where to Look）

### DDBOT-WSa（Go）
- `DDBOT-WSa/admin/server.go`：Admin API
- `DDBOT-WSa/bot.go`：启动流程 + exampleConfig

### Desktop（Rust/Tauri）
- `DDBOT-WSa-Desktop/src-tauri/src/ddbot.rs`：路径约定/导入/开启 admin/token
- `DDBOT-WSa-Desktop/src-tauri/src/process.rs`：进程管理（cwd=managed dir）
- `DDBOT-WSa-Desktop/src-tauri/src/main.rs`：Tauri commands

---

## 下一步建议（Next Steps）
1. 做前端“首次运行向导”页面（目录选择 + 提示文案 + 导入/下载动作 + 授权按钮）。
2. 实现 Admin API 调用（带 token）并填充 HomePage 的 OneBot/订阅统计。
3. 实现离线 DB 只读 fallback（严格拿锁）。
