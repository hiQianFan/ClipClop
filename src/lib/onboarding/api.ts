import { invoke } from "@tauri-apps/api/core";
import type { LanguagePreference } from "$lib/settings/api";

export type OnboardingStep = "overview" | "practice" | "auto_paste" | "file_preview";
export type OnboardingExample = "image" | "link" | "text";
export type OnboardingState = {
  completed_revision: number | null;
  current_step: OnboardingStep | null;
  visited_steps: OnboardingStep[];
  selected_example: OnboardingExample | null;
};
export const getOnboardingState = () => invoke<OnboardingState>("get_onboarding_state");
export const saveOnboardingState = (onboarding: OnboardingState) =>
  invoke<OnboardingState>("save_onboarding_state", { onboarding });
export const openAutoPasteSettings = () => invoke<void>("open_auto_paste_settings");
export const saveLanguagePreference = (language: LanguagePreference) =>
  invoke<LanguagePreference>("set_language_preference", { language });

export type PreviewOutcome = "native_opened" | "native_closed" | "not_previewable";
export const supportsOnboardingPreview = (platform: "macos" | "windows") => platform === "macos";
// Sets native Quick Look to the requested state over a built-in example (no clipboard/DB access).
export const previewOnboardingExample = (example: OnboardingExample, open: boolean) =>
  invoke<PreviewOutcome>("preview_onboarding_example", { example, open });
