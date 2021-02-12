TARGET ?= aarch64-none-elf
CROSS ?= $(TARGET)

GDB := $(CROSS)-gdb
LDFLAGS ?= --gc-sections -static -nostdlib -nostartfiles --no-dynamic-linker
CARGO ?= CARGO_INCREMENTAL=0 cargo

LD_LAYOUT := layout.ld

RUST_BINARY := $(shell cat Cargo.toml | grep name | cut -d\" -f 2 | tr - _)
RUST_BUILD_DIR := target/$(TARGET)
RUST_RELEASE_BIN := $(RUST_BUILD_DIR)/release/$(RUST_BINARY)

RUST_LIB_DEPS = ../lib/pi/src/* ../lib/pi/src/*/**

RUST_DEPS = Cargo.toml $(LD_LAYOUT) $(RUST_LIB_DEPS) $(shell find src -type f)

BUILD_DIR := build
KERNEL := $(BUILD_DIR)/$(RUST_BINARY)

.PHONY: all clean $(BUILD_DIR)/initfs.cpio

VPATH = ext

all: $(KERNEL).bin

$(BUILD_DIR)/initfs.cpio: $(BUILD_DIR)
	@echo "+ Building Userland"
	@cd ../userland; \
	$(CARGO) build -Z unstable-options --release --out-dir build; \
	cd build; \
	echo hello world > hello; \
	ls * | xargs aarch64-none-elf-strip; \
	ls * | cpio -H newc -ov > $(PWD)/build/initfs.cpio

$(RUST_RELEASE_BIN): $(RUST_DEPS) $(BUILD_DIR)/initfs.cpio | $(BUILD_DIR)
	@echo "+ Building $@"
	@$(CARGO) build --target ${TARGET}.json --release

$(BUILD_DIR):
	@mkdir -p $@

$(KERNEL).bin: $(RUST_RELEASE_BIN) | $(BUILD_DIR)
	@echo "+ Building $@ [objcopy $<]"
	@$(CARGO) objcopy --release -- -O binary $@

clean:
	cd ../userland; $(CARGO) clean; rm -rf build
	$(CARGO) clean
	rm -rf $(BUILD_DIR)

qemu-debug: $(KERNEL).bin
	qemu-system-aarch64 -nographic -M raspi3 -serial null -serial mon:stdio -kernel $(KERNEL).bin -gdb tcp::1234 -S

qemu: $(KERNEL).bin
	qemu-system-aarch64 -nographic -M raspi3 -serial null -serial mon:stdio -kernel $(KERNEL).bin

gdb:
	$(GDB)

