.PHONY: all clean dist frontend brain-mcp brain-app dev test

DIST := dist

all: dist

# Build everything and collect into dist/
dist: brain-mcp brain-app
	@echo "\n✅ Build complete. Output:"
	@ls -lh $(DIST)/brain-mcp $(DIST)/ClaudeBrain.app/Contents/MacOS/brain-app
	@echo "\nRun MCP server:  ./dist/brain-mcp"
	@echo "Open desktop app: open ./dist/Claude\\ Brain.app"

# Build the MCP server binary
brain-mcp:
	cargo build --release -p brain-mcp
	@mkdir -p $(DIST)
	cp target/release/claude-brain $(DIST)/brain-mcp

# Build the frontend WASM bundle (prerequisite for brain-app)
frontend:
	cd crates/brain-app-frontend && trunk build --release

# Build the Tauri desktop app (includes frontend build)
brain-app: frontend
	cd crates/brain-app && cargo tauri build
	@mkdir -p $(DIST)
	cp -R target/release/bundle/macos/ClaudeBrain.app $(DIST)/

# Run the desktop app in dev mode (Tauri manages trunk via beforeDevCommand)
dev:
	cd crates/brain-app && cargo tauri dev

# Run all tests
test:
	cargo test --workspace

# Remove build artifacts
clean:
	cargo clean
	rm -rf $(DIST)
	rm -rf crates/brain-app-frontend/dist
