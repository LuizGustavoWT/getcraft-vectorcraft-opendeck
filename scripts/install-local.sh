#!/usr/bin/env bash
set -euo pipefail

readonly plugin_id="wt.luizgustavo.getcraft.vectorcraft.sdPlugin"
readonly binary_name="getcraft-vectorcraft-opendeck"
readonly target="x86_64-unknown-linux-gnu"
readonly script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly repo_root="$(cd -- "$script_dir/.." && pwd)"
readonly config_home="${XDG_CONFIG_HOME:-$HOME/.config}"
readonly plugins_dir="${OPENDECK_PLUGINS_DIR:-$config_home/opendeck/plugins}"
readonly destination="$plugins_dir/$plugin_id"
readonly package_dir="$repo_root/dist/$plugin_id"

if [[ -z "$plugins_dir" || "$plugins_dir" == "/" || "$destination" == "$plugins_dir" || "$destination" != *.sdPlugin ]]; then
  echo "Refusing unsafe OpenDeck plugin destination: $destination" >&2
  exit 2
fi

cd "$repo_root"
bash scripts/package.sh

if [[ ! -f "$package_dir/manifest.json" || ! -f "$package_dir/$target/bin/$binary_name" ]]; then
  echo "The package is incomplete: $package_dir" >&2
  exit 3
fi

mkdir -p "$plugins_dir"
staging_dir="$(mktemp -d "$plugins_dir/.${plugin_id}.staging.XXXXXX")"

cleanup() {
  if [[ -n "${staging_dir:-}" && -d "$staging_dir" ]]; then
    rm -rf -- "$staging_dir"
  fi
}
trap cleanup EXIT

cp -a "$package_dir/." "$staging_dir/"

backup_dir=""
if [[ -e "$destination" ]]; then
  backup_dir="$(mktemp -d "$plugins_dir/.${plugin_id}.backup.XXXXXX")"
  mv -- "$destination" "$backup_dir/"
fi

mv -- "$staging_dir" "$destination"
staging_dir=""
trap - EXIT

echo "Installed $plugin_id in $destination"
if [[ -n "$backup_dir" ]]; then
  echo "Previous installation preserved in $backup_dir"
fi
echo "This installer never restarts or reloads OpenDeck."
echo "When convenient, use OpenDeck's plugin reload control or restart it manually."
