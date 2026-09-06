#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly repo_root
readonly prefix="$repo_root/target/local"
cd "$repo_root"

for command_name in cargo msgfmt; do
	if ! command -v "$command_name" >/dev/null 2>&1; then
		printf 'Missing required command: %s\n' "$command_name" >&2
		exit 127
	fi
done

cargo build --release --workspace --locked --target-dir "$repo_root/target"
install -d "$prefix/bin" "$prefix/share"
cp -a biglinux-webapps/usr/share/. "$prefix/share/"

for catalog in po/*.po; do
	language="${catalog##*/}"
	language="${language%.po}"
	locale_dir="$prefix/share/locale/${language//-/_}/LC_MESSAGES"
	install -d "$locale_dir"
	msgfmt --check "$catalog" -o "$locale_dir/biglinux-webapps.mo"
done

for binary in big-webapps-gui big-webapps-exec big-webapps-viewer; do
	install -m755 "target/release/$binary" "$prefix/bin/$binary"
done

printf 'Run: %s/bin/big-webapps-gui\n' "$prefix"
