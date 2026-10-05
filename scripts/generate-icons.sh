#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LOGO="$ROOT/branding/logo.png"

if [ ! -s "$LOGO" ]; then
    echo "branding/logo.png is empty, keeping default placeholder icons"
    exit 0
fi

if command -v magick >/dev/null 2>&1; then
    IM="magick"
elif command -v convert >/dev/null 2>&1; then
    IM="convert"
else
    echo "ImageMagick is required"
    exit 1
fi

android_res="$ROOT/android/app/src/main/res"
for entry in mipmap-mdpi:108 mipmap-hdpi:162 mipmap-xhdpi:216 mipmap-xxhdpi:324 mipmap-xxxhdpi:432; do
    dir="${entry%%:*}"
    size="${entry##*:}"
    inner=$((size * 2 / 3))
    mkdir -p "$android_res/$dir"
    "$IM" "$LOGO" -resize "${inner}x${inner}" -background none -gravity center \
        -extent "${size}x${size}" "$android_res/$dir/ic_launcher_fg.png"
done
cat > "$android_res/mipmap-anydpi-v26/ic_launcher.xml" <<'XML'
<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@color/ic_launcher_background" />
    <foreground android:drawable="@mipmap/ic_launcher_fg" />
    <monochrome android:drawable="@drawable/ic_launcher_foreground" />
</adaptive-icon>
XML

appiconset="$ROOT/apple/Resources/Assets.xcassets/AppIcon.appiconset"
mkdir -p "$appiconset"
for size in 16 32 64 128 256 512 1024; do
    "$IM" "$LOGO" -resize "${size}x${size}" "$appiconset/icon_${size}.png"
done
cat > "$appiconset/Contents.json" <<'JSON'
{
  "images": [
    { "idiom": "universal", "platform": "ios", "size": "1024x1024", "filename": "icon_1024.png" },
    { "idiom": "mac", "scale": "1x", "size": "16x16", "filename": "icon_16.png" },
    { "idiom": "mac", "scale": "2x", "size": "16x16", "filename": "icon_32.png" },
    { "idiom": "mac", "scale": "1x", "size": "32x32", "filename": "icon_32.png" },
    { "idiom": "mac", "scale": "2x", "size": "32x32", "filename": "icon_64.png" },
    { "idiom": "mac", "scale": "1x", "size": "128x128", "filename": "icon_128.png" },
    { "idiom": "mac", "scale": "2x", "size": "128x128", "filename": "icon_256.png" },
    { "idiom": "mac", "scale": "1x", "size": "256x256", "filename": "icon_256.png" },
    { "idiom": "mac", "scale": "2x", "size": "256x256", "filename": "icon_512.png" },
    { "idiom": "mac", "scale": "1x", "size": "512x512", "filename": "icon_512.png" },
    { "idiom": "mac", "scale": "2x", "size": "512x512", "filename": "icon_1024.png" }
  ],
  "info": { "author": "xcode", "version": 1 }
}
JSON

"$IM" "$LOGO" -resize 256x256 -define icon:auto-resize=16,24,32,48,64,128,256 "$ROOT/branding/icon.ico"

echo "Icons generated for Android, Apple and Windows"
