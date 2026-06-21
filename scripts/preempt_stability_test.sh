#!/usr/bin/env bash
# Runs smoke_test.sh repeatedly to catch nondeterministic scheduler/preempt bugs.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

RUNS="${PREEMPT_STABILITY_RUNS:-20}"

pass=0
fail=0

echo "==> Preempt stability test (${RUNS} runs)..."

for i in $(seq 1 "$RUNS"); do
    if ./scripts/smoke_test.sh >/tmp/flynn_preempt_stability.log 2>&1; then
        pass=$((pass + 1))
        echo "  run ${i}/${RUNS}: PASS"
    else
        fail=$((fail + 1))
        echo "  run ${i}/${RUNS}: FAIL"
        tail -15 /tmp/flynn_preempt_stability.log
        echo ""
        echo "==> Stopping on first failure (set PREEMPT_STABILITY_RUNS to retry all)."
        exit 1
    fi
done

echo "==> Preempt stability test passed (${pass}/${RUNS})"
