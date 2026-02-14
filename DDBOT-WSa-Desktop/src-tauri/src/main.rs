#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ddbot;
mod process;

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
      process_start,
      process_stop,
      process_restart,
      process_status_text,
      onebot_status_text,
      subs_summary_text,
      installed_version_text,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
