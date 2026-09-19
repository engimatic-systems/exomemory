#!/usr/bin/env bash
set -u

script_dir="$(cd "$(dirname "$0")" && pwd)"
tools_dir="$script_dir/.tools"
results_dir="$script_dir/results"
tlc_dir="$script_dir/.tlc"
jar="$tools_dir/tla2tools.jar"
java_bin="${JAVA_BIN:-/opt/homebrew/opt/openjdk/bin/java}"

mkdir -p "$tools_dir" "$results_dir" "$tlc_dir"

if [[ ! -x "$java_bin" ]]; then
  java_bin="$(command -v java 2>/dev/null || true)"
fi

if [[ -z "$java_bin" || ! -x "$java_bin" ]]; then
  echo "Java 11 or newer is required. Set JAVA_BIN to its java executable." >&2
  exit 2
fi

if [[ ! -f "$jar" ]]; then
  echo "Downloading the official TLA+ tools v1.7.4..."
  curl -fL \
    https://github.com/tlaplus/tlaplus/releases/download/v1.7.4/tla2tools.jar \
    -o "$jar"
fi

failures=0

run_case() {
  local config="$1"
  local expected="$2"
  local expected_invariant="${3:-}"
  local module="${4:-Exomemory.tla}"
  local name="${config%.cfg}"
  local log="$results_dir/$name.log"
  local model_dir="$tlc_dir/$name"

  rm -rf "$model_dir"
  mkdir -p "$model_dir"

  echo "Checking $name (expected: $expected)..."
  (
    cd "$script_dir" || exit 1
    "$java_bin" -cp "$jar" tlc2.TLC \
      -workers 1 \
      -metadir "$model_dir" \
      -config "configs/$config" \
      "$module"
  ) >"$log" 2>&1
  local status=$?

  if [[ "$expected" == "counterexample" \
        && "$status" -ne 0 \
        && $(grep -c "Invariant $expected_invariant is violated" "$log") -gt 0 ]]; then
    echo "  PASS: TLC violated $expected_invariant as expected."
  elif [[ "$expected" == "safe" \
        && "$status" -eq 0 \
        && $(grep -c 'Model checking completed. No error has been found.' "$log") -gt 0 ]]; then
    echo "  PASS: exhaustive finite-state check found no violation."
  else
    echo "  FAIL: unexpected TLC result; inspect $log" >&2
    failures=$((failures + 1))
  fi
}

run_case Current_Create.cfg counterexample NoDanglingGhost
run_case Current_Overwrite.cfg counterexample GhostBodySameVersion
run_case Current_Dedup.cfg counterexample AtMostOneGhostPerWindow
run_case Current_Compaction.cfg counterexample NoStaleHistorySuppression
run_case Current_Budget.cfg counterexample WithinGhostBudget
run_case Current_Authority.cfg counterexample NoUntrustedDeveloperContext
run_case Current_CAS.cfg counterexample NoStaleWriterCommit
run_case Hardened.cfg safe
run_case Current_RandomSurfacing.cfg counterexample AlwaysSelectedTopRanked GhostSelection.tla
run_case Finalized_Selection.cfg safe "" GhostSelection.tla

if [[ "$failures" -ne 0 ]]; then
  exit 1
fi

echo "All model-check expectations matched. Detailed logs are in $results_dir."
