CARGO := cargo
TARGET := aarch64-none-elf
CROSS := $(TARGET)
TARGET_FILE := ../$(TARGET).json
RUST_LIB := target/$(TARGET)/release/libinit_thread.a
BUILD := $(shell pwd)/build
LD_FLAGS := --gc-sections -static -nostdlib -nostartfiles --no-dynamic-linker

BINARIES = shell

.PHONY: all $(BINARIES)

all: $(RUST_LIB) $(BINARIES)

$(RUST_LIB): $(BINARIES) $(BUILD)
	$(CARGO) build --target=$(TARGET_FILE) --release
	$(CROSS)-ld $(LD_FLAGS) $(RUST_LIB) -o $(BUILD)/init_thread.elf

$(BUILD):
	mkdir -p $(BUILD)

$(BINARIES): $(BUILD)
	cd ../$@; $(CARGO) build -Z unstable-options --target=$(TARGET_FILE) --out-dir $(BUILD) --release

clean:
	rm -rf build target
	$(foreach bin, $(BINARIES), cd ../$(bin); $(CARGO) clean;)