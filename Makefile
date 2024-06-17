
# ALWAYS BUILD WITH THE MAKEFILE
# echidna-lib contains, as a CONST variable, the echidna-shim binary. This allows
# echidna-cli to exist as a standalone executable. However, cargo doesn't (yet)
# recognize a dependency on a binary, so it tries to build echidna-lib first, or
# worse, without recompiling echidna-shim.

MODE_DIR = release
MODE_FLAG = --$(MODE_DIR)
ifeq ($(mode), )
# Nothing
else ifeq ($(mode), release)
# Nothing
else ifeq ($(mode), debug)
    MODE_DIR = debug
    MODE_FLAG =
else
    $(error Invalid mode argument $(mode))
endif


E_SHIM = target/$(MODE_DIR)/echidna-shim
E_LIB = target/$(MODE_DIR)/echidna_lib.rlib
E_CLI = target/$(MODE_DIR)/echidna-cli
E_APP = target/$(MODE_DIR)/echidna
E_APP_BUNDLE = target/$(MODE_DIR)/Echidna.app
# echidna-helpers should be fine as a regular cargo dep


all: $(E_CLI) $(E_APP_BUNDLE)

$(E_SHIM):
	cargo build $(MODE_FLAG) --bin echidna-shim

$(E_LIB): $(E_SHIM)
	cargo build $(MODE_FLAG) --lib

$(E_CLI): $(E_LIB)
	cargo build $(MODE_FLAG) --bin echidna-cli

$(E_APP): $(E_LIB)
	cargo build $(MODE_FLAG) --bin echidna

$(E_APP_BUNDLE): $(E_APP) echidna/app_files/Info.plist echidna/scripts/make-app.sh
	./echidna/scripts/make-app.sh $(MODE_FLAG)


.PHONY: all clean $(E_SHIM)
clean:
	-@rm -r target &>/dev/null || true

