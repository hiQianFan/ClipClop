# ClipClop tray icon assets

Tray icons intentionally follow each platform instead of sharing one rendered
asset.

## Source of truth

- `source/tray-hooves.svg`: monochrome macOS master on a square canvas.
- `source/tray-cow-horse.png`: transparent Windows master extracted from the
  app icon without generative redraw.
- `preview/approved-preview.png`: approved macOS hoof reference.

The macOS SVG was traced from the two dark connected components in the
approved preview. The Windows PNG preserves the original cow-and-horse pixels;
macOS Vision removed the clipboard and background, then the result was placed
on a transparent square canvas. Neither asset uses a generative redraw.

## macOS

- `macos/trayTemplate@2x.png`: 36×36 px Retina asset.

The image is a transparent monochrome template. Tauri constrains it to 18 pt
and `icon_as_template(true)` lets macOS tint it for light, dark, selected, and
accessibility appearances. Do not put brand colors in this asset: template
images intentionally discard them.

## Windows

- `windows/tray-32.png`: 32×32 px full-color cow-and-horse asset.

Windows notification-area icons keep their own colors and alpha instead of
using macOS-style template tinting. The app mark is therefore used without its
white background, with one transparent pixel of minimum padding. Windows may
display it at 16 px, so preserve the tight crop and always inspect the generated
32 px asset on both light and dark taskbars. No theme-specific variants are
needed because the colored mark has its own contrast.

## Regeneration

Install ImageMagick and run:

```sh
brew install imagemagick
scripts/generate-tray-icons.sh
```

Potrace is only required if the approved raster reference needs to be traced
again. Normal asset regeneration uses the committed SVG master.
