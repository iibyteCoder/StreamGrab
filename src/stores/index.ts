import { createPinia } from 'pinia';

export const pinia = createPinia();

// 导出所有 stores
export { useTaskStore } from './taskStore';
export { useSettingsStore } from './settingsStore';
export { useUiStore } from './uiStore';
export { useHistoryStore } from './historyStore';
export { useTemplateStore } from './templateStore';
