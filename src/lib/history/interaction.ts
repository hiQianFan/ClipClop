export type InteractionMode =
  | "browse"
  | "search"
  | "menu"
  | "confirmation"
  | "file-tablist";

export type EscapeAction =
  | "hide-panel"
  | "exit-to-browse"
  | "close-menu"
  | "cancel-confirmation";

export function escapeAction(mode: InteractionMode): EscapeAction {
  if (mode === "confirmation") return "cancel-confirmation";
  if (mode === "menu") return "close-menu";
  if (mode === "search" || mode === "file-tablist") return "exit-to-browse";
  return "hide-panel";
}
