/**
 * 任务持久化服务
 * 处理所有任务相关的后端 API 调用
 */

import { invokeTauri } from './tauri';
import type { DownloadTask, TaskStatus, TaskProgressData } from '@/types';

/**
 * 后端任务记录格式（与 Rust 结构体匹配）
 */
interface TaskRecord {
  id: string;
  url: string;
  file_name: string;
  save_dir: string;
  output_path: string | null;
  status: string;
  error: string | null;
  progress_json: string;
  config_json: string | null;
  created_at: string;
  updated_at: string;
  started_at: string | null;
  completed_at: string | null;
  was_interrupted: boolean;
}

/**
 * 任务服务类
 */
class TaskService {
  /**
   * 加载所有任务
   */
  async loadAllTasks(): Promise<TaskRecord[]> {
    return invokeTauri<TaskRecord[]>('load_all_tasks');
  }

  /**
   * 加载可恢复的任务（被中断的下载）
   */
  async loadRecoverableTasks(): Promise<TaskRecord[]> {
    return invokeTauri<TaskRecord[]>('load_recoverable_tasks');
  }

  /**
   * 保存任务（创建或更新）
   */
  async saveTask(task: DownloadTask): Promise<void> {
    const record = this.toTaskRecord(task);
    await invokeTauri('save_task', { task: record });
  }

  /**
   * 批量保存任务
   */
  async saveTasks(tasks: DownloadTask[]): Promise<void> {
    const records = tasks.map(t => this.toTaskRecord(t));
    await invokeTauri('save_tasks', { tasks: records });
  }

  /**
   * 更新任务状态
   */
  async updateTaskStatus(taskId: string, status: TaskStatus, error?: string): Promise<void> {
    await invokeTauri('update_task_status', {
      taskId,
      status,
      error: error || null,
    });
  }

  /**
   * 更新任务进度
   */
  async updateTaskProgress(taskId: string, progress: TaskProgressData): Promise<void> {
    await invokeTauri('update_task_progress', {
      taskId,
      progressJson: JSON.stringify(progress),
    });
  }

  /**
   * 删除任务
   */
  async deleteTask(taskId: string): Promise<void> {
    await invokeTauri('delete_task', { taskId });
  }

  /**
   * 清除已完成的任务
   */
  async clearFinishedTasks(): Promise<number> {
    return invokeTauri<number>('clear_finished_tasks');
  }

  /**
   * 标记活跃任务为已中断
   */
  async markActiveTasksInterrupted(): Promise<number> {
    return invokeTauri<number>('mark_active_tasks_interrupted');
  }

  /**
   * 清除所有任务
   */
  async clearAllTasks(): Promise<void> {
    await invokeTauri('clear_all_tasks');
  }

  /**
   * 将 DownloadTask 转换为 TaskRecord（用于后端）
   */
  toTaskRecord(task: DownloadTask): TaskRecord {
    return {
      id: task.id,
      url: task.url,
      file_name: task.fileName,
      save_dir: task.saveDir,
      output_path: task.outputPath || null,
      status: task.status,
      error: task.error || null,
      progress_json: JSON.stringify(task.progress),
      config_json: task.config ? JSON.stringify(task.config) : null,
      created_at: task.createdAt instanceof Date
        ? task.createdAt.toISOString()
        : String(task.createdAt),
      updated_at: task.updatedAt instanceof Date
        ? task.updatedAt.toISOString()
        : String(task.updatedAt),
      started_at: task.startedAt
        ? (task.startedAt instanceof Date
            ? task.startedAt.toISOString()
            : String(task.startedAt))
        : null,
      completed_at: task.completedAt
        ? (task.completedAt instanceof Date
            ? task.completedAt.toISOString()
            : String(task.completedAt))
        : null,
      was_interrupted: false,
    };
  }

  /**
   * 将 TaskRecord 转换为 DownloadTask（用于前端）
   */
  toDownloadTask(record: TaskRecord): DownloadTask {
    return {
      id: record.id,
      url: record.url,
      fileName: record.file_name,
      saveDir: record.save_dir,
      outputPath: record.output_path || undefined,
      status: record.status as TaskStatus,
      error: record.error || undefined,
      progress: JSON.parse(record.progress_json) as TaskProgressData,
      config: record.config_json ? JSON.parse(record.config_json) : undefined,
      createdAt: new Date(record.created_at),
      updatedAt: new Date(record.updated_at),
      startedAt: record.started_at ? new Date(record.started_at) : undefined,
      completedAt: record.completed_at ? new Date(record.completed_at) : undefined,
    };
  }
}

export const taskService = new TaskService();
export type { TaskRecord };
