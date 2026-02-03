CONTRACT := contract
WEB_DRIVER := web/public/data_driver.wasm
DD_WASM := $(shell $(MAKE) -s -C $(CONTRACT) echo-dd)

all: ## Build contract WASM + data-driver WASM
	$(MAKE) -C $(CONTRACT) wasm-opt
	$(MAKE) data-driver

help: ## Display this help screen
	@grep -h \
		-E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

wasm: ## Build the DRC20 contract WASM
	$(MAKE) -C $(CONTRACT) wasm

wasm-opt: ## Build + optimize the DRC20 contract WASM
	$(MAKE) -C $(CONTRACT) wasm-opt

data-driver: ## Build the data-driver WASM (alloc enabled) and copy into web/public
	$(MAKE) -C $(CONTRACT) wasm-dd DD_FEATURE=data-driver
	@cp "$(DD_WASM)" "$(WEB_DRIVER)"
	@echo "Copied $(DD_WASM) -> $(WEB_DRIVER)"

test-caller-wasm: ## Build the helper caller contract used by the test-suite
	$(MAKE) -C tests/caller wasm

test: ## Build wasm artifacts and run the DRC20 spec tests
	$(MAKE) wasm
	$(MAKE) test-caller-wasm
	cargo test -p drc20-tests

init-args: ## Encode Init JSON to rkyv hex (for deployment constructor args). Usage: make init-args FILE=./init.json (default: ./init.json)
	@set -e; \
	FILE="${FILE:-./init.json}"; \
	if [ -f "$$FILE" ]; then \
	  cargo run -p drc20-tools --quiet -- init-args --file "$$FILE"; \
	  echo; \
	else \
	  echo "Missing $$FILE"; \
	  echo "Create it (it's git-ignored) or copy the example:"; \
	  echo "  cp example.init.json init.json"; \
	  exit 2; \
	fi

.PHONY: all help wasm wasm-opt data-driver init-args test-caller-wasm test
