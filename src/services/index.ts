/**
 * 服务层导出
 */

export { invokeTauri, subscribeToEvent, type UnlistenFn } from "./tauri";
export {
  downloadService,
  type DownloadEvent,
  type DownloadEventType,
  type ProgressEventData,
  type StatusEventData,
  type LogEventData,
} from "./downloadService";
export { configService } from "./configService";
export { taskService, type TaskRecord } from "./taskService";
