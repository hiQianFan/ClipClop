import { invoke } from "@tauri-apps/api/core";
import type { LanguagePreference } from "$lib/settings/api";

export type OnboardingStep = "overview" | "practice" | "auto_paste";
export type OnboardingExample = "image" | "link" | "text";
export type OnboardingState = {
  completed_revision: number | null;
  current_step: OnboardingStep | null;
  visited_steps: OnboardingStep[];
  selected_example: OnboardingExample | null;
};
export type AutoPasteReadiness =
  | "available"
  | "permission_required"
  | "available_with_elevated_target_limit"
  | "unsupported";

export const getOnboardingState = () => invoke<OnboardingState>("get_onboarding_state");
export const saveOnboardingState = (onboarding: OnboardingState) =>
  invoke<OnboardingState>("save_onboarding_state", { onboarding });
export const getAutoPasteReadiness = () =>
  invoke<AutoPasteReadiness>("get_auto_paste_readiness");
export const requestAutoPasteAccess = () =>
  invoke<boolean>("request_auto_paste_access");
export const openAutoPasteSettings = () => invoke<void>("open_auto_paste_settings");
export const saveLanguagePreference = (language: LanguagePreference) =>
  invoke<LanguagePreference>("set_language_preference", { language });

export type PreviewOutcome = "native_opened" | "native_closed" | "fallback_opened" | "not_previewable";
export const supportsOnboardingPreview = (platform: "macos" | "windows") => platform === "macos";
// Toggles native Quick Look over a built-in onboarding example string (no clipboard/DB access).
export const previewOnboardingExample = (example: OnboardingExample) =>
  invoke<PreviewOutcome>("preview_onboarding_example", { example });
