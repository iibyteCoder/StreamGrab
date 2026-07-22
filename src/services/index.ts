/**
 * 服务层导出
 *
 * 每个 service 与后端一个命令域对应：
 * task / download / settings / preset / tools / system
 */

export { invokeTauri, subscribeToEvent, type UnlistenFn } from "./tauri";

export { taskService } from "./taskService";

export {
  downloadService,
  type DownloadEvent,
  type DownloadEventType,
  type LogEventData,
  type StatusEventData,
} from "./downloadService";

export { settingsService, type DeepPartial } from "./settingsService";

export { presetService } from "./presetService";

export {
  toolsService,
  type ToolDownloadProgress,
  type ToolInfo,
  type ToolReleaseInfo,
} from "./toolsService";

export {
  systemService,
  type AppDownloadProgress,
  type FileInfo,
  type FileFilter,
} from "./systemService";

export { clipboardService } from "./clipboardService";

export {
  updateService,
  type ReleaseInfo,
  type ReleaseAsset,
  type UpdateError,
} from "./updateService";
