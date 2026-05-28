#!/usr/bin/env bash
# Regenerate po/biglinux-webapps.pot from the Rust sources and merge the
# updated template into every per-language .po. Run after adding, removing,
# or modifying gettext() strings in the codebase.
#
# Uses `xtr` (a Rust-aware gettext extractor) instead of `xgettext --language=C`.
# xtr understands Rust syntax, so lifetime apostrophes ('a) and multi-byte
# literals no longer produce spurious warnings. It follows `mod` declarations
# from each entry point, so passing the three crate roots covers the whole
# workspace.
#
# Install with: cargo install xtr
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly repo_root
cd "$repo_root"

readonly pot='po/biglinux-webapps.pot'

for command_name in xtr msgmerge; do
	if ! command -v "$command_name" >/dev/null 2>&1; then
		printf 'Missing required command: %s\n' "$command_name" >&2
		printf 'Install xtr with: cargo install xtr\n' >&2
		exit 127
	fi
done

# Crate entry points. xtr recurses through `mod` declarations, so these three
# files cover every .rs file that could call gettext().
readonly -a entry_points=(
	'crates/webapps-manager/src/lib.rs'
	'crates/webapps-viewer/src/main.rs'
	'crates/webapps-core/src/lib.rs'
)

# POTFILES is kept in sync with what actually contains gettext() calls, for
# tools and translators that rely on it. Ordering is stable (sorted).
find crates -name '*.rs' -print0 \
	| xargs -0 grep -l 'gettext(' 2>/dev/null \
	| sort >po/POTFILES

xtr \
	--package-name='biglinux-webapps' \
	--package-version='4.0.0' \
	--msgid-bugs-address='https://github.com/biglinux/biglinux-webapps/issues' \
	--copyright-holder='BigLinux' \
	--output="$pot" \
	"${entry_points[@]}"

# Merge the updated template into each language. --previous preserves the
# prior msgid so translators see what changed when a string is marked fuzzy.
# Force C.UTF-8 so msgmerge treats the input as UTF-8 regardless of the
# caller's locale (some locales mis-decode em-dash / ellipsis as multi-byte).
for po in po/*.po; do
	LC_ALL=C.UTF-8 msgmerge --quiet --update --backup=none --previous "$po" "$pot"
done

printf 'Updated %s and %d .po files.\n' "$pot" "$(find po -name '*.po' | wc -l)"
