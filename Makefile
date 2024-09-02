# Makefile for RapidMQ

# Variables
CARGO = cargo
TARGET = target
BIN = $(TARGET)/debug/rapidmq
TESTS = tests/integration_tests.rs
PROTO_DIR = proto
PROTO_FILES = $(PROTO_DIR)/rapidmq.proto

# Default target
all: build

# Build the project
build:
	$(CARGO) build

# Run the project
run: build
	$(BIN) 1 2 3

# Clean the project
clean:
	$(CARGO) clean

# Run unit tests
test:
	$(CARGO) test

# Run integration tests
integration-test:
	$(CARGO) test --test integration_tests

# Generate protobuf files
proto:
	$(CARGO) build --build-plan | grep -oE 'tonic_build::compile_protos\(".*?"\)' | xargs -I {} sh -c '{}'

# Generate SSL certificates
generate-certs:
	./generate_certs.sh

# Help message
help:
	@echo "Makefile for RapidMQ"
	@echo ""
	@echo "Usage:"
	@echo "  make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  all                Build the project (default)"
	@echo "  build              Build the project"
	@echo "  run                Run the project"
	@echo "  clean              Clean the project"
	@echo "  test               Run unit tests"
	@echo "  integration-test   Run integration tests"
	@echo "  proto              Generate protobuf files"
	@echo "  generate-certs     Generate SSL certificates"
	@echo "  help               Show this help message"

.PHONY: all build run clean test integration-test proto generate-certs help