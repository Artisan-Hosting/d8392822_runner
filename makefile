# Variables
PROJECT_NAME := ais_63c35f4b
PROJECT_DIR := $(CURDIR)
TARGET_DIR := $(PROJECT_DIR)/target
RELEASE_DIR := $(TARGET_DIR)/release
BIN_DIR := /usr/local/bin
CONFIG_DIR := /etc/$(PROJECT_NAME)
CONFIG_FILES := Config.toml Overrides.toml
INSTALL_BIN := $(RELEASE_DIR)/$(PROJECT_NAME)

# Targets
.PHONY: all build setup install clean uninstall

# Default target: build and setup
all: build

# Setup the build environment (create directories if needed)
setup:
	@echo "Setting up directory structure..."
	@mkdir -p $(RELEASE_DIR)
	@echo "Directory structure setup complete."

# Build the Rust project in release mode
build: setup
	@echo "Building the project in release mode..."
	cargo build --release
	@echo "Build complete."

# Install the binary and configuration files on the system
install: build
	@echo "Installing $(PROJECT_NAME)..."
	# Install the binary
	@install -m 0755 $(INSTALL_BIN) $(BIN_DIR)
	@echo "Binary installed to $(BIN_DIR)/$(PROJECT_NAME)"
	# Install the configuration files
	@mkdir -p $(CONFIG_DIR)
	@cp $(CONFIG_FILES) $(CONFIG_DIR)
	@chmod 644 $(CONFIG_DIR)/*
	@echo "Configuration files installed to $(CONFIG_DIR)"
	@echo "Installation complete."

# Uninstall the project (remove installed files)
uninstall:
	@echo "Uninstalling $(PROJECT_NAME)..."
	# Remove the binary
	@rm -f $(BIN_DIR)/$(PROJECT_NAME)
	@echo "Binary removed from $(BIN_DIR)"
	# Remove the configuration files
	@rm -rf $(CONFIG_DIR)
	@echo "Configuration files removed from $(CONFIG_DIR)"
	@echo "Uninstallation complete."

# Clean build artifacts
clean:
	@echo "Cleaning up build artifacts..."
	cargo clean
	@rm -rf $(RELEASE_DIR)
	@echo "Clean complete."

