#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)"
DATA_DIR="$ROOT_DIR/data"
OUTPUT_GZ="$DATA_DIR/diffs.json.gz"
OUTPUT_JSON="$DATA_DIR/diffs.json"

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/mdd-diffs.XXXXXX")"
trap 'rm -rf "$TMP_DIR"' EXIT

WORK_OUTPUT="$TMP_DIR/diffs.json.gz"
MDD_BIN="$ROOT_DIR/target/debug/mdd"

cargo build --quiet --manifest-path "$ROOT_DIR/Cargo.toml" --bin mdd

sorted_taxonomy_files="$(
    printf '%s\n' "$DATA_DIR"/Diff_v*.csv |
        LC_ALL=C sort
)"

if [[ -z "$sorted_taxonomy_files" || ! -f "${sorted_taxonomy_files%%$'\n'*}" ]]; then
    echo "No taxonomy diff files found in $DATA_DIR" >&2
    exit 1
fi

processed=0
while IFS= read -r taxonomy_path; do
    [[ -n "$taxonomy_path" ]] || continue
    [[ -f "$taxonomy_path" ]] || continue

    taxonomy_name="${taxonomy_path##*/}"
    all_changes_name="${taxonomy_name/Diff_v/Diff-AllChanges_v}"
    all_changes_path="$DATA_DIR/$all_changes_name"

    if [[ ! -f "$all_changes_path" ]]; then
        all_changes_name="${all_changes_name/-v/-}"
        all_changes_path="$DATA_DIR/$all_changes_name"
    fi

    if [[ ! -f "$all_changes_path" ]]; then
        echo "Missing all-changes diff for $taxonomy_name: $all_changes_path" >&2
        exit 1
    fi

    if [[ -f "$WORK_OUTPUT" ]]; then
        "$MDD_BIN" diff \
            --input "$taxonomy_path" \
            --all-changes "$all_changes_path" \
            --output "$WORK_OUTPUT" \
            --plain-text \
            --append "$WORK_OUTPUT"
    else
        "$MDD_BIN" diff \
            --input "$taxonomy_path" \
            --all-changes "$all_changes_path" \
            --output "$WORK_OUTPUT" \
            --plain-text
    fi

    processed=$((processed + 1))
done <<< "$sorted_taxonomy_files"

if (( processed == 0 )); then
    echo "No taxonomy diff files found in $DATA_DIR" >&2
    exit 1
fi

mv "$TMP_DIR/diffs.json.gz" "$OUTPUT_GZ"
mv "$TMP_DIR/diffs.json" "$OUTPUT_JSON"

echo "Wrote $processed diff releases to:"
echo "  $OUTPUT_GZ"
echo "  $OUTPUT_JSON"
