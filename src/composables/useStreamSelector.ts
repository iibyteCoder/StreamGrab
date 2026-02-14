/**
 * 流选择器组合式函数
 * 负责流选择的业务逻辑
 */

import { ref, computed, watch } from 'vue';
import type { Ref } from 'vue';
import type { StreamInfo, StreamSelection, VideoStream, AudioStream, SubtitleStream, BaseStream } from '@/types';

/**
 * 格式化比特率
 */
export function formatBandwidth(bandwidth: number): string {
  if (bandwidth >= 1000000) {
    return `${(bandwidth / 1000000).toFixed(1)} Mbps`;
  } else if (bandwidth >= 1000) {
    return `${(bandwidth / 1000).toFixed(0)} Kbps`;
  }
  return `${bandwidth} bps`;
}

/**
 * 格式化时长
 */
export function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  if (h > 0) {
    return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  }
  return `${m}:${s.toString().padStart(2, '0')}`;
}

/**
 * 获取流名称
 */
export function getStreamName(stream: BaseStream): string {
  return stream.name || stream.language || stream.id;
}

/**
 * 获取视频流描述
 */
export function getVideoDescription(stream: VideoStream): string {
  const parts: string[] = [];
  if (stream.resolution) parts.push(stream.resolution);
  if (stream.frameRate) parts.push(`${stream.frameRate}fps`);
  if (stream.videoRange && stream.videoRange !== 'SDR') parts.push(stream.videoRange);
  if (stream.codecs) parts.push(stream.codecs);
  return parts.join(' · ');
}

/**
 * 获取音频流描述
 */
export function getAudioDescription(stream: AudioStream): string {
  const parts: string[] = [];
  if (stream.channels) parts.push(`${stream.channels}ch`);
  if (stream.sampleRate) parts.push(`${(stream.sampleRate / 1000).toFixed(1)}kHz`);
  if (stream.codecs) parts.push(stream.codecs);
  return parts.join(' · ');
}

/**
 * 获取字幕流描述
 */
export function getSubtitleDescription(stream: SubtitleStream): string {
  const parts: string[] = [];
  if (stream.format) parts.push(stream.format.toUpperCase());
  if (stream.isForced) parts.push('强制');
  if (stream.isDefault) parts.push('默认');
  return parts.join(' · ');
}

/**
 * 流选择器组合式函数
 */
export function useStreamSelector(streamInfo: Ref<StreamInfo | null>) {
  // 选中的流
  const selectedVideos = ref<Set<string>>(new Set());
  const selectedAudios = ref<Set<string>>(new Set());
  const selectedSubtitles = ref<Set<string>>(new Set());

  // 当前标签页
  const activeTab = ref<'video' | 'audio' | 'subtitle'>('video');

  // 当流信息变化时，自动选择默认流
  watch(
    streamInfo,
    (info) => {
      if (!info) return;

      // 默认选择第一个视频流
      const firstVideo = info.videos[0];
      selectedVideos.value = firstVideo ? new Set([firstVideo.id]) : new Set();

      // 默认选择默认音频流或第一个
      const defaultAudio = info.audios.find((a) => a.isDefault);
      const firstAudio = info.audios[0];
      const audioId = defaultAudio?.id ?? firstAudio?.id;
      selectedAudios.value = audioId ? new Set([audioId]) : new Set();

      // 默认不选择字幕
      selectedSubtitles.value = new Set();
    },
    { immediate: true }
  );

  // 统计信息
  const stats = computed(() => {
    const info = streamInfo.value;
    if (!info) return null;

    return {
      videoCount: info.videos.length,
      audioCount: info.audios.length,
      subtitleCount: info.subtitles.length,
      duration: info.duration > 0 ? formatDuration(info.duration) : '未知',
      isLive: info.isLive,
      isEncrypted: info.isEncrypted,
    };
  });

  // 是否可以确认
  const canConfirm = computed(() => {
    return selectedVideos.value.size > 0 || selectedAudios.value.size > 0;
  });

  // 切换视频流选择（单选）
  const toggleVideo = (id: string) => {
    selectedVideos.value = selectedVideos.value.has(id) ? new Set() : new Set([id]);
  };

  // 切换音频流选择（多选）
  const toggleAudio = (id: string) => {
    const newSet = new Set(selectedAudios.value);
    newSet.has(id) ? newSet.delete(id) : newSet.add(id);
    selectedAudios.value = newSet;
  };

  // 切换字幕流选择（多选）
  const toggleSubtitle = (id: string) => {
    const newSet = new Set(selectedSubtitles.value);
    newSet.has(id) ? newSet.delete(id) : newSet.add(id);
    selectedSubtitles.value = newSet;
  };

  // 全选/取消全选音频
  const toggleAllAudio = () => {
    const audios = streamInfo.value?.audios;
    if (!audios) return;
    selectedAudios.value = selectedAudios.value.size === audios.length
      ? new Set()
      : new Set(audios.map((a) => a.id));
  };

  // 全选/取消全选字幕
  const toggleAllSubtitle = () => {
    const subtitles = streamInfo.value?.subtitles;
    if (!subtitles) return;
    selectedSubtitles.value = selectedSubtitles.value.size === subtitles.length
      ? new Set()
      : new Set(subtitles.map((s) => s.id));
  };

  // 获取选择结果
  const getSelection = (): StreamSelection => ({
    videoIds: Array.from(selectedVideos.value),
    audioIds: Array.from(selectedAudios.value),
    subtitleIds: Array.from(selectedSubtitles.value),
  });

  // 重置选择
  const reset = () => {
    selectedVideos.value = new Set();
    selectedAudios.value = new Set();
    selectedSubtitles.value = new Set();
  };

  return {
    // 状态
    activeTab,
    selectedVideos,
    selectedAudios,
    selectedSubtitles,

    // 计算属性
    stats,
    canConfirm,

    // 方法
    toggleVideo,
    toggleAudio,
    toggleSubtitle,
    toggleAllAudio,
    toggleAllSubtitle,
    getSelection,
    reset,
  };
}
