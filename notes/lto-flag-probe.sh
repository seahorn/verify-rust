#!/usr/bin/env bash
# Probe: is `-Zemit-thin-lto=no` still needed to get FAT bitcode alongside
# `-Clinker-plugin-lto`, or does `-Clto=fat` now suffice (rust-lang/rust#136840)?
#
# Why this matters here: cmake/SeaHornBuild.cmake links with
#   -Wl,--plugin-opt=-lto-embed-bitcode=post-merge-pre-opt
# and SeaHorn verifies that embedded post-merge module. It needs ONE fat
# whole-program module. Thin-LTO-prepared bitcode carries a module summary and
# takes lld's ThinLTO path instead.
#
# Detection: ThinLTO-prepared bitcode contains a GLOBALVAL_SUMMARY_BLOCK.
#   present -> thin      absent -> fat
#
# Usage: ./lto-flag-probe.sh <toolchain> [<toolchain> ...]
#   e.g. ./lto-flag-probe.sh nightly-2024-07-01 nightly

set -uo pipefail

BCANALYZER=${BCANALYZER:-llvm-bcanalyzer-18}
command -v "$BCANALYZER" >/dev/null || { echo "need $BCANALYZER (apt install llvm-18)"; exit 1; }

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
cat > "$WORK/lib.rs" <<'EOF'
#![crate_type = "staticlib"]
#[no_mangle]
pub extern "C" fn probe(x: i32) -> i32 { x + 1 }
EOF

# Classify the bitcode inside a rustc-produced staticlib.
# NOTE: a rustc staticlib bundles hundreds of NATIVE .o members (std,
# compiler_builtins) even under -Clinker-plugin-lto; only the crate's own cgu
# objects become bitcode. So: inspect EVERY member, skip non-bitcode, and judge
# on the bitcode ones. Looking at just the first .o tells you nothing.
# The extraction dir must be unique per archive -- sharing one dir across probes
# lets members from a previous probe be counted against this one.
classify() {
    local ar="$1" d="$WORK/x$((CLASSIFY_N++))"
    rm -rf "$d"; mkdir -p "$d"; ( cd "$d" && ar x "$ar" 2>/dev/null )

    local found_bc=0 saw_summary=0
    for o in "$d"/*.o; do
        [ -e "$o" ] || continue
        # rustc with -Clinker-plugin-lto emits bitcode where .o would be.
        if head -c4 "$o" | grep -q 'BC'; then
            found_bc=1
            if "$BCANALYZER" "$o" 2>/dev/null | grep -q "GLOBALVAL_SUMMARY_BLOCK"; then
                saw_summary=1
            fi
        fi
    done

    if [ "$found_bc" = 0 ]; then echo "NATIVE (no bitcode -> linker-plugin-lto not in effect)"
    elif [ "$saw_summary" = 1 ]; then echo "THIN  (module summary present)"
    else echo "FAT   (no module summary)"
    fi
}
CLASSIFY_N=0

probe_one() {
    local tc="$1" label="$2"; shift 2
    # NOTE: the output path must be unique per (toolchain, flags) AND removed
    # first. Reusing one path across toolchains lets a stale artifact from an
    # earlier toolchain be measured when a later rustc rejects the flags --
    # which silently reports a flag as working when it has actually been removed.
    local n=$((PROBE_N++))
    local out="$WORK/probe${n}.a"
    rm -f "$out"
    local err
    err=$(rustc "+$tc" --edition 2021 --crate-name p "$@" -o "$out" "$WORK/lib.rs" 2>&1)
    if [ ! -f "$out" ]; then
        printf '  %-40s ERROR: %s\n' "$label" "$(echo "$err" | grep -E '^error' | head -1)"
        return
    fi
    printf '  %-40s %s\n' "$label" "$(classify "$out")"
}
PROBE_N=0

for tc in "$@"; do
    echo "=================================================================="
    echo "toolchain: $tc"
    rustc "+$tc" --version --verbose 2>/dev/null | grep -E "^(release|LLVM version)" | sed 's/^/  /'
    echo "  -Z emit-thin-lto listed: $(rustc "+$tc" -Z help 2>/dev/null | grep -ci 'emit-thin-lto')"
    echo "------------------------------------------------------------------"
    probe_one "$tc" "baseline (-O only)"                    -O
    probe_one "$tc" "-Clinker-plugin-lto"                   -O -Clinker-plugin-lto
    probe_one "$tc" "-Clinker-plugin-lto -Zemit-thin-lto=no" -O -Clinker-plugin-lto -Zemit-thin-lto=no
    probe_one "$tc" "-Clinker-plugin-lto -Clto=fat"          -O -Clinker-plugin-lto -Clto=fat
    probe_one "$tc" "-Clinker-plugin-lto -Clto=thin"         -O -Clinker-plugin-lto -Clto=thin
    echo
done

cat <<'EOF'
Reading the result
------------------
The question is the `-Clinker-plugin-lto -Clto=fat` row:

  FAT  -> -Clto=fat alone is enough on that toolchain; -Zemit-thin-lto=no is redundant.
  THIN -> -Clto=fat is NOT enough there; -Zemit-thin-lto=no is still load-bearing.

Compare across a pre-#136840 and a post-#136840 toolchain to see the change.
EOF
