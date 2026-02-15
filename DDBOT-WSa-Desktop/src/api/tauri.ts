// 提供默认的模拟实现
const defaultInvoke: any = async (cmd: string, args?: any) => {
  console.log(`Mock invoke: ${cmd}`, args);
  
  // 根据命令返回模拟数据
  switch (cmd) {
    case 'read_config_file':
      return `bot:\n  onJoinGroup:\n    rename: "【bot】"\n  sendFailureReminder:\n    enable: false\n    times: 3\n  offlineQueue:\n    enable: false\n    expire: 30m\nbilibili:\n  SESSDATA: \n  bili_jct: \n  qrlogin: true\n  interval: 25s\n  imageMergeMode: "auto"\n  hiddenSub: false\n  unsub: false\n  minFollowerCap: 0\n  disableSub: false\n  onlyOnlineNotify: false\n  autoParsePosts: false\n  secAnalysis: false\nlogLevel: info`;
    case 'write_config_file':
      console.log('Mock write config file:', args?.filename);
      console.log('Config content:', args?.content);
      return null;
    case 'read_logs_tail':
      return [
        '[INFO] DDBOT-WSa started',
        '[INFO] Loading configuration...',
        '[INFO] Starting WebSocket server...',
        '[INFO] Ready to accept connections',
        '[INFO] Checking for updates...'
      ];
    case 'list_config_backups':
      return [];
    case 'check_firewall_rule':
      return true;
    case 'get_current_version':
      return '1.0.0';
    case 'get_admin_password':
      return 'admin123';
    default:
      console.log(`Mock invoke: ${cmd} (unhandled)`, args);
      return null;
  }
};

const defaultListen: any = async (event: string, callback: any) => {
  console.log(`Mock listen: ${event}`);
  return {
    remove: () => console.log(`Mock remove listener: ${event}`)
  };
};

const defaultAppDataDir: any = async () => {
  return 'C:\\Users\\User\\AppData\\Roaming\\DDBOT-WSa-Desktop';
};

const defaultJoin: any = async (...paths: string[]) => {
  return paths.join('\\');
};

// 导出的变量，始终保持为有效的函数
let invoke: any = defaultInvoke;
let listen: any = defaultListen;
let appDataDir: any = defaultAppDataDir;
let join: any = defaultJoin;

// 尝试替换为实际的Tauri API实现
async function initTauriAPI() {
  try {
    // 检查是否在浏览器环境中
    if (typeof window !== 'undefined' && window.location && window.location.protocol.startsWith('http')) {
      console.log('Running in browser environment, using mock implementations');
      return;
    }
    
    // 动态导入Tauri API
    const core = await import('@tauri-apps/api/core');
    const event = await import('@tauri-apps/api/event');
    const path = await import('@tauri-apps/api/path');
    
    console.log('Tauri API modules loaded:', {
      core: !!core,
      event: !!event,
      path: !!path,
      coreInvoke: core?.invoke
    });
    
    // 检查导入的API是否有效
    if (core && typeof core.invoke === 'function') {
      console.log('Tauri invoke API available, using real implementation');
      invoke = core.invoke;
    } else {
      console.warn('Tauri invoke API not available, keeping mock implementation');
      invoke = defaultInvoke;
    }
    
    if (event && typeof event.listen === 'function') {
      listen = event.listen;
    } else {
      listen = defaultListen;
    }
    
    if (path && typeof path.appDataDir === 'function') {
      appDataDir = path.appDataDir;
    } else {
      appDataDir = defaultAppDataDir;
    }
    
    if (path && typeof path.join === 'function') {
      join = path.join;
    } else {
      join = defaultJoin;
    }
  } catch (error) {
    console.warn('Tauri API not available, using mock implementations');
    console.error('Error loading Tauri API:', error);
    // 确保变量恢复为默认的mock实现
    invoke = defaultInvoke;
    listen = defaultListen;
    appDataDir = defaultAppDataDir;
    join = defaultJoin;
  }
}

// 立即执行初始化
initTauriAPI();

// 模拟插件功能
const relaunch = async (): Promise<void> => {
  // 实际应用中，这会重启应用
  console.log('Relaunching application...');
};

const check = async (): Promise<any> => {
  // 实际应用中，这会检查更新
  console.log('Checking for updates...');
  return null;
};

let pendingUpdateInstance: any = null;

// 类型定义
interface ProcessInfo {
  pid: number;
  running: boolean;
}

interface DDBOTCoreStatus {
  running: boolean;
  pid?: number;
  version?: string;
}

interface MergedSettings {
  [key: string]: any;
}

interface UpdateCheck {
  hasUpdate: boolean;
  currentVersion: string;
  latestVersion: string;
  releaseDate: string;
  releaseNotes: string;
  assets: any[];
}

interface DownloadProgress {
  downloaded: number;
  total: number;
  percentage: number;
  speed: number;
}

// 使用函数工厂模式创建TauriAPI，确保每次调用都使用最新的invoke变量
export const TauriAPI = {
  // --- File management ---
  get files() {
    return {
      open: (path: string): Promise<boolean> => invoke('open_file', { path }),
      folder: (path: string): Promise<boolean> => invoke('open_folder', { path }),
      urlInBrowser: (url: string): Promise<boolean> => invoke('open_url_in_browser', { url }),
      openDDBotDataDir: (): Promise<boolean> => invoke('open_ddbot_data_dir'),
      openLogsDirectory: (): Promise<boolean> => invoke('open_logs_directory'),
      openConfigFile: (): Promise<boolean> => invoke('open_config_file'),
      openSettingsFile: (): Promise<boolean> => invoke('open_settings_file'),
    };
  },

  // --- DDBOT management ---
  get ddbot() {
    return {
      getPlatformTriple: (): Promise<string> => invoke('get_platform_triple'),
      getWorkdir: (): Promise<string> => invoke('get_workdir'),
      getBinaryPath: (): Promise<string> => invoke('get_binary_path'),
      getManagedDdbotDir: (): Promise<string> => invoke('get_managed_ddbot_dir'),
      isUserApproved: (): Promise<boolean> => invoke('is_user_approved'),
      setUserApproved: (approved: boolean): Promise<void> => invoke('set_user_approved', { approved }),
      importExistingDeployment: (srcDir: string): Promise<void> => invoke('import_existing_deployment', { src_dir: srcDir }),
      ensureDdbotInstalled: (): Promise<void> => invoke('ensure_ddbot_installed'),
      readConfigFile: (filename: string): Promise<string> => invoke('read_config_file', { filename }),
      writeConfigFile: (filename: string, content: string): Promise<void> => invoke('write_config_file', { filename, content }),
      readLogsTail: (lines: number): Promise<string[]> => invoke('read_logs_tail', { lines }),
      listConfigBackups: (filename: string): Promise<string[]> => invoke('list_config_backups', { filename }),
      restoreConfigBackup: (backupName: string): Promise<void> => invoke('restore_config_backup', { backup_name: backupName }),
      installedVersionText: (): Promise<string> => invoke('installed_version_text'),
    };
  },

  // --- Process management ---
  get process() {
    return {
      start: (): Promise<void> => invoke('process_start'),
      stop: (): Promise<void> => invoke('process_stop'),
      restart: (): Promise<void> => invoke('process_restart'),
      statusText: (): Promise<string> => invoke('process_status_text'),
      onebotStatusText: (): Promise<string> => invoke('onebot_status_text'),
      subsSummaryText: (): Promise<string> => invoke('subs_summary_text'),
      callOnebotStatusApi: (): Promise<any> => invoke('call_onebot_status_api'),
      callSubsSummaryApi: (): Promise<any> => invoke('call_subs_summary_api'),
    };
  },

  // --- Download management ---
  get download() {
    return {
      downloadFile: async (url: string, savePath: string, filename?: string): Promise<string> => {
        return invoke('download_file', { url, save_path: savePath, filename });
      },
    };
  },

  // --- Settings management ---
  get settings() {
    return {
      load: (): Promise<MergedSettings | null> => invoke('load_settings'),
      save: (s: MergedSettings): Promise<boolean> => invoke('save_settings', { settings: s }),
      saveAndRestart: (s: MergedSettings): Promise<boolean> => invoke('save_settings_and_restart', { settings: s }),
      reset: (): Promise<MergedSettings | null> => invoke('reset_settings'),
    };
  },

  // --- Logs management ---
  get logs() {
    return {
      get: (src?: 'ddbot' | 'app' | 'all'): Promise<string[]> =>
        invoke('get_logs', { source: src }),
      clear: (src?: 'ddbot' | 'app' | 'all'): Promise<boolean> =>
        invoke('clear_logs', { source: src }),
      adminPassword: (): Promise<string> => invoke('get_admin_password'),
      resetAdminPassword: (): Promise<string> => invoke('reset_admin_password'),
      setAdminPassword: (password: string): Promise<string> => invoke('set_admin_password', { password }),
    };
  },

  // --- Binary management ---
  get bin() {
    return {
      version: (name: 'ddbot'): Promise<string> => invoke('get_binary_version', { binaryName: name }),
      availableVersions: (tool: string, force: boolean): Promise<string[]> =>
        invoke('get_available_versions', { tool, force }),
      updateVersion: (tool: string, version: string): Promise<string> => invoke('update_tool_version', { tool, version }),
    };
  },

  // --- Firewall management ---
  get firewall() {
    return {
      check: (): Promise<boolean> => invoke('check_firewall_rule'),
      add: (): Promise<boolean> => invoke('add_firewall_rule'),
      remove: (): Promise<boolean> => invoke('remove_firewall_rule'),
    };
  },

  // --- Tray management ---
  get tray() {
    return {
      update: (r: boolean): Promise<void> => invoke('update_tray_menu', { serviceRunning: r }),
    };
  },

  // --- Update management ---
  get updater() {
    return {
      check: async (): Promise<UpdateCheck> => {
        const update = await check();
        const currentVersion = await invoke<string>('get_current_version');

        pendingUpdateInstance = update;

        if (update) {
          return {
            hasUpdate: true,
            currentVersion,
            latestVersion: '1.0.0',
            releaseDate: new Date().toISOString(),
            releaseNotes: 'New features and bug fixes',
            assets: [],
          };
        }

        return {
          hasUpdate: false,
          currentVersion,
          latestVersion: currentVersion,
          releaseDate: '',
          releaseNotes: '',
          assets: [],
        };
      },

      hasPendingUpdate: (): boolean => {
        return pendingUpdateInstance !== null;
      },

      downloadAndInstall: async (onProgress?: (progress: DownloadProgress) => void): Promise<void> => {
        if (!pendingUpdateInstance) {
          throw new Error('No pending update available. Call check() first.');
        }

        // 模拟下载进度
        let downloaded = 0;
        const contentLength = 1000000;

        if (onProgress) {
          onProgress({
            downloaded: 0,
            total: contentLength,
            percentage: 0,
            speed: 0,
          });
        }

        // 模拟下载过程
        for (let i = 0; i < 10; i++) {
          await new Promise(resolve => setTimeout(resolve, 500));
          downloaded += contentLength / 10;
          if (onProgress) {
            onProgress({
              downloaded,
              total: contentLength,
              percentage: (downloaded / contentLength) * 100,
              speed: contentLength / 10 / 0.5,
            });
          }
        }

        if (onProgress) {
          onProgress({
            downloaded: contentLength,
            total: contentLength,
            percentage: 100,
            speed: 0,
          });
        }

        pendingUpdateInstance = null;
      },

      clearPendingUpdate: (): void => {
        pendingUpdateInstance = null;
      },

      relaunch: async (): Promise<void> => {
        await relaunch();
      },

      currentVersion: (): Promise<string> => invoke('get_current_version'),
      setAutoCheck: (e: boolean): Promise<void> => invoke('set_auto_check_enabled', { enabled: e }),
      isAutoCheckEnabled: (): Promise<boolean> => invoke('is_auto_check_enabled'),
      onBackgroundUpdate: (cb: (u: UpdateCheck) => void) =>
        listen('background-update-available', e => cb(e.payload as UpdateCheck)),
    };
  },

  // --- Utility functions ---
  get util() {
    return {
      defaultDataDir: async (): Promise<string> => {
        const base = await appDataDir();
        return join(base, 'DDBOT-WSa-Desktop');
      },
      defaultBinaryDir: async (): Promise<string> => {
        const base = await appDataDir();
        return join(base, 'DDBOT-WSa-Desktop', 'binary');
      },
      defaultDataDdbotDir: async (): Promise<string> => {
        const base = await appDataDir();
        return join(base, 'DDBOT-WSa-Desktop', 'data', 'ddbot');
      },
      selectDirectory: (title: string): Promise<string | null> => invoke('select_directory', { title }),
    };
  },
};
