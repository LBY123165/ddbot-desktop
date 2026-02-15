import { invoke } from '@tauri-apps/api/core';
import { appDataDir, join } from '@tauri-apps/api/path';

export class TauriAPI {
  // --- File management ---
  static files = {
    open: (path: string): Promise<boolean> => invoke('open_file', { path }),
    folder: (path: string): Promise<boolean> => invoke('open_folder', { path }),
    urlInBrowser: (url: string): Promise<boolean> => invoke('open_url_in_browser', { url }),
  };

  // --- DDBOT management ---
  static ddbot = {
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

  // --- Process management ---
  static process = {
    start: (): Promise<void> => invoke('process_start'),
    stop: (): Promise<void> => invoke('process_stop'),
    restart: (): Promise<void> => invoke('process_restart'),
    statusText: (): Promise<string> => invoke('process_status_text'),
    onebotStatusText: (): Promise<string> => invoke('onebot_status_text'),
    subsSummaryText: (): Promise<string> => invoke('subs_summary_text'),
    callOnebotStatusApi: (): Promise<any> => invoke('call_onebot_status_api'),
    callSubsSummaryApi: (): Promise<any> => invoke('call_subs_summary_api'),
  };

  // --- Download management ---
  static download = {
    downloadFile: async (url: string, savePath: string, filename?: string): Promise<string> => {
      return invoke('download_file', { url, save_path: savePath, filename });
    },
  };

  // --- Utility functions ---
  static util = {
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
  };
}
