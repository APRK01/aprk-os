# =============================================================================
# APRK OS Makefile
# =============================================================================
# Build automation for APRK OS
# Run 'make help' to see available commands
# =============================================================================

# Default target
.DEFAULT_GOAL := build

# Project settings
KERNEL_BIN = target/aarch64-unknown-none/debug/aprk-kernel
KERNEL_BIN_RELEASE = target/aarch64-unknown-none/release/aprk-kernel

# Colors for output
GREEN = \033[0;32m
YELLOW = \033[0;33m
RED = \033[0;31m
NC = \033[0m # No Color

# Disk Configuration
DISK_DIR = disk_root
DISK_IMG = disk.img
DISK_DMG = disk.dmg

# =============================================================================
# Main Targets
# =============================================================================

.PHONY: all
all: build ## Build everything

# User Programs
USER_BIN_DIR = target/aarch64-unknown-none/release

.PHONY: user
user: ## Build user programs
	@echo "$(GREEN)[USER]$(NC) Building Userland..."
	RUSTFLAGS="-C link-arg=-Ttext=0x40200000 -C link-arg=-zmax-page-size=4096" cargo build -p hello --release --target aarch64-unknown-none
	@mkdir -p $(DISK_DIR)
	@cp $(USER_BIN_DIR)/hello $(DISK_DIR)/hello

.PHONY: disk
disk: user ## Create FAT32 disk image
	@echo "$(GREEN)[DISK]$(NC) Creating FAT32 disk image..."
	@mkdir -p $(DISK_DIR)
	@if [ ! -f $(DISK_DIR)/hello.txt ]; then \
		echo "Hello from APRK OS FAT32 Filesystem!" > $(DISK_DIR)/hello.txt; \
		echo "APRK OS v0.0.1" > $(DISK_DIR)/version; \
	fi
	@# Create FAT32 image using hdiutil on macOS
	@./scripts/make-disk.sh

.PHONY: build
build: disk ## Build the kernel (debug mode)
	@echo "$(GREEN)[BUILD]$(NC) Building APRK OS kernel (debug)..."
	cargo build
	@echo "$(GREEN)[BUILD]$(NC) Done! Kernel at $(KERNEL_BIN)"

.PHONY: release
release: ## Build the kernel (release mode)
	@echo "$(GREEN)[BUILD]$(NC) Building APRK OS kernel (release)..."
	cargo build --release
	@echo "$(GREEN)[BUILD]$(NC) Done! Kernel at $(KERNEL_BIN_RELEASE)"

.PHONY: run
run: build ## Build and run on QEMU
	@echo "$(GREEN)[RUN]$(NC) Starting QEMU..."
	./scripts/qemu-run.sh $(KERNEL_BIN)

.PHONY: run-release
run-release: release ## Build release and run on QEMU
	@echo "$(GREEN)[RUN]$(NC) Starting QEMU (release build)..."
	./scripts/qemu-run.sh $(KERNEL_BIN_RELEASE)

.PHONY: clean
clean: ## Clean build artifacts
	@echo "$(YELLOW)[CLEAN]$(NC) Removing build artifacts..."
	cargo clean
	@echo "$(YELLOW)[CLEAN]$(NC) Done!"

# =============================================================================
# Development Targets
# =============================================================================

.PHONY: check
check: ## Check code without building
	cargo check

.PHONY: clippy
clippy: ## Run clippy linter
	cargo clippy -- -D warnings

.PHONY: fmt
fmt: ## Format code
	cargo fmt

.PHONY: fmt-check
fmt-check: ## Check code formatting
	cargo fmt --check

.PHONY: test
test: ## Run tests (host machine tests only)
	cargo test --target=aarch64-apple-darwin

# =============================================================================
# Debug Targets
# =============================================================================

.PHONY: debug
debug: build ## Run with GDB debugging enabled
	@echo "$(GREEN)[DEBUG]$(NC) Starting QEMU with GDB server..."
	@echo "$(YELLOW)Connect GDB with: aarch64-none-elf-gdb -ex 'target remote :1234'$(NC)"
	qemu-system-aarch64 \
		-machine virt \
		-cpu cortex-a72 \
		-m 512M \
		-nographic \
		-kernel $(KERNEL_BIN) \
		-serial mon:stdio \
		-S -gdb tcp::1234

.PHONY: objdump
objdump: build ## Disassemble the kernel
	llvm-objdump -d $(KERNEL_BIN) | less

.PHONY: readelf
readelf: build ## Show ELF information
	llvm-readelf -a $(KERNEL_BIN) | less

.PHONY: size
size: build ## Show kernel size information
	@echo "$(GREEN)[SIZE]$(NC) Kernel binary size:"
	@ls -lh $(KERNEL_BIN)
	@echo ""
	@llvm-size $(KERNEL_BIN)

# =============================================================================
# Documentation Targets
# =============================================================================

.PHONY: doc
doc: ## Generate documentation
	cargo doc --no-deps --document-private-items
	@echo "$(GREEN)[DOC]$(NC) Documentation at target/aarch64-unknown-none/doc/"

.PHONY: doc-open
doc-open: doc ## Generate and open documentation
	open target/aarch64-unknown-none/doc/aprk_kernel/index.html

# =============================================================================
# Help
# =============================================================================

.PHONY: help
help: ## Show this help
	@echo "APRK OS Build System"
	@echo "===================="
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-15s$(NC) %s\n", $$1, $$2}'
	@echo ""
