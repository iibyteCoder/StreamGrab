/**
 * 系统服务
 *
 * 对话框、文件系统操作、应用更新下载与安装
 */

import { invokeTauri, subscribeToEvent, type UnlistenFn } from "./tauri";

/** 文件过滤器 */
export interface FileFilter {
  name: string;
  extensions: string[];
}

/** 文件信息 */
export interface FileInfo {
  path: string;
  fileName: string;
  extension: string;
  size: number;
  /** Unix 毫秒时间戳 */
  modified: number | null;
  exists: boolean;
}

/** 应用下载进度事件 */
export interface AppDownloadProgress {
  status: string;
  downloaded: number;
  total: number;
  percent: number;
}

class SystemService {
  // ===== 对话框 =====

  /** 选择目录（取消返回 null） */
  selectDirectory(): Promise<string | null> {
    return invokeTauri<string | null>("select_directory");
  }

  /** 选择文件（取消返回 null） */
  selectFile(filters?: FileFilter[]): Promise<string | null> {
    return invokeTauri<string | null>("select_file", {
      filters: filters ?? null,
    });
  }

  // ===== 文件系统 =====

  /** 在文件管理器中打开路径 */
  openInExplorer(path: string): Promise<void> {
    return invokeTauri("open_in_explorer", { path });
  }

  /** 打开文件所在目录并选中文件 */
  openFileInExplorer(filePath: string): Promise<void> {
    return invokeTauri("open_file_in_explorer", { filePath });
  }

  fileExists(path: string): Promise<boolean> {
    return invokeTauri<boolean>("file_exists", { path });
  }

  /** 删除文件或文件夹（文件夹递归删除） */
  deleteFileOrFolder(path: string): Promise<void> {
    return invokeTauri("delete_file_or_folder", { path });
  }

  getDbPath(): Promise<string> {
    return invokeTauri<string>("get_db_path");
  }

  // ===== 应用更新 =====

  /** 下载应用更新安装包，返回保存路径 */
  downloadAppUpdate(downloadUrl: string, savePath: string): Promise<string> {
    return invokeTauri<string>("download_app_update", {
      downloadUrl,
      savePath,
    });
  }

  runInstaller(installerPath: string): Promise<void> {
    return invokeTauri("run_installer", { installerPath });
  }

  /** 订阅应用更新下载进度 */
  subscribeToUpdateProgress(
    handler: (progress: AppDownloadProgress) => void,
  ): Promise<UnlistenFn> {
    return subscribeToEvent<AppDownloadProgress>(
      "app:update:progress",
      handler,
    );
  }
}

export const systemService = new SystemService();
