import { createPinia } from "pinia";

export const pinia = createPinia();

// 导出全部 stores
export { useTaskStore } from "./taskStore";
export type { TaskLogEntry } from "./taskStore";
export { useSettingsStore } from "./settingsStore";
export { usePresetStore } from "./presetStore";
export { useHistoryStore } from "./historyStore";
