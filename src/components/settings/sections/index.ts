/**
 * 设置区域组件导出
 *
 * 保留的 section 组件（已 rewire 到 domain 类型）：
 * - NetworkSettings: 接收 NetworkConfig，emit DeepPartial<Nm3u8dlConfig>
 * - DecryptionSettings: 接收 DecryptionConfig，emit DeepPartial<Nm3u8dlConfig>
 * - LiveSettings: 接收 LiveFields，emit DeepPartial<Nm3u8dlConfig>
 * - MuxSettings: 接收 MuxFields，emit DeepPartial<FfmpegConfig>
 */

export { default as MuxSettings } from "./MuxSettings.vue";
export { default as NetworkSettings } from "./NetworkSettings.vue";
export { default as DecryptionSettings } from "./DecryptionSettings.vue";
export { default as LiveSettings } from "./LiveSettings.vue";

// 类型导出（供标签页组件使用）
export type { MuxFields } from "./MuxSettings.vue";
export type { LiveFields } from "./LiveSettings.vue";
