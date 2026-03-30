/**
 * 轻量 i18n 模块
 * 支持 zh-CN（简体中文）、en（英文）
 */
import { writable, derived } from 'svelte/store';
import zhCN from './locales/zh-CN.json';
import en from './locales/en.json';

// 所有支持的语言
export const SUPPORTED_LOCALES = [
  { code: 'zh-CN', label: '简体中文' },
  { code: 'en', label: 'English' },
];

// 语言包对照表
const messages = {
  'zh-CN': zhCN,
  'en': en,
};

// localStorage key
const LOCALE_KEY = 'work-review-locale';

// 侦测默认语言
function detectLocale() {
  // 1. 优先读取 localStorage
  const saved = localStorage.getItem(LOCALE_KEY);
  if (saved && messages[saved]) return saved;

  // 2. 侦测浏览器语言
  const browserLang = navigator.language || navigator.languages?.[0] || 'zh-CN';
  if (messages[browserLang]) return browserLang;

  // 3. 模糊匹配（zh-* → zh-CN, en-US → en）
  if (browserLang.startsWith('zh')) {
    return 'zh-CN';
  }
  if (browserLang.startsWith('en')) return 'en';

  // 4. 默认简体中文
  return 'zh-CN';
}

// 当前语言 store
export const locale = writable(detectLocale());

// 切换语言（返回 true 表示成功切换）
export function setLocale(code) {
  if (messages[code]) {
    locale.set(code);
    localStorage.setItem(LOCALE_KEY, code);
    return true;
  }
  return false;
}

// 根据 key path 取得翻译值（支持 'nav.overview' 格式）
function getNestedValue(obj, path) {
  return path.split('.').reduce((o, k) => (o && o[k] !== undefined ? o[k] : undefined), obj);
}

// 翻译函数 store（reactive）
// 使用方式：$t('nav.overview') 或 $t('timeline.expandAll', { count: 5 })
export const t = derived(locale, ($locale) => {
  const dict = messages[$locale] || messages['zh-CN'];

  return (key, params) => {
    let value = getNestedValue(dict, key);

    // fallback 到 zh-CN
    if (value === undefined) {
      value = getNestedValue(messages['zh-CN'], key);
    }

    // 找不到就回传 key
    if (value === undefined) return key;

    // 数组直接回传
    if (Array.isArray(value)) return value;

    // 对象直接回传
    if (typeof value === 'object') return value;

    // 字符串插值：{date} → params.date
    if (params && typeof value === 'string') {
      return value.replace(/\{(\w+)\}/g, (_, k) => params[k] ?? `{${k}}`);
    }

    return value;
  };
});
