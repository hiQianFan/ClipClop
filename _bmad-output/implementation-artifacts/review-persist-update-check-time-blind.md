# Blind Hunter Review

Use the `bmad-review-adversarial-general` skill. Review only the patch below. Do not inspect the repository or infer requirements beyond the diff. Report concrete correctness, regression, accessibility, localization, and test-quality findings; omit style preferences.

```diff
diff --git a/src/lib/i18n/catalogs.ts b/src/lib/i18n/catalogs.ts
@@
-  "settings.notChecked": "Not checked yet",
+  "settings.notChecked": "Waiting for first check",
@@
-  "settings.notChecked": "尚未检查更新",
+  "settings.notChecked": "等待首次检查",
diff --git a/src/lib/settings/UpdateSettings.svelte b/src/lib/settings/UpdateSettings.svelte
@@
-      default: return update ? t("settings.newVersion", { version: update.version }) : t("settings.notChecked");
+      default: return update ? t("settings.newVersion", { version: update.version }) : lastChecked;
diff --git a/src/lib/settings/SettingsView.test.ts b/src/lib/settings/SettingsView.test.ts
@@
+    hasUpdate: true,
@@
-    get update() { return { version: "0.7.3", currentVersion: "0.7.2", date: null, notes: "" }; },
+    get update() { return preview.hasUpdate ? { version: "0.7.3", currentVersion: "0.7.2", date: null, notes: "" } : null; },
@@
+  it("shows the persisted check time after updater state resets", async () => {
+    await show("idle", { lastUpdateCheck: "2026-08-30T14:32:00+08:00", hasUpdate: false });
+    expect(screen.getAllByText(/Last checked:/).length).toBeGreaterThan(0);
+    expect(screen.queryByText("Not checked yet")).toBeNull();
+  });
+
+  it("uses an honest placeholder before the first check", async () => {
+    await show("idle", { hasUpdate: false });
+    expect(screen.getAllByText("Waiting for first check").length).toBeGreaterThan(0);
+  });
```
