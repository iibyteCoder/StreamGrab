import { ref, computed } from "vue";
import { useTaskStore } from "@/stores";
import { useDownloader, useToast } from "@/composables";
import { useRecentDirs } from "./useRecentDirs";
import { downloadService } from "@/services/downloadService";
import { systemService } from "@/services/systemService";
import { parsePastedText } from "@/components/task/parseLinks";
import { resolveLinkToTask } from "@/components/task/resolveLinkToTask";
import { generateId } from "@/utils/id";
import { isStreamingType } from "@/domain/url";
import type { DownloadTask } from "@/domain";
import type { StagedLink, WizardStep } from "@/components/task/addTaskTypes";

export function useAddTaskWizard() {
  const taskStore = useTaskStore();
  const { addAndStartTask } = useDownloader();
  const { dirs, defaultDir, remember } = useRecentDirs();
  const toast = useToast();

  // ===== 状态 =====
  const step = ref<WizardStep>("paste");
  const links = ref<StagedLink[]>([]);
  const index = ref(0);
  const parseDone = ref(0);
  const parseTotal = ref(0);
  const parsingId = ref<string | null>(null);
  const isSubmitting = ref(false);
  const addedCount = ref(0);
  const showDuplicate = ref(false);
  const duplicateTask = ref<DownloadTask | null>(null);
  let duplicatePending: StagedLink | null = null;

  // ===== 派生 =====
  const total = computed(() => links.value.length);
  const current = computed<StagedLink | null>(
    () => links.value[index.value] ?? null,
  );
  const isSingle = computed(() => total.value === 1);
  const isLast = computed(() => index.value >= total.value - 1);
  const showAddAll = computed(() => total.value > 1);

  // ===== 生命周期 =====
  function reset(): void {
    step.value = "paste";
    links.value = [];
    index.value = 0;
    parseDone.value = 0;
    parseTotal.value = 0;
    parsingId.value = null;
    isSubmitting.value = false;
    addedCount.value = 0;
    showDuplicate.value = false;
    duplicateTask.value = null;
    duplicatePending = null;
  }

  // ===== 步骤 1 → 2：粘贴 → 解析 =====
  async function submitPaste(text: string): Promise<void> {
    const { links: parsed, skipped } = parsePastedText(text);
    if (skipped > 0) toast.warning(`${skipped} 个链接无法识别已跳过`);
    if (parsed.length === 0) {
      if (skipped === 0) toast.warning("未识别到有效链接");
      return;
    }
    links.value = parsed.map((p) => ({
      id: generateId(),
      url: p.url,
      detectedType: p.detectedType,
      fileName: p.fileName,
      saveDir: "",
      overrides: {},
      parseFailed: false,
    }));
    index.value = 0;

    const streaming = links.value.filter((l) =>
      isStreamingType(l.detectedType),
    );
    if (streaming.length === 0) {
      step.value = "config";
      return;
    }
    step.value = "parsing";
    parseTotal.value = streaming.length;
    parseDone.value = 0;
    await Promise.all(
      streaming.map(async (link) => {
        try {
          link.streamInfo = await downloadService.parseUrl(link.url);
          link.parseFailed = false;
        } catch {
          link.parseFailed = true;
        } finally {
          parseDone.value++;
        }
      }),
    );
    step.value = "config";
  }

  // ===== 配置卡内：重试解析 / 浏览目录 =====
  async function retryParse(link: StagedLink): Promise<void> {
    parsingId.value = link.id;
    try {
      link.streamInfo = await downloadService.parseUrl(link.url);
      link.parseFailed = false;
    } catch {
      link.parseFailed = true;
    } finally {
      parsingId.value = null;
    }
  }

  async function browseSaveDir(): Promise<void> {
    const dir = await systemService.selectDirectory();
    if (dir && current.value) current.value.saveDir = dir;
  }

  // ===== 提交 =====
  async function commitOne(
    link: StagedLink,
    fallback: string,
  ): Promise<boolean> {
    const resolved = resolveLinkToTask(link, fallback);
    try {
      if (resolved.hasSchedule) {
        await taskStore.addTask({
          url: resolved.url,
          fileName: resolved.fileName,
          saveDir: resolved.saveDir,
          overrides: resolved.overrides,
        });
      } else {
        await addAndStartTask(
          resolved.url,
          resolved.fileName,
          resolved.saveDir,
          resolved.overrides,
        );
      }
      if (resolved.saveDir) remember(resolved.saveDir);
      return true;
    } catch (e) {
      console.error("Failed to add task:", e);
      toast.error(`添加失败：${link.fileName.trim() || link.url}`);
      return false;
    }
  }

  function closeWithSummary(added: number, dupSkipped = 0): void {
    const parts: string[] = [];
    if (added > 0) parts.push(`已添加 ${added} 个任务`);
    if (dupSkipped > 0) parts.push(`跳过 ${dupSkipped} 个重复`);
    if (parts.length) toast.success(parts.join("，"));
    step.value = "done";
  }

  function advance(): void {
    if (isLast.value) closeWithSummary(addedCount.value);
    else index.value++;
  }

  async function commitAndAdvance(link: StagedLink): Promise<void> {
    isSubmitting.value = true;
    try {
      if (await commitOne(link, defaultDir.value)) addedCount.value++;
    } finally {
      isSubmitting.value = false;
    }
    advance();
  }

  /** 逐条"添加/完成"：命中重复 → 弹确认；否则提交并推进 */
  async function addCurrent(): Promise<void> {
    const link = current.value;
    if (!link || isSubmitting.value) return;
    const existing = taskStore.checkUrlExists(link.url);
    if (existing) {
      duplicateTask.value = existing;
      duplicatePending = link;
      showDuplicate.value = true;
      return;
    }
    await commitAndAdvance(link);
  }

  /** 跳过当前条 */
  function skip(): void {
    if (isSubmitting.value) return;
    advance();
  }

  /** 批量：剩余链接按当前卡目录（回退 defaultDir）默认入库，重复静默跳过 */
  async function addAll(): Promise<void> {
    if (isSubmitting.value) return;
    isSubmitting.value = true;
    const batchDir = current.value?.saveDir.trim() || defaultDir.value;
    let added = 0;
    let dupSkipped = 0;
    try {
      for (let i = index.value; i < links.value.length; i++) {
        const link = links.value[i]!;
        if (taskStore.checkUrlExists(link.url)) {
          dupSkipped++;
          continue;
        }
        if (await commitOne(link, batchDir)) added++;
      }
    } finally {
      isSubmitting.value = false;
    }
    closeWithSummary(addedCount.value + added, dupSkipped);
  }

  // ===== 重复确认 =====
  async function confirmDuplicate(): Promise<void> {
    showDuplicate.value = false;
    const link = duplicatePending;
    duplicateTask.value = null;
    duplicatePending = null;
    if (link) await commitAndAdvance(link);
  }

  function cancelDuplicate(): void {
    showDuplicate.value = false;
    duplicateTask.value = null;
    duplicatePending = null;
    advance(); // 取消 = 跳过该条
  }

  return {
    // state
    step,
    links,
    index,
    parseDone,
    parseTotal,
    parsingId,
    isSubmitting,
    showDuplicate,
    duplicateTask,
    // derived
    total,
    current,
    isSingle,
    isLast,
    showAddAll,
    dirs,
    defaultDir,
    // actions
    reset,
    submitPaste,
    retryParse,
    browseSaveDir,
    addCurrent,
    skip,
    addAll,
    confirmDuplicate,
    cancelDuplicate,
  };
}
