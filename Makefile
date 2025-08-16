# Makefile for the `otl` Rust CLI

CARGO ?= cargo
BIN    ?= otl

# Set RELEASE=0 for debug builds
RELEASE ?= 1

ifeq ($(RELEASE),1)
  BUILD_MODE = --release
  TARGET_DIR = target/release
else
  BUILD_MODE =
  TARGET_DIR = target/debug
endif

.PHONY: help build run test fmt clippy clean install check canon json watch diff binpath skpdoc openpdf

help:
	@echo "Common targets:"
	@echo "  build        Build ($(if $(RELEASE),release,debug))"
	@echo "  run ARGS=..  Run with args (e.g., ARGS='--canon file.OTL')"
	@echo "  test         Run tests"
	@echo "  fmt          Format code (rustfmt)"
	@echo "  clippy       Lint with clippy (deny warnings)"
	@echo "  clean        Clean target directory"
	@echo "  install      cargo install --path ."
	@echo "  check        fmt + clippy + test"
	@echo "  canon FILE=  Print canonical dump for FILE"
	@echo "  json FILE=   Print JSON for FILE"
	@echo "  watch TARGET= [ARGS=..]  Use watch-otl.sh on file/dir"
	@echo "  diff PREV= CURR= [CURSOR=1]  Canon diff two .OTL files"
	@echo "  binpath      Print built binary path"
	@echo "  skpdoc       Build PDF for about_skplus/cmds_key_mappings.md and open"
	@echo "  %.pdf        Generic: build PDF from Markdown via pandoc"

# -------- Pandoc PDF (Markdown -> PDF) --------
PANDOC ?= pandoc
PDF_ENGINE ?= pdflatex
# Markdown reader flavor (source now controls breaks via formatting)
PANDOC_FROM ?= markdown
PANDOC_FLAGS ?= --toc --toc-depth=2 -V geometry:margin=1in

# Generic rule: any foo.pdf from foo.md
%.pdf: %.md
	@echo $(PANDOC) -f $(PANDOC_FROM) $< -o $@ --pdf-engine=$(PDF_ENGINE) $(PANDOC_FLAGS)
	@$(PANDOC) -f $(PANDOC_FROM) "$<" -o "$@" --pdf-engine="$(PDF_ENGINE)" $(PANDOC_FLAGS)

# Specific helper for Sidekick Plus command doc

skpdoc:
	@$(MAKE) about_skplus/cmds_key_mappings.pdf
	@command -v xdg-open >/dev/null 2>&1 && xdg-open about_skplus/cmds_key_mappings.pdf >/dev/null 2>&1 || true

openpdf:
	@command -v xdg-open >/dev/null 2>&1 && xdg-open $(FILE) >/dev/null 2>&1 || echo "Use: make openpdf FILE=path/to/file.pdf"

build:
	$(CARGO) build $(BUILD_MODE)

run:
	$(CARGO) run $(BUILD_MODE) -- $(ARGS)

test:
	$(CARGO) test

fmt:
	$(CARGO) fmt --all

clippy:
	$(CARGO) clippy -- -D warnings

clean:
	$(CARGO) clean

install:
	$(CARGO) install --path .

check: fmt clippy test

canon:
	@test -n "$(FILE)" || (echo "Usage: make canon FILE=path/to/file.OTL" && exit 2)
	$(CARGO) run $(BUILD_MODE) -- --canon $(FILE)

json:
	@test -n "$(FILE)" || (echo "Usage: make json FILE=path/to/file.OTL" && exit 2)
	$(CARGO) run $(BUILD_MODE) -- --json $(FILE)

watch:
	@test -n "$(TARGET)" || (echo "Usage: make watch TARGET=<file|dir> [ARGS='--validate']" && exit 2)
	./watch-otl.sh $(TARGET) -- $(ARGS)

diff:
	@test -n "$(PREV)" -a -n "$(CURR)" || (echo "Usage: make diff PREV=prev.OTL CURR=curr.OTL [CURSOR=1]" && exit 2)
	$(CARGO) run $(BUILD_MODE) -- --diff $(PREV) $(CURR) $(if $(CURSOR),--show-cursor,)

binpath:
	@echo $(TARGET_DIR)/$(BIN)
