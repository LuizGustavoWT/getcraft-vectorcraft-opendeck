#!/usr/bin/env bash
set -euo pipefail

readonly plugin_id="wt.luizgustavo.getcraft.vectorcraft.sdPlugin"
readonly binary_name="getcraft-vectorcraft-opendeck"
readonly target="x86_64-unknown-linux-gnu"
readonly output_dir="dist"
readonly package_dir="$output_dir/$plugin_id"
readonly zip_name="getcraft-vectorcraft-opendeck-linux-x86_64.zip"

cargo build --release --target "$target"

rm -rf "$package_dir"
rm -f "$output_dir/$zip_name"
mkdir -p "$package_dir/$target/bin" "$package_dir/assets" "$package_dir/propertyInspector"
cp "assets/manifest.json" "$package_dir/manifest.json"
cp assets/*.svg assets/*.png "$package_dir/assets/"
cp propertyInspector/* "$package_dir/propertyInspector/"
cp "target/$target/release/$binary_name" "$package_dir/$target/bin/$binary_name"

(
  cd "$output_dir"
  zip -qr "$zip_name" "$plugin_id"
)

echo "Created $output_dir/$zip_name"
