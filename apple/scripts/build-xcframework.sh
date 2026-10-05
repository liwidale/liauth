#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
APPLE_DIR="$ROOT/apple"
OUT_DIR="$APPLE_DIR/Frameworks"
GEN_DIR="$APPLE_DIR/Sources/Generated"
BUILD_DIR="$ROOT/target"

IOS_TARGETS=("aarch64-apple-ios")
IOS_SIM_TARGETS=("aarch64-apple-ios-sim" "x86_64-apple-ios")
MACOS_TARGETS=("aarch64-apple-darwin" "x86_64-apple-darwin")

for target in "${IOS_TARGETS[@]}" "${IOS_SIM_TARGETS[@]}" "${MACOS_TARGETS[@]}"; do
    rustup target add "$target"
    cargo build --release -p liauth-ffi --target "$target"
done

rm -rf "$GEN_DIR" "$OUT_DIR/LiAuthCore.xcframework"
mkdir -p "$GEN_DIR" "$OUT_DIR"

cargo run --release -p liauth-ffi --bin uniffi-bindgen -- generate \
    --library "$BUILD_DIR/${MACOS_TARGETS[0]}/release/libliauth.dylib" \
    --language swift \
    --out-dir "$GEN_DIR"

HEADERS_DIR="$BUILD_DIR/uniffi-headers"
rm -rf "$HEADERS_DIR"
mkdir -p "$HEADERS_DIR"
cp "$GEN_DIR"/*.h "$HEADERS_DIR/"
cat "$GEN_DIR"/*.modulemap > "$HEADERS_DIR/module.modulemap"
rm -f "$GEN_DIR"/*.h "$GEN_DIR"/*.modulemap

SIM_LIB="$BUILD_DIR/libliauth-ios-sim.a"
lipo -create \
    "$BUILD_DIR/aarch64-apple-ios-sim/release/libliauth.a" \
    "$BUILD_DIR/x86_64-apple-ios/release/libliauth.a" \
    -output "$SIM_LIB"

MACOS_LIB="$BUILD_DIR/libliauth-macos.a"
lipo -create \
    "$BUILD_DIR/aarch64-apple-darwin/release/libliauth.a" \
    "$BUILD_DIR/x86_64-apple-darwin/release/libliauth.a" \
    -output "$MACOS_LIB"

xcodebuild -create-xcframework \
    -library "$BUILD_DIR/aarch64-apple-ios/release/libliauth.a" -headers "$HEADERS_DIR" \
    -library "$SIM_LIB" -headers "$HEADERS_DIR" \
    -library "$MACOS_LIB" -headers "$HEADERS_DIR" \
    -output "$OUT_DIR/LiAuthCore.xcframework"

echo "LiAuthCore.xcframework ready at $OUT_DIR"
