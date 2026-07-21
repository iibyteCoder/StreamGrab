/**
 * 历史记录服务
 *
 * 任务终态快照的查询与清理（独立于任务表：清除任务不删除历史）
 */

import { invokeTauri } from "./tauri";
import type { HistoryRecord } from "@/domain";

class HistoryService {
  /** 加载全部历史（按完成时间倒序） */
  loadHistory(): Promise<HistoryRecord[]> {
    return invokeTauri<HistoryRecord[]>("load_history");
  }

  deleteHistoryRecord(id: number): Promise<void> {
    return invokeTauri("delete_history_record", { id });
  }

  /** 清空历史，返回删除数量 */
  clearHistory(): Promise<number> {
    return invokeTauri<number>("clear_history");
  }
}

export const historyService = new HistoryService();
