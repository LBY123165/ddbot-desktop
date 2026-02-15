#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ddbot;
mod process;

use std::fs;
use std::path::PathBuf;

#[tauri::command]
fn get_platform_triple() -> String {
  format!(
    "{}-{}",
    std::env::consts::OS,
    std::env::consts::ARCH
  )
}

#[tauri::command]
fn get_workdir() -> String {
  ddbot::workdir().display().to_string()
}

#[tauri::command]
fn get_binary_path() -> String {
  ddbot::binary_path().display().to_string()
}

#[tauri::command]
fn get_managed_ddbot_dir() -> String {
  ddbot::managed_ddbot_dir().display().to_string()
}

#[tauri::command]
fn is_user_approved() -> bool {
  ddbot::approval_marker_path().exists()
}

#[tauri::command]
fn set_user_approved(approved: bool) -> Result<(), String> {
  use std::fs;
  ddbot::ensure_managed_dirs().map_err(|e| e.to_string())?;
  let p = ddbot::approval_marker_path();
  if approved {
    fs::create_dir_all(p.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::write(&p, b"{}\n").map_err(|e| e.to_string())?;
  } else {
    let _ = fs::remove_file(&p);
  }
  Ok(())
}

#[tauri::command]
async fn import_existing_deployment(src_dir: String) -> Result<(), String> {
  let src = std::path::PathBuf::from(src_dir);
  ddbot::copy_deployment_to_managed_dir(&src).map_err(|e| e.to_string())?;

  ddbot::ensure_installed().await.map_err(|e| e.to_string())?;

  let _ = ddbot::enable_admin_in_config_if_approved().map_err(|e| e.to_string())?;
  Ok(())
}

#[tauri::command]
async fn ensure_ddbot_installed() -> Result<(), String> {
  ddbot::ensure_installed().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn read_config_file(filename: String) -> Result<String, String> {
  ddbot::read_config_file(&filename).map_err(|e| e.to_string())
}

#[tauri::command]
async fn write_config_file(filename: String, content: String) -> Result<(), String> {
  ddbot::write_config_file(&filename, &content).map_err(|e| e.to_string())
}

#[tauri::command]
async fn call_onebot_status_api() -> Result<serde_json::Value, String> {
  process::call_onebot_status_api().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn call_subs_summary_api() -> Result<serde_json::Value, String> {
  process::call_subs_summary_api().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn process_start() -> Result<(), String> {
  process::start().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn process_stop() -> Result<(), String> {
  process::stop().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn process_restart() -> Result<(), String> {
  process::restart().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn process_status_text() -> Result<String, String> {
  Ok(process::status_text().await)
}

#[tauri::command]
async fn onebot_status_text() -> Result<String, String> {
  Ok(process::onebot_status_text().await)
}

#[tauri::command]
async fn subs_summary_text() -> Result<String, String> {
  Ok(process::subs_summary_text().await)
}

#[tauri::command]
async fn installed_version_text() -> Result<String, String> {
  Ok(ddbot::installed_version_text().await)
}

#[tauri::command]
async fn read_logs_tail(lines: usize) -> Result<Vec<String>, String> {
  ddbot::read_logs_tail(lines).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_config_backups(filename: String) -> Result<Vec<String>, String> {
  ddbot::list_config_backups(&filename).map_err(|e| e.to_string())
}

#[tauri::command]
async fn restore_config_backup(backup_name: String) -> Result<(), String> {
  ddbot::restore_config_backup(&backup_name).map_err(|e| e.to_string())
}

#[tauri::command]
async fn download_file(url: String, save_path: String, filename: Option<String>) -> Result<String, String> {
  use reqwest::Client;
  
  let client = Client::new();
  let resp = client
    .get(&url)
    .header("User-Agent", "DDBOT-WSa-Desktop")
    .send()
    .await
    .map_err(|e| format!("下载请求失败: {}", e))?
    .error_for_status()
    .map_err(|e| format!("HTTP 错误: {}", e))?;
  
  let bytes = resp.bytes().await.map_err(|e| format!("读取响应失败: {}", e))?;
  
  let save_path = PathBuf::from(save_path);
  fs::create_dir_all(&save_path).map_err(|e| format!("创建目录失败: {}", e))?;
  
  let filename = filename.unwrap_or_else(|| {
    url.split('/')
      .last()
      .unwrap_or("downloaded_file")
      .to_string()
  });
  
  let file_path = save_path.join(filename);
  fs::write(&file_path, bytes).map_err(|e| format!("写入文件失败: {}", e))?;
  
  Ok(file_path.to_string_lossy().to_string())
}

fn main() {
  env_logger::init();
  
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_shell::init())
    .setup(|app| {
      let _ = app.handle();
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      get_platform_triple,
      get_workdir,
      get_binary_path,
      get_managed_ddbot_dir,
      is_user_approved,
      set_user_approved,
      import_existing_deployment,
      ensure_ddbot_installed,
      read_config_file,
      write_config_file,
      call_onebot_status_api,
      call_subs_summary_api,
      process_start,
      process_stop,
      process_restart,
      process_status_text,
      onebot_status_text,
      subs_summary_text,
      installed_version_text,
      read_logs_tail,
      list_config_backups,
      restore_config_backup,
      download_file,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
