# ClipClop agent guide

## Frontend components

- The frontend uses Svelte 5, TypeScript, and the project's existing CSS tokens. Do not introduce Tailwind solely to use a component.
- `bits-ui` is the headless accessibility primitive for complex controls. Prefer it for dialogs, menus, popovers, comboboxes, selects, tabs, tooltips, and other controls that need managed keyboard navigation or focus.
- Keep simple controls native: semantic buttons, inputs, selects, switches, and static lists do not need Bits UI.
- Bits UI supplies behavior and ARIA only. Keep styling in the local component CSS and reuse the existing `--*` design tokens.
- Do not add shadcn-svelte unless its copied component source is specifically needed; use Bits UI directly first.
- Before implementing a complex interaction, read the matching Bits UI documentation and preserve its keyboard and focus behavior when customizing markup.

## Icons

- Use `@lucide/svelte` for product UI icons.
- Use `@iconify/svelte/dist/OfflineIcon.svelte` with locally imported `@iconify-icons/simple-icons/*` data for brand logos.
- Never pass string icon names to Iconify; they fetch from the public API at runtime and break the app's offline-first contract.
- Do not hand-author SVG markup when either approved source contains the icon. Keep accessible names on the owning control; decorative icon components use `aria-hidden="true"`.
