#!/bin/bash
# =============================================================================
# APRK OS - QEMU Run Script
# =============================================================================
# Runs APRK OS kernel on QEMU ARM64 virt machine.
# Usage: ./scripts/qemu-run.sh [kernel-binary]
# =============================================================================

set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Default kernel binary path
KERNEL="${1:-$PROJECT_ROOT/target/aarch64-unknown-none/debug/aprk-kernel}"

# QEMU binary
QEMU="qemu-system-aarch64"

# Check if QEMU is installed
if ! command -v $QEMU &> /dev/null; then
    echo "Error: $QEMU not found. Please install QEMU."
    echo "  macOS: brew install qemu"
    echo "  Debian/Ubuntu: sudo apt install qemu-system-arm"
    exit 1
fi

# Check if kernel binary exists
if [ ! -f "$KERNEL" ]; then
    echo "Error: Kernel binary not found at $KERNEL"
    echo "Please build the kernel first with: cargo build"
    exit 1
fi

echo "=============================================="
echo "  APRK OS - Starting QEMU"
echo "=============================================="
echo "Kernel: $KERNEL"
echo "Press Ctrl+A, X to exit QEMU"
echo "=============================================="
echo

# Run QEMU with the following configuration:
# -machine virt     : ARM virt machine (similar to real hardware)
# -cpu cortex-a72   : Cortex-A72 CPU (good ARM64 core)
# -m 512M           : 512MB RAM
# -nographic        : No graphical output, use serial console
# -kernel           : Load our kernel binary
# -serial mon:stdio : Connect serial port to terminal
$QEMU \
    -machine virt,gic-version=2 \
    -cpu cortex-a72 \
    -m 512M \
    -device virtio-gpu-device \
    -drive file=disk.img,if=none,format=raw,id=drive0 \
    -device virtio-blk-device,drive=drive0 \
    -kernel "$KERNEL" \
    -serial mon:stdio
