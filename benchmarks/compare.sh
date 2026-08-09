#!/bin/sh
set -eu

REPOSITORY=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
OUTPUT=${1:-"$REPOSITORY/target/search-comparison.md"}
RUNS=${RUNS:-10}
FIXTURE=$(mktemp -d "${TMPDIR:-/tmp}/rust-search-bench.XXXXXX")
trap 'rm -rf "$FIXTURE"' EXIT HUP INT TERM

if command -v fd >/dev/null 2>&1; then
    FD=fd
elif command -v fdfind >/dev/null 2>&1; then
    FD=fdfind
else
    echo "fd (or fdfind) is required" >&2
    exit 1
fi

for tool in cargo hyperfine python3 rg find; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required" >&2
        exit 1
    fi
done

python3 "$REPOSITORY/benchmarks/generate_fixture.py" "$FIXTURE"
cargo build --manifest-path "$REPOSITORY/Cargo.toml" --release --example search
RUST_SEARCH="$REPOSITORY/target/release/examples/search"

rust_results="$FIXTURE/.rust-search-results"
fd_results="$FIXTURE/.fd-results"
rg_results="$FIXTURE/.ripgrep-results"
find_results="$FIXTURE/.find-results"

"$RUST_SEARCH" "$FIXTURE" rs | LC_ALL=C sort > "$rust_results"
"$FD" --type f --extension rs . "$FIXTURE" | LC_ALL=C sort > "$fd_results"
rg --files --glob '*.rs' "$FIXTURE" | LC_ALL=C sort > "$rg_results"
find "$FIXTURE" -type f -name '*.rs' | LC_ALL=C sort > "$find_results"

rust_count=$(wc -l < "$rust_results" | tr -d ' ')
fd_count=$(wc -l < "$fd_results" | tr -d ' ')
rg_count=$(wc -l < "$rg_results" | tr -d ' ')
find_count=$(wc -l < "$find_results" | tr -d ' ')

if [ "$rust_count" != "$fd_count" ] || [ "$rust_count" != "$rg_count" ] || [ "$rust_count" != "$find_count" ]; then
    echo "result counts differ: rust_search=$rust_count fd=$fd_count rg=$rg_count find=$find_count" >&2
    exit 1
fi

for candidate in "$fd_results" "$rg_results" "$find_results"; do
    if ! cmp -s "$rust_results" "$candidate"; then
        echo "result paths differ between rust_search and $candidate" >&2
        diff -u "$rust_results" "$candidate" | sed -n '1,80p' >&2
        exit 1
    fi
done

mkdir -p "$(dirname -- "$OUTPUT")"
hyperfine --warmup 3 --runs "$RUNS" --export-markdown "$OUTPUT" \
    --command-name rust_search "'$RUST_SEARCH' '$FIXTURE' rs > /dev/null" \
    --command-name fd "'$FD' --type f --extension rs . '$FIXTURE' > /dev/null" \
    --command-name 'ripgrep --files' "rg --files --glob '*.rs' '$FIXTURE' > /dev/null" \
    --command-name find "find '$FIXTURE' -type f -name '*.rs' > /dev/null"

echo "verified an identical set of $rust_count paths for every tool"
echo "wrote $OUTPUT"
