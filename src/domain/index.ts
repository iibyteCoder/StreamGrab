/**
 * 领域层
 *
 * 前端业务类型的唯一来源（与后端 JSON 契约一一对应）。
 * 组件/服务/Store 一律从 `@/domain` 导入类型。
 */

export * from "./config";
export * from "./stream";
export * from "./task";
export * from "./url";
