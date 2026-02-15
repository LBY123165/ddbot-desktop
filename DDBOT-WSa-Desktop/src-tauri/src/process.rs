use anyhow::{anyhow, Result};
use reqwest::header;
use serde::Deserialize;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use crate::ddbot;

static ADMIN_TOKEN: Mutex<Option<String>> = Mutex::const_new(None);

static CHILD: Mutex<Option<Child>> = Mutex::const_new(None);

#[derive(Debug, Deserialize)]
struct OneBotStatusResponse {
  connected: bool,
  protocol: Option<String>,
  self_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SubsSummaryResponse {
  total: u32,
  #[serde(rename = "bySite")]
  by_site: Option<std::collections::HashMap<String, u32>>,
}

async fn get_admin_token() -> Option<String> {
  // Try in-memory cache first
  {
    let guard = ADMIN_TOKEN.lock().await;
    if guard.is_some() {
      return guard.clone();
    }
  }

  // Fall back to reading from config
  if let Ok(Some(token)) = ddbot::read_admin_token() {
    let mut guard = ADMIN_TOKEN.lock().await;
    *guard = Some(token.clone());
    Some(token)
  } else {
    None
  }
}

async fn call_admin_api<T: for<'de> Deserialize<'de>>(endpoint: &str) -> Result<T> {
  let token = get_admin_token().await.ok_or_else(|| anyhow!("Admin token not available"))?;

  let client = reqwest::Client::new();
  let url = format!("http://127.0.0.1:15631/api/v1{}", endpoint);

  let response = client
    .get(&url)
    .header(header::AUTHORIZATION, format!("Bearer {}", token))
    .send()
    .await?
    .error_for_status()?;

  let data = response.json::<T>().await?;
  Ok(data)
}

pub async fn start() -> Result<()> {
  ddbot::ensure_installed().await?;
  ddbot::ensure_managed_dirs()?;

  // If the user approved desktop modifications, ensure admin API is enabled and token is available.
  if let Some(token) = ddbot::enable_admin_in_config_if_approved()? {
    let mut g = ADMIN_TOKEN.lock().await;
    *g = Some(token);
  }
  let bin = ddbot::binary_path();
  if !bin.exists() {
    return Err(anyhow!("binary not found: {}", bin.display()));
  }

  let mut guard = CHILD.lock().await;
  if let Some(child) = guard.as_mut() {
    if child.try_wait()?.is_none() {
      return Ok(());
    }
  }

  let mut cmd = Command::new(bin);
  cmd.current_dir(ddbot::managed_ddbot_dir());
  cmd.stdin(Stdio::null());
  cmd.stdout(Stdio::null());
  cmd.stderr(Stdio::null());

  let child = cmd.spawn()?;
  *guard = Some(child);
  Ok(())
}

pub async fn stop() -> Result<()> {
  let mut guard = CHILD.lock().await;
  if let Some(child) = guard.as_mut() {
    let _ = child.kill().await;
  }
  *guard = None;
  Ok(())
}

pub async fn restart() -> Result<()> {
  stop().await?;
  start().await?;
  Ok(())
}

pub async fn status_text() -> String {
  let mut guard = CHILD.lock().await;
  if let Some(child) = guard.as_mut() {
    match child.try_wait() {
      Ok(Some(status)) => {
        let code = status.code().map(|c| c.to_string()).unwrap_or_else(|| "-".to_string());
        *guard = None;
        format!("已退出 (code={})", code)
      }
      Ok(None) => {
        let pid = child.id().map(|v| v.to_string()).unwrap_or_else(|| "?".to_string());
        format!("运行中 (pid={})", pid)
      }
      Err(_) => "未知".to_string(),
    }
  } else {
    "未运行".to_string()
  }
}

pub async fn onebot_status_text() -> String {
  match call_admin_api::<OneBotStatusResponse>("/onebot/status").await {
    Ok(status) => {
      if status.connected {
        let protocol = status.protocol.as_deref().unwrap_or("OneBot v11");
        let self_id = status.self_id.as_deref().unwrap_or("未知");
        format!("已连接 ({}) - {}", protocol, self_id)
      } else {
        "未连接".to_string()
      }
    }
    Err(_) => "无法获取状态".to_string(),
  }
}

pub async fn subs_summary_text() -> String {
  match call_admin_api::<SubsSummaryResponse>("/subs/summary").await {
    Ok(summary) => format!("{}/{}", summary.total, summary.total),
    Err(_) => {
      // Fallback to DB
      match ddbot::get_subs_summary_from_db() {
        Ok(summary) => {
          let total: u32 = summary.values().sum();
          format!("{} (离线)", total)
        }
        Err(_) => "无法获取状态".to_string(),
      }
    }
  }
}

pub async fn call_onebot_status_api() -> Result<serde_json::Value, anyhow::Error> {
  match call_admin_api::<OneBotStatusResponse>("/onebot/status").await {
    Ok(response) => {
      Ok(serde_json::json!({
        "online": response.connected,
        "good": response.connected,
        "connected": response.connected
      }))
    },
    Err(e) => {
      eprintln!("OneBot API 调用失败: {}", e);
      Err(e)
    }
  }
}

pub async fn call_subs_summary_api() -> Result<serde_json::Value, anyhow::Error> {
  match call_admin_api::<SubsSummaryResponse>("/subs/summary").await {
    Ok(response) => {
      Ok(serde_json::json!({
        "total": response.total,
        "active": response.total,
        "paused": 0,
        "bySite": response.by_site.unwrap_or_default()
      }))
    },
    Err(_) => {
      // Fallback to DB
      match ddbot::get_subs_summary_from_db() {
        Ok(summary) => {
          let total: u32 = summary.values().sum();
          Ok(serde_json::json!({
            "total": total,
            "active": total,
            "paused": 0,
            "bySite": summary,
            "offline": true
          }))
        }
        Err(e) => {
          eprintln!("订阅统计 API 和数据库回退均失败: {}", e);
          Err(anyhow::anyhow!("无法获取统计数据"))
        }
      }
    }
  }
}

pub async fn admin_token() -> Option<String> {
  ADMIN_TOKEN.lock().await.clone()
}
