SHELL := /bin/sh
TOOLS_DIR := $(CURDIR)/.tools
BUF := $(TOOLS_DIR)/buf
PROTOC_GEN_GO := $(TOOLS_DIR)/protoc-gen-go
PROTOC_GEN_GO_GRPC := $(TOOLS_DIR)/protoc-gen-go-grpc
CARGO ?= $(shell command -v cargo 2>/dev/null || printf '%s/.cargo/bin/cargo' "$(HOME)")
COMPOSE ?= docker compose

.PHONY: proto build test lint data build-routing dev compose-config clean

proto: $(BUF) $(PROTOC_GEN_GO) $(PROTOC_GEN_GO_GRPC)
	PATH="$(TOOLS_DIR):$$PATH" $(BUF) lint
	PATH="$(TOOLS_DIR):$$PATH" $(BUF) generate

$(BUF):
	mkdir -p $(TOOLS_DIR)
	GOBIN=$(TOOLS_DIR) go install github.com/bufbuild/buf/cmd/buf@v1.47.2

$(PROTOC_GEN_GO):
	mkdir -p $(TOOLS_DIR)
	GOBIN=$(TOOLS_DIR) go install google.golang.org/protobuf/cmd/protoc-gen-go@v1.35.1

$(PROTOC_GEN_GO_GRPC):
	mkdir -p $(TOOLS_DIR)
	GOBIN=$(TOOLS_DIR) go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@v1.5.1

build: proto
	npm run build --prefix gpx
	cd services/api && go build ./...
	cd services/route-engine && $(CARGO) build --locked
	npm run build --prefix website

test: proto
	cd services/api && go test ./...
	cd services/route-engine && $(CARGO) test --locked

lint: proto
	npm exec --prefix website prettier -- --check AGENTS.md docs proto services migrations deploy scripts compose.yaml
	cd services/api && test -z "$$(gofmt -l .)" && go vet ./...
	cd services/route-engine && $(CARGO) fmt --check && $(CARGO) clippy --locked --all-targets -- -D warnings

ifndef OSM_PBF_URL
data:
	$(error OSM_PBF_URL is required)
else ifndef OSM_PBF_SHA256
data:
	$(error OSM_PBF_SHA256 is required)
else
data:
	OSM_PBF_URL="$(OSM_PBF_URL)" OSM_PBF_SHA256="$(OSM_PBF_SHA256)" ./scripts/prepare-data.sh
endif

build-routing:
	test -s data/region.osm.pbf
	$(COMPOSE) build graphhopper
	$(COMPOSE) run --rm graphhopper import

dev:
	$(COMPOSE) up --build

compose-config:
	$(COMPOSE) config --quiet

clean:
	rm -rf services/api/bin services/route-engine/target
