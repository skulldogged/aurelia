#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC_ICON="${AURELIA_APP_ICON_SOURCE:-$ROOT/apps/desktop/tauri/src-tauri/icons/icon.png}"

IOS_ASSETS_DIR="$ROOT/apps/mobile/ios/Aurelia/Assets.xcassets"
IOS_ICONSET_DIR="$IOS_ASSETS_DIR/AppIcon.appiconset"

if [[ ! -f "$SRC_ICON" ]]; then
  echo "Missing source icon: $SRC_ICON" >&2
  exit 1
fi

mkdir -p "$IOS_ICONSET_DIR"

resize() {
  local px="$1"
  local out="$2"
  if node -e "require.resolve('sharp')" >/dev/null 2>&1; then
    node -e '
      const sharp = require("sharp");
      const [src, out, size] = process.argv.slice(1);
      sharp(src)
        .resize(Number(size), Number(size), { fit: "fill" })
        .png()
        .toFile(out)
        .catch(error => {
          console.error(error);
          process.exit(1);
        });
    ' "$SRC_ICON" "$out" "$px"
  else
    sips -s format png -z "$px" "$px" "$SRC_ICON" --out "$out" >/dev/null
  fi
}

resize_dark() {
  local px="$1"
  local out="$2"
  if node -e "require.resolve('sharp')" >/dev/null 2>&1; then
    node -e '
      const sharp = require("sharp");
      const [src, out, size] = process.argv.slice(1);
      sharp(src)
        .resize(Number(size), Number(size), { fit: "fill" })
        .modulate({ brightness: 0.55, saturation: 0.8 })
        .png()
        .toFile(out)
        .catch(error => {
          console.error(error);
          process.exit(1);
        });
    ' "$SRC_ICON" "$out" "$px"
  else
    sips -s format png -z "$px" "$px" "$SRC_ICON" --out "$out" >/dev/null
  fi
}

resize_tinted() {
  local px="$1"
  local out="$2"
  if node -e "require.resolve('sharp')" >/dev/null 2>&1; then
    node -e '
      const sharp = require("sharp");
      const [src, out, size] = process.argv.slice(1);
      const n = Number(size);
      (async () => {
        const alpha = await sharp(src)
          .resize(n, n, { fit: "fill" })
          .ensureAlpha()
          .extractChannel("alpha")
          .toBuffer();
        await sharp({
          create: {
            width: n,
            height: n,
            channels: 3,
            background: { r: 255, g: 255, b: 255 }
          }
        })
          .joinChannel(alpha)
          .png()
          .toFile(out);
      })().catch(error => {
        console.error(error);
        process.exit(1);
      });
    ' "$SRC_ICON" "$out" "$px"
  else
    sips -s format png -z "$px" "$px" "$SRC_ICON" --out "$out" >/dev/null
  fi
}

# iOS + iPadOS icon images
resize 40 "$IOS_ICONSET_DIR/iphone-notification-20@2x.png"
resize 60 "$IOS_ICONSET_DIR/iphone-notification-20@3x.png"
resize 58 "$IOS_ICONSET_DIR/iphone-settings-29@2x.png"
resize 87 "$IOS_ICONSET_DIR/iphone-settings-29@3x.png"
resize 80 "$IOS_ICONSET_DIR/iphone-spotlight-40@2x.png"
resize 120 "$IOS_ICONSET_DIR/iphone-spotlight-40@3x.png"
resize 120 "$IOS_ICONSET_DIR/iphone-app-60@2x.png"
resize 180 "$IOS_ICONSET_DIR/iphone-app-60@3x.png"
resize 20 "$IOS_ICONSET_DIR/ipad-notification-20@1x.png"
resize 40 "$IOS_ICONSET_DIR/ipad-notification-20@2x.png"
resize 29 "$IOS_ICONSET_DIR/ipad-settings-29@1x.png"
resize 58 "$IOS_ICONSET_DIR/ipad-settings-29@2x.png"
resize 40 "$IOS_ICONSET_DIR/ipad-spotlight-40@1x.png"
resize 80 "$IOS_ICONSET_DIR/ipad-spotlight-40@2x.png"
resize 76 "$IOS_ICONSET_DIR/ipad-app-76@1x.png"
resize 152 "$IOS_ICONSET_DIR/ipad-app-76@2x.png"
resize 167 "$IOS_ICONSET_DIR/ipad-pro-app-83.5@2x.png"
resize 1024 "$IOS_ICONSET_DIR/ios-marketing-1024@1x.png"
resize_dark 1024 "$IOS_ICONSET_DIR/ios-dark-1024@1x.png"
resize_tinted 1024 "$IOS_ICONSET_DIR/ios-tinted-1024@1x.png"

cat >"$IOS_ICONSET_DIR/Contents.json" <<'EOF'
{
  "images" : [
    {
      "filename" : "iphone-notification-20@2x.png",
      "idiom" : "iphone",
      "scale" : "2x",
      "size" : "20x20"
    },
    {
      "filename" : "iphone-notification-20@3x.png",
      "idiom" : "iphone",
      "scale" : "3x",
      "size" : "20x20"
    },
    {
      "filename" : "iphone-settings-29@2x.png",
      "idiom" : "iphone",
      "scale" : "2x",
      "size" : "29x29"
    },
    {
      "filename" : "iphone-settings-29@3x.png",
      "idiom" : "iphone",
      "scale" : "3x",
      "size" : "29x29"
    },
    {
      "filename" : "iphone-spotlight-40@2x.png",
      "idiom" : "iphone",
      "scale" : "2x",
      "size" : "40x40"
    },
    {
      "filename" : "iphone-spotlight-40@3x.png",
      "idiom" : "iphone",
      "scale" : "3x",
      "size" : "40x40"
    },
    {
      "filename" : "iphone-app-60@2x.png",
      "idiom" : "iphone",
      "scale" : "2x",
      "size" : "60x60"
    },
    {
      "filename" : "iphone-app-60@3x.png",
      "idiom" : "iphone",
      "scale" : "3x",
      "size" : "60x60"
    },
    {
      "filename" : "ipad-notification-20@1x.png",
      "idiom" : "ipad",
      "scale" : "1x",
      "size" : "20x20"
    },
    {
      "filename" : "ipad-notification-20@2x.png",
      "idiom" : "ipad",
      "scale" : "2x",
      "size" : "20x20"
    },
    {
      "filename" : "ipad-settings-29@1x.png",
      "idiom" : "ipad",
      "scale" : "1x",
      "size" : "29x29"
    },
    {
      "filename" : "ipad-settings-29@2x.png",
      "idiom" : "ipad",
      "scale" : "2x",
      "size" : "29x29"
    },
    {
      "filename" : "ipad-spotlight-40@1x.png",
      "idiom" : "ipad",
      "scale" : "1x",
      "size" : "40x40"
    },
    {
      "filename" : "ipad-spotlight-40@2x.png",
      "idiom" : "ipad",
      "scale" : "2x",
      "size" : "40x40"
    },
    {
      "filename" : "ipad-app-76@1x.png",
      "idiom" : "ipad",
      "scale" : "1x",
      "size" : "76x76"
    },
    {
      "filename" : "ipad-app-76@2x.png",
      "idiom" : "ipad",
      "scale" : "2x",
      "size" : "76x76"
    },
    {
      "filename" : "ipad-pro-app-83.5@2x.png",
      "idiom" : "ipad",
      "scale" : "2x",
      "size" : "83.5x83.5"
    },
    {
      "filename" : "ios-marketing-1024@1x.png",
      "idiom" : "ios-marketing",
      "scale" : "1x",
      "size" : "1024x1024"
    },
    {
      "appearances" : [
        {
          "appearance" : "luminosity",
          "value" : "dark"
        }
      ],
      "filename" : "ios-dark-1024@1x.png",
      "idiom" : "universal",
      "platform" : "ios",
      "size" : "1024x1024"
    },
    {
      "appearances" : [
        {
          "appearance" : "luminosity",
          "value" : "tinted"
        }
      ],
      "filename" : "ios-tinted-1024@1x.png",
      "idiom" : "universal",
      "platform" : "ios",
      "size" : "1024x1024"
    }
  ],
  "info" : {
    "author" : "xcode",
    "version" : 1
  }
}
EOF

echo "Generated iOS/iPadOS app icons from $SRC_ICON"
