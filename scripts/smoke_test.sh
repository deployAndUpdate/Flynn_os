#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

TIMEOUT_SECS="${SMOKE_TIMEOUT_SECS:-90}"

echo "==> Building boot image..."
cargo build --quiet

BIOS="$ROOT/target/flynn_os/bios.img"
if [[ ! -f "$BIOS" ]]; then
    echo "error: boot image not found at $BIOS"
    exit 1
fi

if ! command -v qemu-system-x86_64 >/dev/null 2>&1; then
    echo "error: qemu-system-x86_64 is not installed"
    exit 1
fi

LOG="$(mktemp)"
trap 'rm -f "$LOG"' EXIT

echo "==> Running QEMU smoke test (timeout ${TIMEOUT_SECS}s)..."
set +e
timeout "${TIMEOUT_SECS}s" qemu-system-x86_64 \
    -m 512M \
    -serial stdio \
    -display none \
    -no-reboot \
    -drive "format=raw,file=$BIOS" \
    >"$LOG" 2>&1
qemu_status=$?
set -e

if [[ $qemu_status -ne 124 ]]; then
    echo "error: QEMU exited with status $qemu_status (expected timeout 124)"
    tail -50 "$LOG"
    exit 1
fi

MARKERS=(
    "logger initialized"
    "frame self-test ok"
    "KERNEL STATUS : ONLINE"
    "A:done"
    "B:done"
    "> "
)

failed=0
for marker in "${MARKERS[@]}"; do
    if grep -qF "$marker" "$LOG"; then
        echo "  OK: $marker"
    else
        echo "  MISSING: $marker"
        failed=1
    fi
done

if grep -qF "PANIC:" "$LOG"; then
    echo "  FAIL: kernel panic detected"
    failed=1
fi

if [[ $failed -ne 0 ]]; then
    echo ""
    echo "==> Boot log (last 80 lines):"
    tail -80 "$LOG"
    exit 1
fi

echo "==> Smoke test passed"
