<script setup lang="ts">
/**
 * AdvancedSettings - 高级设置组件
 */

import { ref, onMounted, computed } from "vue";
import {
  RotateCcw,
  AlertTriangle,
  Check,
  X,
  Download,
  Loader2,
} from "lucide-vue-next";
import { Button } from "@/components/ui/button";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { SettingSelect, SettingInput, SettingSwitch, SettingsGroup } from "..";
import {
  getNm3u8dlInfo,
  getFfmpegInfo,
  getNm3u8dlLatestRelease,
  getFfmpegLatestRelease,
  downloadTool,
  type ToolInfo,
  type ToolReleaseInfo,
  type DownloadProgress,
} from "@/services";
import { useToast } from "@/composables";
import { appConfigDir, join } from "@tauri-apps/api/path";

interface Settings {
  advanced: {
    ffmpegPath: string;
    n_m3u8dlPath: string;
    logLevel: string;
    logFilePath: string;
    noLog: boolean;
    allowHlsMultiExtMap: boolean;
    disableUpdateCheck: boolean;
    urlProcessorArgs: string;
  };
}

interface Props {
  settings: Settings;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "update:settings", value: any): void;
  (e: "reset"): void;
}>();

const toast = useToast();

// 工具状态
const nm3u8dlInfo = ref<ToolInfo | null>(null);
const ffmpegInfo = ref<ToolInfo | null>(null);
const isLoadingTools = ref(false);

// 最新版本信息
const nm3u8dlLatest = ref<ToolReleaseInfo | null>(null);
const ffmpegLatest = ref<ToolReleaseInfo | null>(null);

// 下载状态
const isDownloadingNm3u8dl = ref(false);
const isDownloadingFfmpeg = ref(false);
const downloadProgress = ref<DownloadProgress | null>(null);

// 日志级别选项
const logLevelOptions = [
  { value: "DEBUG", label: "调试" },
  { value: "INFO", label: "信息" },
  { value: "WARN", label: "警告" },
  { value: "ERROR", label: "错误" },
  { value: "OFF", label: "关闭" },
];

// 计算是否有更新可用
const hasNm3u8dlUpdate = computed(() => {
  if (!nm3u8dlInfo.value?.version || !nm3u8dlLatest.value?.version)
    return false;
  return (
    compareVersions(nm3u8dlLatest.value.version, nm3u8dlInfo.value.version) > 0
  );
});

const hasFfmpegUpdate = computed(() => {
  if (!ffmpegInfo.value?.version || !ffmpegLatest.value?.version) return false;
  return (
    compareVersions(ffmpegLatest.value.version, ffmpegInfo.value.version) > 0
  );
});

// 更新设置
const updateAdvanced = (value: any) => {
  emit("update:settings", value);
};

// 重置设置
const handleReset = () => {
  emit("reset");
};

// 比较版本号
function compareVersions(v1: string, v2: string): number {
  const parts1 = v1.replace(/^v/, "").split(".").map(Number);
  const parts2 = v2.replace(/^v/, "").split(".").map(Number);

  for (let i = 0; i < Math.max(parts1.length, parts2.length); i++) {
    const p1 = parts1[i] || 0;
    const p2 = parts2[i] || 0;
    if (p1 > p2) return 1;
    if (p1 < p2) return -1;
  }
  return 0;
}

// 检查工具状态
async function checkToolsStatus() {
  isLoadingTools.value = true;
  try {
    const [nm3u8dl, ffmpeg] = await Promise.all([
      getNm3u8dlInfo(props.settings.advanced.n_m3u8dlPath || undefined),
      getFfmpegInfo(props.settings.advanced.ffmpegPath || undefined),
    ]);
    nm3u8dlInfo.value = nm3u8dl;
    ffmpegInfo.value = ffmpeg;
  } catch (e) {
    console.error("Failed to check tools status:", e);
  } finally {
    isLoadingTools.value = false;
  }
}

// 获取最新版本信息
async function fetchLatestReleases() {
  try {
    const [nm3u8dl, ffmpeg] = await Promise.all([
      getNm3u8dlLatestRelease(),
      getFfmpegLatestRelease(),
    ]);
    nm3u8dlLatest.value = nm3u8dl;
    ffmpegLatest.value = ffmpeg;
  } catch (e) {
    console.error("Failed to fetch latest releases:", e);
  }
}

// 下载 N_m3u8DL-RE
async function handleDownloadNm3u8dl() {
  if (!nm3u8dlLatest.value) {
    toast.warning("正在获取最新版本信息...");
    await fetchLatestReleases();
    if (!nm3u8dlLatest.value) {
      toast.error("获取最新版本信息失败");
      return;
    }
  }

  isDownloadingNm3u8dl.value = true;
  downloadProgress.value = null;

  try {
    const configDir = await appConfigDir();
    const toolsDir = await join(configDir, "tools");

    const extractedPath = await downloadTool(
      "N_m3u8DL-RE",
      nm3u8dlLatest.value.downloadUrl,
      toolsDir,
      (progress) => {
        downloadProgress.value = progress;
      },
    );

    // 更新设置中的路径
    updateAdvanced({ n_m3u8dlPath: extractedPath });

    toast.success("N_m3u8DL-RE 下载完成");

    // 重新检查状态
    await checkToolsStatus();
  } catch (e) {
    console.error("Failed to download N_m3u8DL-RE:", e);
    toast.error(`下载失败: ${e}`);
  } finally {
    isDownloadingNm3u8dl.value = false;
    downloadProgress.value = null;
  }
}

// 下载 FFmpeg
async function handleDownloadFfmpeg() {
  if (!ffmpegLatest.value) {
    toast.warning("正在获取最新版本信息...");
    await fetchLatestReleases();
    if (!ffmpegLatest.value) {
      toast.error("获取最新版本信息失败");
      return;
    }
  }

  isDownloadingFfmpeg.value = true;
  downloadProgress.value = null;

  try {
    const configDir = await appConfigDir();
    const toolsDir = await join(configDir, "tools");

    const extractedPath = await downloadTool(
      "FFmpeg",
      ffmpegLatest.value.downloadUrl,
      toolsDir,
      (progress) => {
        downloadProgress.value = progress;
      },
    );

    // 更新设置中的路径
    updateAdvanced({ ffmpegPath: extractedPath });

    toast.success("FFmpeg 下载完成");

    // 重新检查状态
    await checkToolsStatus();
  } catch (e) {
    console.error("Failed to download FFmpeg:", e);
    toast.error(`下载失败: ${e}`);
  } finally {
    isDownloadingFfmpeg.value = false;
    downloadProgress.value = null;
  }
}

// 组件挂载时检查工具状态
onMounted(async () => {
  await checkToolsStatus();
  await fetchLatestReleases();
});
</script>

<template>
  <div class="space-y-2">
    <SettingsGroup title="工具路径" description="配置外部工具的路径">
      <!-- N_m3u8DL-RE 设置 -->
      <div class="space-y-3">
        <div class="flex items-end gap-2">
          <div class="flex-1">
            <SettingInput
              :model-value="settings.advanced.n_m3u8dlPath"
              label="N_m3u8DL-RE 路径"
              placeholder="留空则使用系统 PATH"
              @update:model-value="updateAdvanced({ n_m3u8dlPath: $event })"
            />
          </div>
          <Button
            variant="outline"
            size="sm"
            :disabled="isDownloadingNm3u8dl"
            @click="handleDownloadNm3u8dl"
          >
            <Loader2
              v-if="isDownloadingNm3u8dl"
              class="mr-2 h-4 w-4 animate-spin"
            />
            <Download v-else class="mr-2 h-4 w-4" />
            {{
              isDownloadingNm3u8dl
                ? "下载中..."
                : hasNm3u8dlUpdate
                  ? "更新"
                  : "下载"
            }}
          </Button>
        </div>

        <!-- N_m3u8DL-RE 状态显示 -->
        <div v-if="nm3u8dlInfo" class="flex items-center gap-2 text-xs">
          <template v-if="nm3u8dlInfo.installed">
            <Check class="h-3.5 w-3.5 text-green-500" />
            <span class="text-muted-foreground">
              已安装 v{{ nm3u8dlInfo.version || "未知" }}
            </span>
            <span v-if="hasNm3u8dlUpdate" class="text-primary">
              (有新版本 {{ nm3u8dlLatest?.version }})
            </span>
          </template>
          <template v-else>
            <X class="h-3.5 w-3.5 text-red-500" />
            <span class="text-destructive">
              {{ nm3u8dlInfo.error || "未找到" }}
            </span>
          </template>
        </div>

        <!-- 下载进度 -->
        <div
          v-if="isDownloadingNm3u8dl && downloadProgress"
          class="text-xs text-muted-foreground"
        >
          <span v-if="downloadProgress.status === 'downloading'">
            下载中... {{ downloadProgress.percent.toFixed(1) }}%
          </span>
          <span v-else-if="downloadProgress.status === 'extracting'">
            解压中...
          </span>
        </div>
      </div>

      <!-- FFmpeg 设置 -->
      <div class="space-y-3">
        <div class="flex items-end gap-2">
          <div class="flex-1">
            <SettingInput
              :model-value="settings.advanced.ffmpegPath"
              label="FFmpeg 路径"
              placeholder="留空则使用系统 PATH"
              @update:model-value="updateAdvanced({ ffmpegPath: $event })"
            />
          </div>
          <Button
            variant="outline"
            size="sm"
            :disabled="isDownloadingFfmpeg"
            @click="handleDownloadFfmpeg"
          >
            <Loader2
              v-if="isDownloadingFfmpeg"
              class="mr-2 h-4 w-4 animate-spin"
            />
            <Download v-else class="mr-2 h-4 w-4" />
            {{
              isDownloadingFfmpeg
                ? "下载中..."
                : hasFfmpegUpdate
                  ? "更新"
                  : "下载"
            }}
          </Button>
        </div>

        <!-- FFmpeg 状态显示 -->
        <div v-if="ffmpegInfo" class="flex items-center gap-2 text-xs">
          <template v-if="ffmpegInfo.installed">
            <Check class="h-3.5 w-3.5 text-green-500" />
            <span class="text-muted-foreground">
              已安装 v{{ ffmpegInfo.version || "未知" }}
            </span>
            <span v-if="hasFfmpegUpdate" class="text-primary">
              (有新版本 {{ ffmpegLatest?.version }})
            </span>
          </template>
          <template v-else>
            <X class="h-3.5 w-3.5 text-red-500" />
            <span class="text-destructive">
              {{ ffmpegInfo.error || "未找到" }}
            </span>
          </template>
        </div>

        <!-- 下载进度 -->
        <div
          v-if="isDownloadingFfmpeg && downloadProgress"
          class="text-xs text-muted-foreground"
        >
          <span v-if="downloadProgress.status === 'downloading'">
            下载中... {{ downloadProgress.percent.toFixed(1) }}%
          </span>
          <span v-else-if="downloadProgress.status === 'extracting'">
            解压中...
          </span>
        </div>
      </div>
    </SettingsGroup>

    <SettingsGroup title="日志设置">
      <SettingSelect
        :model-value="settings.advanced.logLevel"
        label="日志级别"
        :options="logLevelOptions"
        placeholder="选择级别"
        @update:model-value="updateAdvanced({ logLevel: $event })"
      />

      <SettingInput
        :model-value="settings.advanced.logFilePath"
        label="日志文件路径"
        placeholder="留空则不写入文件"
        @update:model-value="updateAdvanced({ logFilePath: $event })"
      />

      <SettingSwitch
        :model-value="settings.advanced.noLog"
        label="禁用日志"
        @update:model-value="updateAdvanced({ noLog: $event })"
      />
    </SettingsGroup>

    <SettingsGroup title="实验性功能">
      <SettingSwitch
        :model-value="settings.advanced.allowHlsMultiExtMap"
        label="允许多 EXT-X-MAP"
        description="允许 HLS 多个 EXT-X-MAP 标签"
        @update:model-value="updateAdvanced({ allowHlsMultiExtMap: $event })"
      />

      <SettingSwitch
        :model-value="settings.advanced.disableUpdateCheck"
        label="禁用更新检查"
        @update:model-value="updateAdvanced({ disableUpdateCheck: $event })"
      />

      <SettingInput
        :model-value="settings.advanced.urlProcessorArgs"
        label="URL 处理器参数"
        placeholder="传递给 URL 处理器的额外参数"
        @update:model-value="updateAdvanced({ urlProcessorArgs: $event })"
      />
    </SettingsGroup>

    <!-- 恢复默认配置 -->
    <section class="mb-8 last:mb-0">
      <div class="mb-4">
        <h3
          class="text-sm font-medium text-destructive flex items-center gap-2"
        >
          <AlertTriangle class="h-4 w-4" />
          危险操作
        </h3>
        <p class="mt-1 text-xs text-muted-foreground">
          以下操作不可撤销，请谨慎使用
        </p>
      </div>

      <div
        class="space-y-5 rounded-lg border border-destructive/20 bg-card/50 p-4"
      >
        <div class="flex items-center justify-between">
          <div class="space-y-0.5">
            <p class="text-sm font-medium">恢复默认配置</p>
            <p class="text-xs text-muted-foreground">将所有设置恢复为默认值</p>
          </div>
          <AlertDialog>
            <AlertDialogTrigger as-child>
              <Button
                variant="outline"
                size="sm"
                class="text-destructive border-destructive/30 hover:bg-destructive/10"
              >
                <RotateCcw class="mr-2 h-4 w-4" />
                恢复默认
              </Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>确认恢复默认配置？</AlertDialogTitle>
                <AlertDialogDescription>
                  此操作将把所有设置恢复为默认值，当前配置将被覆盖。此操作无法撤销。
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>取消</AlertDialogCancel>
                <AlertDialogAction @click="handleReset">
                  确认恢复
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        </div>
      </div>
    </section>
  </div>
</template>
