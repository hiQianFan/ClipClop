import { en, zhCN, type MessageKey } from "./catalogs";
import type { LanguagePreference } from "$lib/settings/api";

export type EffectiveLocale = "en" | "zh-CN";
type PlaceholderNames<Value extends string> =
  Value extends `${string}{${infer Name}}${infer Rest}` ? Name | PlaceholderNames<Rest> : never;
export type StaticMessageKey = {
  [Key in MessageKey]: [PlaceholderNames<(typeof en)[Key]>] extends [never] ? Key : never
}[MessageKey];
type MessageParameters<Key extends MessageKey> = Record<PlaceholderNames<(typeof en)[Key]>, string | number>;
type TranslationArguments<Key extends MessageKey> =
  [PlaceholderNames<(typeof en)[Key]>] extends [never] ? [] : [parameters: MessageParameters<Key>];

let preference = $state<LanguagePreference>("system");
let locale = $state<EffectiveLocale>("en");

export function resolveLocale(value: LanguagePreference, languages: readonly string[] = []): EffectiveLocale {
  if (value === "en" || value === "zh-CN") return value;
  const primary = languages[0]?.trim();
  if (!primary) return "en";
  try {
    return new Intl.Locale(primary).language.toLowerCase() === "zh" ? "zh-CN" : "en";
  } catch {
    return "en";
  }
}

function systemLanguages(): readonly string[] {
  return typeof navigator === "undefined" ? [] : navigator.languages;
}

export function setLanguagePreference(value: LanguagePreference) {
  preference = value;
  locale = resolveLocale(value, systemLanguages());
  if (typeof document !== "undefined") document.documentElement.lang = locale;
}

export function languagePreference() { return preference; }
export function effectiveLocale() { return locale; }

export function t<Key extends MessageKey>(key: Key, ...args: TranslationArguments<Key>) {
  const catalog: Record<MessageKey, string> = locale === "zh-CN" ? zhCN : en;
  const parameters = (args[0] ?? {}) as Record<string, string | number>;
  const translated = catalog[key].replace(/\{(\w+)\}/g, (_match: string, name: string) =>
    Object.hasOwn(parameters, name) ? String(parameters[name]) : `{${name}}`);
  if (/\{\w+\}/.test(translated)) throw new Error(`Missing translation parameter for ${key}`);
  return translated;
}

export function formatNumber(value: number, options?: Intl.NumberFormatOptions) {
  return new Intl.NumberFormat(locale, options).format(value);
}

export function formatDateTime(value: string | Date) {
  const date = value instanceof Date ? value : new Date(value);
  if (Number.isNaN(date.getTime())) return "";
  const now = new Date();
  const includeYear = date.getFullYear() !== now.getFullYear();
  const parts = new Intl.DateTimeFormat(locale, {
    ...(includeYear ? { year: "numeric" as const } : {}), month: "2-digit", day: "2-digit",
    hour: "2-digit", minute: "2-digit", hour12: false,
  }).formatToParts(date);
  const get = (type: Intl.DateTimeFormatPartTypes) => parts.find((part) => part.type === type)?.value ?? "";
  const year = includeYear ? `${get("year")}-` : "";
  return `${year}${get("month")}-${get("day")} ${get("hour")}:${get("minute")}`;
}

const errorKeys: Record<string, StaticMessageKey> = {
  NOT_FOUND: "error.notFound", VALIDATION_ERROR: "error.validation",
  HOTKEY_INVALID_FORMAT: "error.hotkeyInvalidFormat", HOTKEY_MISSING_MODIFIER: "error.hotkeyMissingModifier",
  HOTKEY_UNSUPPORTED_KEY: "error.hotkeyUnsupportedKey", HOTKEY_DUPLICATE_MODIFIER: "error.hotkeyDuplicateModifier",
  HOTKEY_RESERVED: "error.hotkeyReserved", HOTKEY_UNAVAILABLE: "error.hotkeyUnavailable",
  STORAGE_ERROR: "error.storage", CLIPBOARD_ERROR: "error.clipboard", PLATFORM_ERROR: "error.platform",
  UPDATE_UNSUPPORTED: "update.unsupported", UPDATE_CHANGED: "update.changed",
};

function errorCode(reason: unknown) {
  return typeof reason === "object" && reason && "code" in reason ? String(reason.code) : "";
}

export function localizedError(reason: unknown) {
  return t(errorKeys[errorCode(reason)] ?? "error.unknown");
}

// The updater surfaces plain Error/string failures from the Tauri plugin (network,
// extraction, install), not backend command results with sensitive prose. When there is
// no recognized code, show the real message so install failures are diagnosable instead
// of collapsing every cause into a single "unknown error".
export function localizedUpdateError(reason: unknown) {
  const code = errorCode(reason);
  if (code && errorKeys[code]) return t(errorKeys[code]);
  const message = reason instanceof Error
    ? reason.message
    : typeof reason === "string"
      ? reason
      : typeof reason === "object" && reason && "message" in reason
        ? String((reason as { message: unknown }).message)
        : "";
  const trimmed = message.trim();
  return trimmed ? trimmed.slice(0, 300) : t("error.unknown");
}
