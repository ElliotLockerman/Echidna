
# ALWAYS BUILD WITH THE MAKEFILE
# echidna-lib contains, as a CONST variable, the echidna-shim binary. This allows
# echidna-cli to exist as a standalone executable. However, cargo doesn't (yet)
# recognize a dependency on a binary, so it tries to build echidna-lib first, or
# worse, without recompiling echidna-shim.

MODE = release

E_SHIM = target/$(MODE)/echidna-shim
E_LIB = target/$(MODE)/echidna-lib
E_CLI = target/$(MODE)/echidna-cli
E_APP = target/$(MODE)/echidna-app
# echidna-helpers should be find as a regular cargo dep

all: $(E_CLI) $(E_APP)

$(E_SHIM):
	cargo build --release --bin echidna-shim

$(E_LIB): $(E_SHIM)
	cargo build --release --lib

$(E_CLI): $(E_LIB)
	cargo build --release --bin echidna-cli

$(E_APP): $(E_LIB)
	cargo build --release --bin echidna-app

.PHONY: all clean
clean:
	-@rm -r target &>/dev/null || true

