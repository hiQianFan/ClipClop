# ClipClop tray icon assets

These assets are derived from the user-approved paired-hoof design. The
approved image-generation preview is retained in
`preview/approved-preview.png` as the visual reference.

## Source of truth

- `source/tray-hooves.svg`: traced vector master on a square canvas.
- `preview/approved-preview.png`: approved design reference.

The SVG was traced from the two dark connected components in the approved
preview. No generative redraw or manual replacement geometry was used.
Rendering the traced outline back to the extraction size differs from the
binary source mask by 568 pixels out of 296,700 (0.1914%).

## macOS

- `macos/trayTemplate@2x.png`: 36×36 px Retina asset.

The image is a transparent monochrome template image. Tauri constrains it to
18 pt and `icon_as_template(true)` lets macOS control its light, dark, and
selected appearance.

## Windows

- `windows/tray-light-32.png`: black mark for light taskbars.
- `windows/tray-dark-32.png`: white mark for dark taskbars.

Windows does not automatically tint tray icons like macOS template images.
ClipClop selects the matching PNG once at startup and lets Windows perform the
final notification-area scaling.

## Regeneration

Install ImageMagick and run:

```sh
brew install imagemagick
scripts/generate-tray-icons.sh
```

Potrace is only required if the approved raster reference needs to be traced
again. Normal asset regeneration uses the committed SVG master.
