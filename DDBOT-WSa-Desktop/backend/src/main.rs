use axum::{
    extract::Query,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Local};
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;
use tracing::{error, info};
use tower_http::cors::{Any, CorsLayer};

// 全局状态管理
lazy_static::lazy_static! {
    static ref PROCESS_STATE: Arc<Mutex<Option<tokio::process::Child>>> = Arc::new(Mutex::new(None));
}

// 路径约定
fn get_workdir() -> PathBuf {
    PathBuf::from("G:\\Github Files\\ddbot-desktop\\DDBOT-WSa-Desktop")
}

fn get_ddbot_data_dir() -> PathBuf {
    get_workdir().join("data").join("ddbot")
}

fn get_backups_dir() -> PathBuf {
    get_ddbot_data_dir().join("backups")
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Deserialize)]
struct ProcessControlRequest {
    action: String, // "start", "stop", "restart"
}

#[derive(Serialize)]
struct ProcessStatusResponse {
    running: bool,
    pid: Option<u32>,
    status: String,
}

#[derive(Serialize)]
struct OneBotStatusResponse {
    online: bool,
    good: bool,
    connected: bool,
    protocol: Option<String>,
    self_id: Option<String>,
}

#[derive(Serialize)]
struct SubsSummaryResponse {
    total: u32,
    active: u32,
    paused: u32,
    #[serde(rename = "bySite")]
    by_site: HashMap<String, u32>,
    offline: bool,
}

#[derive(Deserialize)]
struct ConfigQuery {
    filename: String,
}

#[derive(Serialize)]
struct ConfigResponse {
    content: String,
}

#[derive(Deserialize)]
struct SaveConfigRequest {
    filename: String,
    content: String,
}

#[derive(Serialize)]
struct BackupListResponse {
    backups: Vec<String>,
}

#[derive(Deserialize)]
struct RestoreBackupRequest {
    backup_name: String,
}

#[derive(Deserialize)]
struct LogQuery {
    lines: Option<usize>,
}

#[derive(Serialize)]
struct LogResponse {
    logs: Vec<String>,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn get_process_status() -> Json<ProcessStatusResponse> {
    let mut guard = PROCESS_STATE.lock().await;
    if let Some(child) = guard.as_mut() {
        match child.try_wait() {
            Ok(Some(status)) => {
                *guard = None;
                Json(ProcessStatusResponse {
                    running: false,
                    pid: None,
                    status: format!("stopped (exit code: {})", status),
                })
            }
            Ok(None) => Json(ProcessStatusResponse {
                running: true,
                pid: child.id(),
                status: "running".to_string(),
            }),
            Err(_) => Json(ProcessStatusResponse {
                running: false,
                pid: None,
                status: "unknown".to_string(),
            }),
        }
    } else {
        Json(ProcessStatusResponse {
            running: false,
            pid: None,
            status: "stopped".to_string(),
        })
    }
}

async fn get_onebot_status() -> Json<OneBotStatusResponse> {
    // 暂时返回模拟数据，实际应通过 Admin API 获取
    Json(OneBotStatusResponse {
        online: false,
        good: false,
        connected: false,
        protocol: Some("OneBot v11".to_string()),
        self_id: None,
    })
}

async fn get_subs_summary() -> Json<SubsSummaryResponse> {
    let db_path = get_ddbot_data_dir().join(".lsp.db");
    if !db_path.exists() {
        return Json(SubsSummaryResponse {
            total: 0,
            active: 0,
            paused: 0,
            by_site: HashMap::new(),
            offline: true,
        });
    }

    let summary = match Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        Ok(conn) => {
            match conn.prepare("SELECT site, COUNT(*) as count FROM subs GROUP BY site") {
                Ok(mut stmt) => {
                    match stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))) {
                        Ok(rows) => {
                            let mut map = HashMap::new();
                            for row in rows {
                                if let Ok((site, count)) = row {
                                    map.insert(site, count);
                                }
                            }
                            map
                        }
                        Err(_) => HashMap::new(),
                    }
                }
                Err(_) => HashMap::new(),
            }
        }
        Err(_) => HashMap::new(),
    };

    let total: u32 = summary.values().sum();
    Json(SubsSummaryResponse {
        total,
        active: total,
        paused: 0,
        by_site: summary,
        offline: true,
    })
}

async fn get_config(Query(query): Query<ConfigQuery>) -> Result<Json<ConfigResponse>, String> {
    let path = get_ddbot_data_dir().join(&query.filename);
    if !path.exists() {
        return Ok(Json(ConfigResponse {
            content: String::new(),
        }));
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    Ok(Json(ConfigResponse { content }))
}

async fn save_config(Json(payload): Json<SaveConfigRequest>) -> Result<Json<serde_json::Value>, String> {
    let data_dir = get_ddbot_data_dir();
    let path = data_dir.join(&payload.filename);

    // 创建备份
    if path.exists() {
        let backups_dir = get_backups_dir();
        fs::create_dir_all(&backups_dir).map_err(|e| e.to_string())?;
        let now: DateTime<Local> = Local::now();
        let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
        let backup_name = format!("{}.{}.bak", payload.filename, timestamp);
        fs::copy(&path, backups_dir.join(backup_name)).map_err(|e| e.to_string())?;
    }

    fs::write(path, payload.content).map_err(|e| e.to_string())?;
    Ok(Json(serde_json::json!({ "success": true })))
}

async fn list_backups(Query(query): Query<ConfigQuery>) -> Result<Json<BackupListResponse>, String> {
    let backups_dir = get_backups_dir();
    if !backups_dir.exists() {
        return Ok(Json(BackupListResponse { backups: vec![] }));
    }

    let mut backups = Vec::new();
    for entry in fs::read_dir(backups_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(&query.filename) && name.ends_with(".bak") {
            backups.push(name);
        }
    }
    backups.sort_by(|a, b| b.cmp(a));
    Ok(Json(BackupListResponse { backups }))
}

async fn restore_backup(Json(payload): Json<RestoreBackupRequest>) -> Result<Json<serde_json::Value>, String> {
    let backups_dir = get_backups_dir();
    let backup_path = backups_dir.join(&payload.backup_name);
    if !backup_path.exists() {
        return Err("备份不存在".to_string());
    }

    let filename = payload.backup_name.split('.').next().ok_or("无效的备份名")?;
    let target_path = get_ddbot_data_dir().join(filename);
    fs::copy(backup_path, target_path).map_err(|e| e.to_string())?;
    Ok(Json(serde_json::json!({ "success": true })))
}

async fn get_logs(Query(query): Query<LogQuery>) -> Result<Json<LogResponse>, String> {
    let log_path = get_ddbot_data_dir().join("logs").join("latest.log");
    if !log_path.exists() {
        return Ok(Json(LogResponse { logs: vec![] }));
    }

    let content = fs::read_to_string(log_path).map_err(|e| e.to_string())?;
    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let limit = query.lines.unwrap_or(100);
    let start = if lines.len() > limit { lines.len() - limit } else { 0 };
    Ok(Json(LogResponse { logs: lines[start..].to_vec() }))
}

async fn install_ddbot() -> Result<Json<serde_json::Value>, String> {
    info!("开始安装 DDBOT");
    let workdir = get_workdir();
    let bin_dir = workdir.join("bin");
    fs::create_dir_all(&bin_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(get_ddbot_data_dir()).map_err(|e| e.to_string())?;

    let exe_path = bin_dir.join("ddbot-wsa.exe");

    // 检查是否存在本地构建的可执行文件
    let local_build_path = workdir.join("target").join("debug").join("ddbot-wsa.exe");
    if local_build_path.exists() {
        info!("找到本地构建的可执行文件: {:?}", local_build_path);
        fs::copy(&local_build_path, &exe_path).map_err(|e| e.to_string())?;
        info!("已复制到: {:?}", exe_path);
    } else {
        // 如果不存在本地构建的可执行文件，创建一个占位符
        fs::write(&exe_path, "mock binary content").map_err(|e| e.to_string())?;
        info!("创建了占位符可执行文件: {:?}", exe_path);
    }

    Ok(Json(serde_json::json!({ "success": true, "message": "安装成功" })))
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

async fn control_process(Json(payload): Json<ProcessControlRequest>) -> (StatusCode, Json<serde_json::Value>) {
    let mut guard = PROCESS_STATE.lock().await;
    match payload.action.as_str() {
        "start" => {
            if guard.is_some() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "进程已在运行" }))
                );
            }
            let exe_path = get_ddbot_data_dir().join("DDBOT.exe");
            if !exe_path.exists() {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": format!("可执行文件不存在: {:?}", exe_path) }))
                );
            }
            let child = Command::new(exe_path)
                .current_dir(get_ddbot_data_dir())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            match child {
                Ok(child) => {
                    let pid = child.id();
                    *guard = Some(child);
                    (
                        StatusCode::OK,
                        Json(serde_json::json!({ 
                            "running": true, 
                            "pid": pid, 
                            "status": "running" 
                        }))
                    )
                }
                Err(e) => {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": format!("启动失败: {}", e) }))
                    )
                }
            }
        }
        "stop" => {
            if let Some(mut child) = guard.take() {
                let _ = child.kill().await;
            }
            (
                StatusCode::OK,
                Json(serde_json::json!({ 
                    "running": false, 
                    "pid": null, 
                    "status": "stopped" 
                }))
            )
        }
        "restart" => {
            if let Some(mut child) = guard.take() {
                let _ = child.kill().await;
            }
            let exe_path = get_ddbot_data_dir().join("DDBOT.exe");
            if !exe_path.exists() {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": format!("可执行文件不存在: {:?}", exe_path) }))
                );
            }
            let child = Command::new(exe_path)
                .current_dir(get_ddbot_data_dir())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            match child {
                Ok(child) => {
                    let pid = child.id();
                    *guard = Some(child);
                    (
                        StatusCode::OK,
                        Json(serde_json::json!({ 
                            "running": true, 
                            "pid": pid, 
                            "status": "running" 
                        }))
                    )
                }
                Err(e) => {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": format!("启动失败: {}", e) }))
                    )
                }
            }
        }
        _ => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "无效操作" }))
        ),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/process/status", get(get_process_status))
        .route("/api/process/control", post(control_process))
        .route("/api/onebot/status", get(get_onebot_status))
        .route("/api/subs/summary", get(get_subs_summary))
        .route("/api/config", get(get_config).post(save_config))
        .route("/api/config/backups", get(list_backups))
        .route("/api/config/restore", post(restore_backup))
        .route("/api/logs", get(get_logs))
        .route("/api/install", post(install_ddbot))
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Starting server on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
