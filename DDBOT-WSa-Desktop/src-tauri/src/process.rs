use anyhow::{anyhow, Result};
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use crate::ddbot;

static ADMIN_TOKEN: Mutex<Option<String>> = Mutex::const_new(None);

static CHILD: Mutex<Option<Child>> = Mutex::const_new(None);

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
  // MVP: later query onebot via local file/port or parse logs.
  // For now we show placeholder.
  "待实现".to_string()
}

pub async fn subs_summary_text() -> String {
  // MVP: later call into bot via admin api or parse .lsp.db.
  "待实现".to_string()
}

pub async fn admin_token() -> Option<String> {
  ADMIN_TOKEN.lock().await.clone()
}
