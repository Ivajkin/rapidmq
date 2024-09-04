# Makefile for RapidMQ

CARGO := cargo
DOCKER := docker
DOCKER_IMAGE := rapidmq
DOCKER_TAG := latest
TEST_COMMAND := $(CARGO) test
RELEASE_FLAG := --release

.PHONY: all build test clean docker-build docker-run deploy

all: build test

build:
	$(CARGO) build $(RELEASE_FLAG)

test:
	$(TEST_COMMAND)

clean:
	$(CARGO) clean
	rm -rf src/protos

docker-build:
	$(DOCKER) build -t $(DOCKER_IMAGE):$(DOCKER_TAG) .

docker-run:
	$(DOCKER) run -p 9092:9092 -p 9093:9093 $(DOCKER_IMAGE):$(DOCKER_TAG)

deploy: docker-build
	./scripts/deploy.sh

integration-test:
	./scripts/run_integration_tests.sh

benchmark:
	./scripts/run_benchmarks.sh

lint:
	$(CARGO) clippy -- -D warnings

format:
	$(CARGO) fmt

check: lint test

help:
	@echo "Available targets:"
	@echo "  build            - Build the project"
	@echo "  test             - Run unit tests"
	@echo "  clean            - Clean build artifacts"
	@echo "  docker-build     - Build Docker image"
	@echo "  docker-run       - Run Docker container"
	@echo "  deploy           - Deploy to production"
	@echo "  integration-test - Run integration tests"
	@echo "  benchmark        - Run benchmarks"
	@echo "  lint             - Run linter"
	@echo "  format           - Format code"
	@echo "  check            - Run linter and tests"
	@echo "  help             - Show this help message"