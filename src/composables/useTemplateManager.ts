/**
 * 模板管理组合式函数
 * 负责配置模板的业务逻辑
 */

import { ref, computed, onMounted } from "vue";
import { useTemplateStore, useSettingsStore } from "@/stores";
import { useToast } from "./useToast";
import type { LegacyConfigTemplate, LegacyAppSettings } from "@/types";

/**
 * 模板管理组合式函数
 */
export function useTemplateManager() {
  const templateStore = useTemplateStore();
  const settingsStore = useSettingsStore();
  const toast = useToast();

  const showEditDialog = ref(false);
  const showDeleteDialog = ref(false);
  const editingTemplate = ref<LegacyConfigTemplate | null>(null);
  const deletingTemplate = ref<LegacyConfigTemplate | null>(null);

  const editForm = ref({
    name: "",
    description: "",
  });

  onMounted(() => {
    templateStore.initialize();
  });

  const presetTemplates = computed(() => templateStore.presetTemplates);
  const customTemplates = computed(() => templateStore.customTemplates);

  const createFromCurrentSettings = () => {
    editingTemplate.value = null;
    editForm.value = { name: "", description: "" };
    showEditDialog.value = true;
  };

  const editTemplate = (template: LegacyConfigTemplate) => {
    if (template.id.startsWith("default-")) {
      toast.warning("无法编辑预设模板");
      return;
    }
    editingTemplate.value = template;
    editForm.value = {
      name: template.name,
      description: template.description || "",
    };
    showEditDialog.value = true;
  };

  /**
   * 从当前设置提取 LegacyAppSettings 格式
   */
  const extractCurrentSettings = (): Partial<LegacyAppSettings> => {
    const {
      m3u8dlSettings,
      networkSettings,
      decryptionSettings,
      networkHeaders,
      decryptionKeys,
    } = settingsStore;

    return {
      download: {
        threadCount: m3u8dlSettings.thread_count,
        retryCount: m3u8dlSettings.retry_count,
        timeout: m3u8dlSettings.timeout,
        maxSpeed: m3u8dlSettings.max_speed,
        autoSelect: m3u8dlSettings.auto_select,
        selectVideo: m3u8dlSettings.select_video || "",
        selectAudio: m3u8dlSettings.select_audio || "",
        selectSubtitle: m3u8dlSettings.select_subtitle || "",
        dropVideo: m3u8dlSettings.drop_video || "",
        dropAudio: m3u8dlSettings.drop_audio || "",
        dropSubtitle: m3u8dlSettings.drop_subtitle || "",
        savePattern: { enabled: false, template: "", presetId: "basic" },
        adFilter: { enabled: false, keywords: [] },
        checkSegmentsCount: m3u8dlSettings.check_segments_count,
        delAfterDone: m3u8dlSettings.del_after_done,
        skipMerge: m3u8dlSettings.skip_merge,
        writeMetaJson: m3u8dlSettings.write_meta_json,
        binaryMerge: m3u8dlSettings.binary_merge,
        concurrentDownload: m3u8dlSettings.concurrent_download,
        subOnly: m3u8dlSettings.sub_only,
        subFormat: m3u8dlSettings.sub_format,
        autoSubtitleFix: m3u8dlSettings.auto_subtitle_fix,
      },
      mux: {
        format: m3u8dlSettings.mux_format,
        muxer: m3u8dlSettings.muxer,
        binPath: m3u8dlSettings.mux_bin_path || "",
        keepOriginal: m3u8dlSettings.mux_keep_original,
        skipSubtitles: m3u8dlSettings.mux_skip_subtitles,
        noDateInfo: m3u8dlSettings.no_date_info,
        useConcatDemuxer: m3u8dlSettings.use_ffmpeg_concat_demuxer,
        muxImports: [],
      },
      network: {
        useSystemProxy: networkSettings.use_system_proxy,
        customProxy: networkSettings.custom_proxy || "",
        headers: networkHeaders.map((h) => ({
          key: h.name,
          value: h.value,
          enabled: h.enabled,
        })),
        baseUrl: networkSettings.base_url || "",
        appendUrlParams: networkSettings.append_url_params,
      },
      live: {
        performAsVod: m3u8dlSettings.live_perform_as_vod,
        realTimeMerge: m3u8dlSettings.live_real_time_merge,
        keepSegments: m3u8dlSettings.live_keep_segments,
        pipeMux: m3u8dlSettings.live_pipe_mux,
        fixVttByAudio: m3u8dlSettings.live_fix_vtt_by_audio,
        recordLimit: m3u8dlSettings.live_record_limit || "",
        waitTime: m3u8dlSettings.live_wait_time,
        takeCount: m3u8dlSettings.live_take_count,
      },
      decryption: {
        keys: decryptionKeys.map((k) => ({
          kid: k.kid || "",
          key: k.key,
        })),
        keyTextFile: decryptionSettings.key_text_file || "",
        engine: decryptionSettings.decryption_engine,
        binPath: decryptionSettings.decryption_bin_path || "",
        realTimeDecryption: decryptionSettings.real_time_decryption,
        customHls: {
          enabled: decryptionSettings.custom_hls_enabled,
          method: decryptionSettings.custom_hls_method,
          key: {
            type: decryptionSettings.custom_hls_key_type,
            value: decryptionSettings.custom_hls_key_value || "",
          },
          iv: {
            type: decryptionSettings.custom_hls_iv_type,
            value: decryptionSettings.custom_hls_iv_value || "",
          },
        },
      },
    };
  };

  const saveTemplate = () => {
    if (!editForm.value.name.trim()) {
      toast.warning("请输入模板名称");
      return false;
    }

    if (editingTemplate.value) {
      const success = templateStore.updateTemplate(editingTemplate.value.id, {
        name: editForm.value.name,
        description: editForm.value.description,
      });
      if (success) {
        toast.success("模板已更新");
      } else {
        toast.error("更新失败");
      }
      return success;
    } else {
      templateStore.addTemplate(
        editForm.value.name,
        editForm.value.description,
        extractCurrentSettings(),
      );
      toast.success("模板已创建");
      return true;
    }
  };

  const confirmDelete = (template: LegacyConfigTemplate) => {
    if (template.id.startsWith("default-")) {
      toast.warning("无法删除预设模板");
      return;
    }
    deletingTemplate.value = template;
    showDeleteDialog.value = true;
  };

  const deleteTemplate = () => {
    if (deletingTemplate.value) {
      const success = templateStore.deleteTemplate(deletingTemplate.value.id);
      if (success) {
        toast.success("模板已删除");
      } else {
        toast.error("删除失败");
      }
    }
    showDeleteDialog.value = false;
    deletingTemplate.value = null;
  };

  const duplicateTemplate = (template: LegacyConfigTemplate) => {
    const newTemplate = templateStore.duplicateTemplate(template.id);
    if (newTemplate) {
      toast.success("模板已复制");
    } else {
      toast.error("复制失败");
    }
  };

  /**
   * 应用模板 - 将模板设置应用到当前配置
   */
  const applyTemplate = async (template: LegacyConfigTemplate) => {
    const settings = template.settings;

    // 应用 download 设置
    if (settings.download) {
      const d = settings.download;
      if (d.threadCount !== undefined)
        await settingsStore.updateM3U8DLField("thread_count", d.threadCount);
      if (d.retryCount !== undefined)
        await settingsStore.updateM3U8DLField("retry_count", d.retryCount);
      if (d.timeout !== undefined)
        await settingsStore.updateM3U8DLField("timeout", d.timeout);
      if (d.maxSpeed !== undefined)
        await settingsStore.updateM3U8DLField("max_speed", d.maxSpeed);
      if (d.autoSelect !== undefined)
        await settingsStore.updateM3U8DLField("auto_select", d.autoSelect);
      if (d.selectVideo !== undefined)
        await settingsStore.updateM3U8DLField(
          "select_video",
          d.selectVideo || null,
        );
      if (d.selectAudio !== undefined)
        await settingsStore.updateM3U8DLField(
          "select_audio",
          d.selectAudio || null,
        );
      if (d.selectSubtitle !== undefined)
        await settingsStore.updateM3U8DLField(
          "select_subtitle",
          d.selectSubtitle || null,
        );
    }

    // 应用 mux 设置
    if (settings.mux) {
      const m = settings.mux;
      if (m.format !== undefined)
        await settingsStore.updateM3U8DLField("mux_format", m.format);
      if (m.muxer !== undefined)
        await settingsStore.updateM3U8DLField("muxer", m.muxer);
    }

    // 应用 network 设置
    if (settings.network) {
      const n = settings.network;
      if (n.useSystemProxy !== undefined)
        await settingsStore.updateNetworkField(
          "use_system_proxy",
          n.useSystemProxy,
        );
      if (n.customProxy !== undefined)
        await settingsStore.updateNetworkField(
          "custom_proxy",
          n.customProxy || null,
        );
      if (n.baseUrl !== undefined)
        await settingsStore.updateNetworkField("base_url", n.baseUrl || null);
    }

    // 应用 live 设置
    if (settings.live) {
      const l = settings.live;
      if (l.performAsVod !== undefined)
        await settingsStore.updateM3U8DLField(
          "live_perform_as_vod",
          l.performAsVod,
        );
      if (l.realTimeMerge !== undefined)
        await settingsStore.updateM3U8DLField(
          "live_real_time_merge",
          l.realTimeMerge,
        );
    }

    // 应用 decryption 设置
    if (settings.decryption) {
      const dec = settings.decryption;
      if (dec.engine !== undefined)
        await settingsStore.updateDecryptionField(
          "decryption_engine",
          dec.engine,
        );
      if (dec.keyTextFile !== undefined)
        await settingsStore.updateDecryptionField(
          "key_text_file",
          dec.keyTextFile || null,
        );
    }

    toast.success(`已应用模板: ${template.name}`);
  };

  const closeEditDialog = () => {
    showEditDialog.value = false;
  };

  const closeDeleteDialog = () => {
    showDeleteDialog.value = false;
    deletingTemplate.value = null;
  };

  return {
    showEditDialog,
    showDeleteDialog,
    editingTemplate,
    deletingTemplate,
    editForm,
    presetTemplates,
    customTemplates,
    createFromCurrentSettings,
    editTemplate,
    saveTemplate,
    confirmDelete,
    deleteTemplate,
    duplicateTemplate,
    applyTemplate,
    closeEditDialog,
    closeDeleteDialog,
  };
}
