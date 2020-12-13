CARGO := cargo
TARGET := aarch64-none-elf
TARGET_FILE := $(TARGET).json
RUST_LIB := target/$(TARGET)/release/libinit_thread.a
BUILD := $(shell pwd)/build

.PHONY: all userland

all: $(RUST_LIB) $(BINARIES)

$(RUST_LIB): userland $(BUILD)
	$(CARGO) build -Z unstable-options --target=$(TARGET_FILE) --release --out-dir $(BUILD)

$(BUILD):
	mkdir -p $(BUILD)

userland:
	cd ../userland; $(CARGO) build -Z unstable-options --out-dir $(BUILD) --release

clean:
	rm -rf build target
	$(foreach bin, $(BINARIES), cd ../$(bin); $(CARGO) clean;)