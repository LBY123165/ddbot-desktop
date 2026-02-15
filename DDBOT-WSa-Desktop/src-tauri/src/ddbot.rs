use anyhow::{anyhow, Context, Result};
use rusqlite::{Connection, OpenFlags};
use flate2::read::GzDecoder;
use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use chrono::{DateTime, Local};
use std::collections::HashMap;
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use tar::Archive;
use walkdir::WalkDir;
use zip::ZipArchive;

const REPO: &str = "cnxysoft/DDBOT-WSa";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledInfo {
  pub tag_name: String,
  pub asset_name: String,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
  tag_name: String,
  assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
  name: String,
  browser_download_url: String,
}

pub fn workdir() -> PathBuf {
  // Put everything under app data dir
  let base = dirs::data_dir().unwrap_or_else(|| std::env::current_dir().unwrap());
  base.join("DDBOT-WSa-Desktop")
}

pub fn data_dir() -> PathBuf {
  workdir().join("data")
}

pub fn managed_ddbot_dir() -> PathBuf {
  // Everything the bot reads/writes lives here (application.yaml, .lsp.db, logs, templates...)
  data_dir().join("ddbot")
}

pub fn config_path() -> PathBuf {
  managed_ddbot_dir().join("application.yaml")
}

pub fn approval_marker_path() -> PathBuf {
  // Marker file indicates the user approved desktop to modify local config (e.g., enabling admin API)
  data_dir().join("approved.json")
}

pub fn binary_dir() -> PathBuf {
  workdir().join("binary")
}

pub fn binary_path() -> PathBuf {
  let exe = if cfg!(windows) { "DDBOT.exe" } else { "DDBOT" };
  binary_dir().join(exe)
}

pub fn lsp_db_path() -> PathBuf {
  managed_ddbot_dir().join(".lsp.db")
}

pub fn lsp_db_lock_path() -> PathBuf {
  managed_ddbot_dir().join(".lsp.db.lock")
}

pub fn ext_db_lock_path() -> PathBuf {
  managed_ddbot_dir().join(".lsp_ext.db.lock")
}

fn installed_info_path() -> PathBuf {
  workdir().join("installed.json")
}

pub async fn installed_version_text() -> String {
  match read_installed_info() {
    Ok(Some(info)) => info.tag_name,
    Ok(None) => "未安装".to_string(),
    Err(_) => "未知".to_string(),
  }
}

fn read_installed_info() -> Result<Option<InstalledInfo>> {
  let p = installed_info_path();
  if !p.exists() {
    return Ok(None);
  }
  let data = fs::read(p)?;
  let info: InstalledInfo = serde_json::from_slice(&data)?;
  Ok(Some(info))
}

fn write_installed_info(info: &InstalledInfo) -> Result<()> {
  fs::create_dir_all(workdir())?;
  let data = serde_json::to_vec_pretty(info)?;
  fs::write(installed_info_path(), data)?;
  Ok(())
}

pub fn ensure_managed_dirs() -> Result<()> {
  fs::create_dir_all(managed_ddbot_dir())?;
  Ok(())
}

pub fn read_config_file(filename: &str) -> Result<String> {
  ensure_managed_dirs()?;
  let path = managed_ddbot_dir().join(filename);
  if path.exists() {
    fs::read_to_string(path).map_err(|e| anyhow!("读取配置文件失败: {}", e))
  } else {
    // 如果文件不存在，返回默认模板
    Ok(get_default_config_template(filename))
  }
}

pub fn write_config_file(filename: &str, content: &str) -> Result<()> {
  ensure_managed_dirs()?;
  let path = managed_ddbot_dir().join(filename);
  
  // 创建带时间戳的备份
  if path.exists() {
    let backups_dir = managed_ddbot_dir().join("backups");
    if !backups_dir.exists() {
      fs::create_dir_all(&backups_dir)?;
    }
    let now: DateTime<Local> = Local::now();
    let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
    let backup_name = format!("{}.{}.bak", filename, timestamp);
    let backup_path = backups_dir.join(backup_name);
    fs::copy(&path, &backup_path)?;
  }
  
  fs::write(path, content).map_err(|e| anyhow!("写入配置文件失败: {}", e))
}

pub fn list_config_backups(filename: &str) -> Result<Vec<String>> {
  let backups_dir = managed_ddbot_dir().join("backups");
  if !backups_dir.exists() {
    return Ok(vec![]);
  }

  let mut backups = Vec::new();
  for entry in fs::read_dir(backups_dir)? {
    let entry = entry?;
    let path = entry.path();
    if path.is_file() {
      if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if name.starts_with(filename) && name.ends_with(".bak") {
          backups.push(name.to_string());
        }
      }
    }
  }
  backups.sort_by(|a, b| b.cmp(a)); // 按时间降序
  Ok(backups)
}

pub fn restore_config_backup(backup_name: &str) -> Result<()> {
  let backups_dir = managed_ddbot_dir().join("backups");
  let backup_path = backups_dir.join(backup_name);
  if !backup_path.exists() {
    return Err(anyhow!("备份文件不存在"));
  }

  // 假设备份文件名格式为 filename.timestamp.bak
  let parts: Vec<&str> = backup_name.split('.').collect();
  if parts.len() < 3 {
    return Err(anyhow!("无效的备份文件名格式"));
  }
  let original_filename = parts[0];
  let target_path = managed_ddbot_dir().join(original_filename);

  fs::copy(backup_path, target_path)?;
  Ok(())
}

pub async fn read_logs_tail(lines: usize) -> Result<Vec<String>> {
  let log_dir = managed_ddbot_dir().join("logs");
  let log_path = log_dir.join("latest.log");
  
  if !log_path.exists() {
    return Ok(vec![]);
  }
  
  let content = fs::read_to_string(&log_path).map_err(|e| anyhow!("读取日志文件失败: {}", e))?;
  let all_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
  let start = if all_lines.len() > lines {
    all_lines.len() - lines
  } else {
    0
  };
  
  Ok(all_lines[start..].to_vec())
}

pub fn get_subs_summary_from_db() -> Result<HashMap<String, u32>> {
    let db_path = managed_ddbot_dir().join(".lsp.db");
    if !db_path.exists() {
        return Ok(HashMap::new());
    }

    // 只读方式打开，不检查锁，因为我们只是读取统计信息
    let conn = Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ).map_err(|e| anyhow!("无法打开数据库: {}", e))?;

    let mut stmt = conn.prepare("SELECT site, COUNT(*) as count FROM subs GROUP BY site")
        .map_err(|e| anyhow!("查询失败: {}", e))?;

    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
    }).map_err(|e| anyhow!("读取结果失败: {}", e))?;

    let mut summary = HashMap::new();
    for row in rows {
        if let Ok((site, count)) = row {
            summary.insert(site, count);
        }
    }

    Ok(summary)
}

fn get_default_config_template(filename: &str) -> String {
  match filename {
    "application.yaml" => {
      r#"# DDBOT-WSa 配置文件
# 请根据需要修改配置

# 基础配置
log_level: info

# Admin API 配置
admin:
  enable: false
  addr: "127.0.0.1:15631"
  token: ""

# 其他配置...
"#.to_string()
    },
    "template.yaml" => {
      r#"# 模板配置文件
# 在这里定义您的模板配置

templates:
  - name: "默认模板"
    content: |
      这是一个示例模板
"#.to_string()
    },
    _ => "".to_string()
  }
}

pub fn ensure_default_config_exists() -> Result<()> {
  ensure_managed_dirs()?;
  let p = config_path();
  if p.exists() {
    return Ok(());
  }
  // Create an empty placeholder; the actual content is expected to come from imported deployment
  // or user-generated config. Keeping it minimal to avoid drifting from upstream defaults.
  fs::write(p, b"")?;
  Ok(())
}

fn looks_like_ddbot_executable(path: &Path) -> bool {
  if !path.is_file() {
    return false;
  }
  let name = path
    .file_name()
    .and_then(|s| s.to_str())
    .unwrap_or("")
    .to_lowercase();
  if cfg!(windows) {
    name == "ddbot.exe" || name.ends_with("/ddbot.exe")
  } else {
    name == "ddbot" || name.ends_with("/ddbot")
  }
}

fn find_ddbot_executable_in_dir(dir: &Path) -> Option<PathBuf> {
  let rd = fs::read_dir(dir).ok()?;
  for e in rd.flatten() {
    let p = e.path();
    if looks_like_ddbot_executable(&p) {
      return Some(p);
    }
  }
  None
}

pub fn validate_existing_deployment(src_dir: &Path) -> Result<()> {
  if !src_dir.exists() {
    return Err(anyhow!("source dir not found: {}", src_dir.display()));
  }
  if !src_dir.is_dir() {
    return Err(anyhow!("source is not a directory: {}", src_dir.display()));
  }

  let exe = find_ddbot_executable_in_dir(src_dir);
  if exe.is_none() {
    return Err(anyhow!("ddbot executable not found in: {}", src_dir.display()));
  }
  let yaml = src_dir.join("application.yaml");
  if !yaml.exists() {
    return Err(anyhow!("application.yaml not found in: {}", src_dir.display()));
  }
  let lsp = src_dir.join(".lsp.db");
  if !lsp.exists() {
    return Err(anyhow!(".lsp.db not found in: {}", src_dir.display()));
  }
  Ok(())
}

pub fn copy_deployment_to_managed_dir(src_dir: &Path) -> Result<()> {
  ensure_managed_dirs()?;
  let src = src_dir;
  validate_existing_deployment(src)?;

  let dst = managed_ddbot_dir();
  for entry in WalkDir::new(src).follow_links(false) {
    let entry = entry?;
    let path = entry.path();
    let rel = path.strip_prefix(src).unwrap();
    if rel.as_os_str().is_empty() {
      continue;
    }
    let out = dst.join(rel);

    if entry.file_type().is_dir() {
      fs::create_dir_all(&out)?;
      continue;
    }
    if entry.file_type().is_file() {
      // Never copy the original executable into managed dir.
      // Desktop will download its own managed binary and run with cwd=managed dir.
      if looks_like_ddbot_executable(path) {
        continue;
      }
      if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
      }
      fs::copy(path, &out)?;
    }
  }
  Ok(())
}

fn random_token() -> String {
  rand::thread_rng()
    .sample_iter(&Alphanumeric)
    .take(48)
    .map(char::from)
    .collect()
}

fn yaml_ensure_mapping(root: &mut Mapping, key: &str) {
  let k = Value::String(key.to_string());
  
  // Ensure the key exists and is a mapping
  if let Some(v) = root.get(&k) {
    if !matches!(v, Value::Mapping(_)) {
      root.remove(&k);
      root.insert(k, Value::Mapping(Mapping::new()));
    }
  } else {
    root.insert(k, Value::Mapping(Mapping::new()));
  }
}

pub fn enable_admin_in_config_if_approved() -> Result<Option<String>> {
  // Only touch config if user has approved marker.
  if !approval_marker_path().exists() {
    return Ok(None);
  }
  ensure_managed_dirs()?;
  let p = config_path();
  if !p.exists() {
    return Ok(None);
  }

  let raw = fs::read_to_string(&p).unwrap_or_default();
  let mut doc = if raw.trim().is_empty() {
    Value::Mapping(Mapping::new())
  } else {
    serde_yaml::from_str::<Value>(&raw).unwrap_or(Value::Mapping(Mapping::new()))
  };

  let root = match doc {
    Value::Mapping(ref mut m) => m,
    _ => {
      doc = Value::Mapping(Mapping::new());
      match doc {
        Value::Mapping(ref mut m) => m,
        _ => unreachable!(),
      }
    }
  };

  yaml_ensure_mapping(root, "admin");
  
  let admin_key = Value::String("admin".to_string());
  if let Some(Value::Mapping(admin)) = root.get_mut(&admin_key) {
    admin.insert(Value::String("enable".to_string()), Value::Bool(true));
    admin.insert(
      Value::String("addr".to_string()),
      Value::String("127.0.0.1:15631".to_string()),
    );

    let token_key = Value::String("token".to_string());
    let token = match admin.get(&token_key) {
      Some(Value::String(s)) if !s.trim().is_empty() => s.clone(),
      _ => {
        let t = random_token();
        admin.insert(token_key, Value::String(t.clone()));
        t
      }
    };
    
    let out = serde_yaml::to_string(&doc)?;
    fs::write(&p, out)?;
    Ok(Some(token))
  } else {
    unreachable!()
  }
}

pub fn read_admin_token() -> Result<Option<String>> {
  let p = config_path();
  if !p.exists() {
    return Ok(None);
  }
  let raw = fs::read_to_string(&p).unwrap_or_default();
  if raw.trim().is_empty() {
    return Ok(None);
  }
  let doc = serde_yaml::from_str::<Value>(&raw).unwrap_or(Value::Mapping(Mapping::new()));
  let root = match doc {
    Value::Mapping(m) => m,
    _ => return Ok(None),
  };
  let admin = root.get(&Value::String("admin".to_string()));
  let admin = match admin {
    Some(Value::Mapping(m)) => m,
    _ => return Ok(None),
  };
  match admin.get(&Value::String("token".to_string())) {
    Some(Value::String(s)) if !s.trim().is_empty() => Ok(Some(s.clone())),
    _ => Ok(None),
  }
}

fn target_keywords() -> (String, Vec<String>) {
  // Map rust consts into repo asset naming.
  // Release examples:
  // DDBOT-WSa-fix_A041-darwin-amd64.tar.gz
  // DDBOT-WSa-fix_A041-linux-arm.tar.gz
  // DDBOT-WSa-fix_A041-linux-arm64.tar.gz
  // DDBOT-WSa-fix_A041-windows-386.zip
  // DDBOT-WSa-fix_A041-windows-arm.zip
  // DDBOT-WSa-fix_A041-windows-arm64.zip
  let os = std::env::consts::OS;
  let arch = std::env::consts::ARCH;

  let os_kw = match os {
    "windows" => "windows",
    "macos" => "darwin",
    "linux" => "linux",
    other => other,
  };

  let arch_kws: Vec<String> = match arch {
    "x86_64" => vec!["amd64".to_string()],
    "x86" | "i686" => vec!["386".to_string()],
    "aarch64" => {
      // Prefer arm64, fallback to arm if a release doesn't provide arm64 assets.
      if os_kw == "linux" || os_kw == "windows" {
        vec!["arm64".to_string(), "arm".to_string()]
      } else {
        vec!["arm64".to_string()]
      }
    }
    other => vec![other.to_string()],
  };

  (os_kw.to_string(), arch_kws)
}

pub async fn ensure_installed() -> Result<()> {
  fs::create_dir_all(binary_dir())?;

  let release = fetch_latest_release().await?;
  let (os_kw, arch_kws) = target_keywords();

  let asset = pick_asset(&release.assets, &os_kw, &arch_kws)
    .with_context(|| format!("no asset found for {}-{:?}", os_kw, arch_kws))?;

  // If already installed same tag+asset, skip
  if let Ok(Some(info)) = read_installed_info() {
    if info.tag_name == release.tag_name && info.asset_name == asset.name && binary_path().exists() {
      return Ok(());
    }
  }

  let bytes = download(&asset.browser_download_url).await?;

  install_from_archive(&asset.name, &bytes)?;

  write_installed_info(&InstalledInfo {
    tag_name: release.tag_name,
    asset_name: asset.name.clone(),
  })?;

  Ok(())
}

async fn fetch_latest_release() -> Result<GithubRelease> {
  let url = format!("https://api.github.com/repos/{}/releases/latest", REPO);
  let client = reqwest::Client::new();
  let resp = client
    .get(url)
    .header("User-Agent", "DDBOT-WSa-Desktop")
    .send()
    .await?
    .error_for_status()?;
  let release = resp.json::<GithubRelease>().await?;
  Ok(release)
}

fn pick_asset<'a>(assets: &'a [GithubAsset], os_kw: &str, arch_kws: &[String]) -> Option<&'a GithubAsset> {
  for arch_kw in arch_kws {
    let re = Regex::new(&format!(
      r"(?i)^DDBOT-WSa-.*-{}-{}\\.(zip|tar\\.gz)$",
      regex::escape(os_kw),
      regex::escape(arch_kw)
    ))
    .ok()?;
    if let Some(asset) = assets.iter().find(|a| re.is_match(&a.name)) {
      return Some(asset);
    }
  }
  None
}

async fn download(url: &str) -> Result<Vec<u8>> {
  let client = reqwest::Client::new();
  let resp = client
    .get(url)
    .header("User-Agent", "DDBOT-WSa-Desktop")
    .send()
    .await?
    .error_for_status()?;
  let bytes = resp.bytes().await?;
  Ok(bytes.to_vec())
}

fn install_from_archive(name: &str, bytes: &[u8]) -> Result<()> {
  let out = binary_path();
  if out.exists() {
    let _ = fs::remove_file(&out);
  }

  if name.to_lowercase().ends_with(".zip") {
    install_from_zip(bytes, &out)
  } else if name.to_lowercase().ends_with(".tar.gz") {
    install_from_targz(bytes, &out)
  } else {
    Err(anyhow!("unsupported archive: {}", name))
  }
}

fn install_from_zip(bytes: &[u8], out: &Path) -> Result<()> {
  let mut zip = ZipArchive::new(Cursor::new(bytes))?;
  for i in 0..zip.len() {
    let mut file = zip.by_index(i)?;
    let fname = file.name().to_string();
    if fname.ends_with("/") {
      continue;
    }
    // pick executable: DDBOT.exe or DDBOT
    let lower = fname.to_lowercase();
    if lower.ends_with("ddbot.exe") || lower.ends_with("ddbot") {
      let mut buf = Vec::new();
      file.read_to_end(&mut buf)?;
      fs::create_dir_all(out.parent().unwrap())?;
      fs::write(out, buf)?;
      #[cfg(unix)]
      {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = fs::metadata(out)?.permissions();
        perm.set_mode(0o755);
        fs::set_permissions(out, perm)?;
      }
      return Ok(());
    }
  }
  Err(anyhow!("executable not found in zip"))
}

fn install_from_targz(bytes: &[u8], out: &Path) -> Result<()> {
  let gz = GzDecoder::new(Cursor::new(bytes));
  let mut ar = Archive::new(gz);
  for entry in ar.entries()? {
    let mut entry = entry?;
    let path = entry.path()?.to_string_lossy().to_string();
    let lower = path.to_lowercase();
    if lower.ends_with("ddbot") || lower.ends_with("ddbot.exe") {
      fs::create_dir_all(out.parent().unwrap())?;
      entry.unpack(out)?;
      #[cfg(unix)]
      {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = fs::metadata(out)?.permissions();
        perm.set_mode(0o755);
        fs::set_permissions(out, perm)?;
      }
      return Ok(());
    }
  }
  Err(anyhow!("executable not found in tar.gz"))
}
