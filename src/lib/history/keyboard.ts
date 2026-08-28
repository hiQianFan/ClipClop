export type WindowKeyContext = {
  view: "history" | "settings" | "onboarding" | "loading";
  mode: "browse" | "search" | "menu" | "confirmation" | "file-tablist";
  deletePending: boolean;
  menuOpen: boolean;
  appMenuOpen: boolean;
};

export type WindowKeyAction = "dismiss-panel" | "cancel-delete" | "close-menu" | "close-app-menu" | "return-to-browse";

export function exitsSearch(key: string) {
  return key === "Escape" || key === "ArrowDown" || key === "ArrowUp";
}

export function routeWindowKey(
  event: Pick<KeyboardEvent, "key" | "ctrlKey" | "metaKey" | "defaultPrevented">,
  context: WindowKeyContext,
): WindowKeyAction | null {
  if (event.defaultPrevented) return null;
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "w") return "dismiss-panel";
  if (event.key !== "Escape" || context.view !== "history") return null;
  if (context.deletePending) return "cancel-delete";
  if (context.menuOpen) return "close-menu";
  if (context.appMenuOpen) return "close-app-menu";
  if (context.mode === "search" || context.mode === "file-tablist") return "return-to-browse";
  return "dismiss-panel";
}
