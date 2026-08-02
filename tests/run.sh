#!/usr/bin/env bash
#
# End-to-end tests for the .sc programs in this directory.
#
# Each program declares its own expectations as `//@` directives:
#
#   //@ run                 compile, link and execute the program
#   //@ exit: 42            required with `run`: expected process exit code
#   //@ stdout: text        substring the program must print (repeatable)
#   //@ compile-fail        scrapc must reject the program
#   //@ error: text         substring scrapc must report (repeatable)
#   //@ manifest: Scrap.toml  passed to --manifest; marks the case as interop
#   //@ ignore: reason      skip this program
#
# `run` and `compile-fail` are mutually exclusive. A program with no directives
# is an error, not a skip, so nothing silently drops out of coverage.
#
# Usage: tests/run.sh [--interop] [name-filter]

set -uo pipefail

cd "$(dirname "$0")/.." || exit 1
ROOT="$PWD"

case "$(uname -s)" in
    MINGW* | MSYS* | CYGWIN*) EXE=".exe" ;;
    *) EXE="" ;;
esac

RUN_INTEROP=0
FILTER=""
for arg in "$@"; do
    case "$arg" in
        --interop) RUN_INTEROP=1 ;;
        -h | --help)
            sed -n '3,20p' "$0" | sed 's|^# \?||'
            exit 0
            ;;
        *) FILTER="$arg" ;;
    esac
done

if [[ -t 1 ]]; then
    RED=$'\033[31m'; GREEN=$'\033[32m'; YELLOW=$'\033[33m'; DIM=$'\033[2m'; OFF=$'\033[0m'
else
    RED=""; GREEN=""; YELLOW=""; DIM=""; OFF=""
fi

# scrap_rt is a staticlib that `cargo check` and `cargo test` never produce. Without
# it every program dies at link with `undefined symbol: __scrap_gc_init`, so build it
# explicitly rather than assuming a previous build left it behind.
echo "building scrapc and scrap_rt..."
# The interop cases need the scrap-rustc metadata driver, which is an optional
# artifact dependency behind `interop-driver` — off by default so an ordinary
# build does not require the rustc-dev component.
build_features=()
if ((RUN_INTEROP)); then
    build_features=(--features interop-driver)
fi
cargo build -p scrapc -p scrap_rt "${build_features[@]}" || exit 1

SCRAPC="$ROOT/target/debug/scrapc$EXE"
if [[ ! -x "$SCRAPC" ]]; then
    echo "${RED}error${OFF}: $SCRAPC not found after build"
    exit 1
fi
if [[ ! -f "$ROOT/target/debug/scrap_rt.lib" && ! -f "$ROOT/target/debug/libscrap_rt.a" ]]; then
    echo "${RED}error${OFF}: scrap_rt staticlib not found in target/debug"
    exit 1
fi
echo

passed=0
failed=0
skipped=0
failures=()

report_fail() {
    local name="$1" reason="$2" detail="${3:-}"
    echo "${RED}FAIL${OFF} $name — $reason"
    if [[ -n "$detail" ]]; then
        echo "$detail" | sed "s|^|     ${DIM}|; s|\$|${OFF}|"
    fi
    failed=$((failed + 1))
    failures+=("$name")
}

while IFS= read -r file; do
    # Crate name doubles as the output filename in target/scrap, and scrapc has no
    # --out-dir. Derive it from the path so tests/methods.sc and
    # tests/rust_interop_abi/methods.sc do not overwrite each other.
    name="${file#tests/}"
    name="${name%.sc}"
    name="${name//\//__}"

    if [[ -n "$FILTER" && "$name" != *"$FILTER"* ]]; then
        continue
    fi

    mode=""
    want_exit=""
    manifest=""
    ignore_reason=""
    want_stdout=()
    want_errors=()
    unknown=""
    count=0

    while IFS= read -r directive; do
        count=$((count + 1))
        value="${directive#*:}"
        value="${value#"${value%%[![:space:]]*}"}"
        case "$directive" in
            run) mode="run" ;;
            compile-fail) mode="compile-fail" ;;
            exit:*) want_exit="$value" ;;
            stdout:*) want_stdout+=("$value") ;;
            error:*) want_errors+=("$value") ;;
            manifest:*) manifest="$value" ;;
            ignore:*) ignore_reason="$value" ;;
            *) unknown="$directive" ;;
        esac
    done < <(sed -n 's|^//@ *||p' "$file")

    if ((count == 0)); then
        report_fail "$name" "no //@ directives (see tests/run.sh for the format)"
        continue
    fi
    if [[ -n "$unknown" ]]; then
        report_fail "$name" "unknown directive: //@ $unknown"
        continue
    fi
    if [[ -n "$ignore_reason" ]]; then
        echo "${YELLOW}SKIP${OFF} $name — $ignore_reason"
        skipped=$((skipped + 1))
        continue
    fi
    if [[ -n "$manifest" && $RUN_INTEROP -eq 0 ]]; then
        skipped=$((skipped + 1))
        continue
    fi
    if [[ -z "$mode" ]]; then
        report_fail "$name" "needs either //@ run or //@ compile-fail"
        continue
    fi
    if [[ "$mode" == "run" && -z "$want_exit" ]]; then
        report_fail "$name" "//@ run needs an //@ exit: directive"
        continue
    fi

    args=(--crate-name "$name" --crate-type bin)
    if [[ -n "$manifest" ]]; then
        args+=(--manifest "$(dirname "$file")/$manifest")
    fi
    args+=("$file")

    compile_out=$("$SCRAPC" "${args[@]}" 2>&1)
    compile_rc=$?

    if [[ "$mode" == "compile-fail" ]]; then
        if ((compile_rc == 0)); then
            report_fail "$name" "expected compilation to fail, but it succeeded"
            continue
        fi
        missing=""
        for want in ${want_errors[@]+"${want_errors[@]}"}; do
            if [[ "$compile_out" != *"$want"* ]]; then
                missing="$want"
                break
            fi
        done
        if [[ -n "$missing" ]]; then
            report_fail "$name" "stderr missing expected text: $missing" "$compile_out"
            continue
        fi
        echo "${GREEN}ok${OFF}   $name ${DIM}(compile-fail)${OFF}"
        passed=$((passed + 1))
        continue
    fi

    if ((compile_rc != 0)); then
        report_fail "$name" "compilation failed (exit $compile_rc)" "$compile_out"
        continue
    fi

    binary="$ROOT/target/scrap/$name$EXE"
    if [[ ! -x "$binary" ]]; then
        report_fail "$name" "compiled but produced no binary at $binary"
        continue
    fi

    run_out=$("$binary" 2>&1)
    run_rc=$?

    if ((run_rc != want_exit)); then
        report_fail "$name" "exit code $run_rc, expected $want_exit" "$run_out"
        continue
    fi
    missing=""
    for want in ${want_stdout[@]+"${want_stdout[@]}"}; do
        if [[ "$run_out" != *"$want"* ]]; then
            missing="$want"
            break
        fi
    done
    if [[ -n "$missing" ]]; then
        report_fail "$name" "output missing expected text: $missing" "$run_out"
        continue
    fi

    echo "${GREEN}ok${OFF}   $name ${DIM}(exit $run_rc)${OFF}"
    passed=$((passed + 1))
done < <(find tests -name '*.sc' | sort)

echo
if ((failed > 0)); then
    echo "${RED}$failed failed${OFF}, $passed passed, $skipped skipped"
    printf '  %s\n' "${failures[@]}"
    exit 1
fi
echo "${GREEN}$passed passed${OFF}, $skipped skipped"
