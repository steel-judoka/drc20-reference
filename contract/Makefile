CRATE_NAME_DASHED := $(shell cargo metadata --no-deps --format-version 1 --manifest-path Cargo.toml | jq -r '.packages[] | select(.manifest_path == "$(realpath Cargo.toml)") | .name')
CRATE_NAME := $(subst -,_,$(CRATE_NAME_DASHED))

# Detect workspace root so we know where target/ actually lives
WORKSPACE_ROOT := $(shell cargo metadata --no-deps --format-version 1 --manifest-path Cargo.toml | jq -r '.workspace_root')

WASM_OPT ?= $(shell command -v wasm-opt 2>/dev/null)
WASM_TARGET = wasm32-unknown-unknown
WASM_RELEASE_DIR = $(WORKSPACE_ROOT)/target/$(WASM_TARGET)/release
WASM_FILE = $(WASM_RELEASE_DIR)/$(CRATE_NAME).wasm
WASM_OPT_FILE = $(WASM_RELEASE_DIR)/$(CRATE_NAME)_opt.wasm

CONTRACT_FEATURE ?= contract
DD_FEATURE ?= data-driver
DD_TARGET_DIR := $(WORKSPACE_ROOT)/target/data-driver
DD_WASM_RELEASE_DIR = $(DD_TARGET_DIR)/$(WASM_TARGET)/release
DD_WASM_FILE = $(DD_WASM_RELEASE_DIR)/$(CRATE_NAME).wasm
DD_WASM_OPT_FILE = $(DD_WASM_RELEASE_DIR)/$(CRATE_NAME)_opt.wasm

all: wasm

wasm: ## Generate WASM
	@echo "Building WASM for crate: $(CRATE_NAME)"
	@RUSTFLAGS="$(RUSTFLAGS) --remap-path-prefix $(HOME)= -C link-args=-zstack-size=65536" \
	  cargo build -Z build-std=core,alloc \
				--release \
				--features $(CONTRACT_FEATURE) \
	    	--color=always \
				--target $(WASM_TARGET)
	@echo "WASM build complete:"
	@$(MAKE) echo

wasm-opt: $(WASM_FILE) ## Generate optimized WASM
	@if [ -z "$(WASM_OPT)" ]; then \
		echo "wasm-opt not found. Please install it to proceed."; \
		echo "You can install wasm-opt by installing the 'binaryen' package from https://github.com/WebAssembly/binaryen"; \
		exit 1; \
	fi
	@echo "Optimizing WASM for crate: $(CRATE_NAME)"
	@wasm-opt -Oz --strip-debug -o $(WASM_OPT_FILE) $(WASM_FILE)
	@echo "Optimized WASM build complete:"
	@$(MAKE) echo-opt

wasm-dd: ## Build data-driver WASM (alloc enabled)
	@if [ -z "$(WASM_OPT)" ]; then \
		echo "wasm-opt not found. Please install it to proceed."; \
		echo "You can install wasm-opt by installing the 'binaryen' package from https://github.com/WebAssembly/binaryen"; \
		exit 1; \
	fi
	@echo "Building data-driver WASM for crate: $(CRATE_NAME)"
	@CARGO_TARGET_DIR=$(DD_TARGET_DIR) \
	  RUSTFLAGS="$(RUSTFLAGS) --remap-path-prefix $(HOME)=" \
	  cargo build \
				--release \
				--features $(DD_FEATURE) \
	    	--color=always \
				--target $(WASM_TARGET)
	@echo "Optimizing data-driver WASM for crate: $(CRATE_NAME)"
	@wasm-opt -Oz --strip-debug -o $(DD_WASM_OPT_FILE) $(DD_WASM_FILE)
	@echo "Data-driver WASM build complete:"
	@$(MAKE) echo-dd

echo:
	@echo $(WASM_FILE)

echo-opt:
	@echo $(WASM_OPT_FILE)

echo-dd:
	@echo $(DD_WASM_OPT_FILE)

$(WASM_FILE): wasm ## Compile the WASM

$(WASM_OPT_FILE): wasm-opt ## Compile and optimize the WASM

$(DD_WASM_FILE): wasm-dd ## Compile the data-driver WASM

$(DD_WASM_OPT_FILE): wasm-dd ## Compile and optimize the data-driver WASM

clippy: ## Run clippy
	@cargo clippy -Z build-std=core,alloc \
		--release \
		--features $(CONTRACT_FEATURE) \
		--target $(WASM_TARGET) \
	  -- -D warnings

doc: ## Build the docs
	@cargo doc --release

.PHONY: all wasm wasm-opt wasm-dd echo echo-opt echo-dd clippy doc
