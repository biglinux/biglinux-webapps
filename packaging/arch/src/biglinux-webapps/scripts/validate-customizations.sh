#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly repo_root
cd "$repo_root"

require_command() {
	local command_name="$1"
	if ! command -v "$command_name" >/dev/null 2>&1; then
		printf 'Missing required command: %s\n' "$command_name" >&2
		exit 127
	fi
}

readonly shell_targets=(
	"biglinux-webapps/usr/bin/biglinux-webapps-systemd"
	"packaging/arch/PKGBUILD"
)

for command_name in cargo shellcheck shfmt bash; do
	require_command "$command_name"
done

cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
shellcheck "${shell_targets[@]}"
shfmt -d "${shell_targets[@]}"
bash -n "${shell_targets[@]}"
