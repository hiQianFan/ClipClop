import { describe, expect, it } from "vitest";
import { en, zhCN } from "./catalogs";
import { effectiveLocale, formatDateTime, formatNumber, languagePreference, localizedError, resolveLocale, setLanguagePreference, t } from "./index.svelte";

describe("localization", () => {
  it("resolves only Chinese primary system locales to Simplified Chinese", () => {
    expect(resolveLocale("system", ["zh-TW", "en"])).toBe("zh-CN");
    expect(resolveLocale("system", ["en", "zh-CN"])).toBe("en");
    expect(resolveLocale("system", [])).toBe("en");
    expect(resolveLocale("system", ["not a locale"])).toBe("en");
    expect(resolveLocale("zh-CN", ["en"])).toBe("zh-CN");
  });

  it("switches complete catalogs and maps error codes without backend prose", () => {
    setLanguagePreference("en");
    expect(t("settings.general")).toBe("General");
    expect(localizedError({ code: "STORAGE_ERROR", message: "secret diagnostic" })).toBe("A storage operation failed.");
    expect(localizedError({ code: "HOTKEY_RESERVED" })).toBe("That combination is reserved by the system or a window.");
    expect(localizedError({ code: "UPDATE_CHANGED" })).toBe("The available version changed. Check again.");
    setLanguagePreference("zh-CN");
    expect(t("settings.general")).toBe("常规");
    expect(languagePreference()).toBe("zh-CN");
    expect(effectiveLocale()).toBe("zh-CN");
    setLanguagePreference("en");
    expect(t("settings.general")).toBe("General");
  });

  it("keeps compact locale-aware date and number presentation", () => {
    setLanguagePreference("en");
    expect(formatDateTime("2020-02-03T04:05:00Z")).toMatch(/^2020-\d{2}-\d{2} \d{2}:\d{2}$/);
    expect(formatNumber(12345)).toBe("12,345");
    setLanguagePreference("zh-CN");
    expect(formatNumber(12345)).toBe("12,345");
  });

  it("keeps catalog keys identical and rejects unresolved interpolation", () => {
    expect(Object.keys(zhCN).sort()).toEqual(Object.keys(en).sort());
    expect(() => (t as unknown as (key: string) => string)("settings.found")).toThrow(/Missing translation parameter/);

    if (false) {
      // @ts-expect-error interpolation parameters are required
      t("settings.found");
      // @ts-expect-error the parameter name must match the catalog placeholder
      t("settings.found", { wrong: "1.0" });
      // @ts-expect-error static messages do not accept parameters
      t("settings.general", { value: "unused" });
    }
  });
});
