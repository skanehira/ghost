# Build and install ghost binary
install:
    cargo install --path .

# Install and run development
dev: install

# Run all tests after installing  
test-all: install
    cargo test

# Watch for changes and auto-install
watch-install:
    cargo watch -x "install --path ."

# Show available commands
list:
    just --list