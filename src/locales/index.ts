/**
 * i18n 配置
 * 多语言支持
 */

import { createI18n } from 'vue-i18n';
import type { Composer } from 'vue-i18n';

// 导入语言包
import zhCN from './zh-CN';
import zhTW from './zh-TW';
import enUS from './en-US';

// 语言消息
const messages = {
  'zh-CN': zhCN,
  'zh-TW': zhTW,
  'en-US': enUS,
};

// 获取存储的语言或浏览器语言
function getDefaultLocale(): string {
  // 从 localStorage 获取
  const stored = localStorage.getItem('locale');
  if (stored && messages[stored as keyof typeof messages]) {
    return stored;
  }

  // 获取浏览器语言
  const browserLang = navigator.language;
  if (browserLang.startsWith('zh-TW') || browserLang.startsWith('zh-Hant')) {
    return 'zh-TW';
  }
  if (browserLang.startsWith('zh')) {
    return 'zh-CN';
  }
  if (browserLang.startsWith('en')) {
    return 'en-US';
  }

  // 默认简体中文
  return 'zh-CN';
}

// 创建 i18n 实例
export const i18n = createI18n({
  legacy: false, // 使用 Composition API 模式
  locale: getDefaultLocale(),
  fallbackLocale: 'zh-CN',
  messages,
  globalInjection: true, // 全局注入 $t
});

// 获取 composer 实例（用于 Composition API 模式）
function getComposer(): Composer<{}, {}, {}, string> {
  return i18n.global as unknown as Composer<{}, {}, {}, string>;
}

// 设置语言
export function setLocale(locale: string): void {
  if (messages[locale as keyof typeof messages]) {
    getComposer().locale.value = locale;
    localStorage.setItem('locale', locale);
    document.documentElement.setAttribute('lang', locale);
  }
}

// 获取当前语言
export function getLocale(): string {
  return getComposer().locale.value;
}

// 获取支持的语言列表
export function getSupportedLocales(): Array<{ value: string; label: string }> {
  return [
    { value: 'zh-CN', label: '简体中文' },
    { value: 'zh-TW', label: '繁體中文' },
    { value: 'en-US', label: 'English' },
  ];
}

export default i18n;
