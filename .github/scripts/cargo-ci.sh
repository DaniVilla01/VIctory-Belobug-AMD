#!/usr/bin/env bash
# Keep the full job log and surface a bounded diagnostic in the PR checks.
set -uo pipefail

log=$(mktemp) || exit 1
trap 'rm -f "$log"' EXIT
cargo "$@" 2>&1 | tee "$log"
status=("${PIPESTATUS[@]}")

if (( status[0] != 0 )); then
    # Include context after the first error (e.g. the Rust source span or
    # the reason Cargo could not fetch a dependency). Fall back to the tail.
    detail=$(awk '
        !first && /^[[:space:]]*(error(\[[^]]+\])?: |thread .* panicked at )/ { first = NR }
        first && NR < first + 16 { print substr($0, 1, 400) }
        first && NR >= first + 16 { exit }
    ' "$log")
    if [[ -z $detail ]]; then
        detail=$(tail -n 16 "$log" | cut -c 1-400)
    fi
    detail=${detail:0:3500}
    detail=${detail//%/%25}
    detail=${detail//$'\r'/%0D}
    detail=${detail//$'\n'/%0A}
    printf '\n::error title=Cargo failed (exit %d)::%s\n' "${status[0]}" "${detail:-No Cargo output}"
    exit "${status[0]}"
fi
exit "${status[1]}"
