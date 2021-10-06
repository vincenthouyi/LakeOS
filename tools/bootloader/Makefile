TARGET ?= aarch64-none-elf
CROSS ?= $(TARGET)

GDB := $(CROSS)-gdb
CARGO ?= CARGO_INCREMENTAL=0 cargo

LD_LAYOUT := layout.ld

RUST_BINARY := $(shell cat Cargo.toml | grep name | cut -d\" -f 2 | tr - _)
RUST_BUILD_DIR := target/$(TARGET)
RUST_RELEASE_BIN := $(RUST_BUILD_DIR)/release/$(RUST_BINARY)

RUST_DEPS = Cargo.toml $(LD_LAYOUT) $(RUST_LIB_DEPS) $(shell find src -type f)

BUILD_DIR := $(PWD)/build
KERNEL := $(BUILD_DIR)/target/rustyl4
KERNEL_DIR := ../../kernel
USERLAND_DIR := ../../userland
BOOTLOADER := $(BUILD_DIR)/$(RUST_BINARY)

.PHONY: all clean $(BOOTLOADER).bin $(KERNEL) userland $(BUILD_DIR)/initfs.cpio initfs

VPATH = ext

all: $(BOOTLOADER).bin

initfs: $(BUILD_DIR)/initfs.cpio

$(BUILD_DIR)/initfs.cpio: userland $(KERNEL) $(BUILD_DIR)
	mkdir -p $(BUILD_DIR)/fs
	echo hello world > $(BUILD_DIR)/fs/hello
	cp $(KERNEL) $(BUILD_DIR)/fs
	ls $(BUILD_DIR)/target | xargs -I '{}' $(CROSS)-strip -o $(BUILD_DIR)/fs/'{}' $(BUILD_DIR)/target/'{}'
	cd $(BUILD_DIR)/fs; ls | cpio -H newc -ov > $(BUILD_DIR)/initfs.cpio

userland: $(BUILD_DIR)
	@echo "+ Building Userland"
	@cd $(USERLAND_DIR); $(CARGO) build -Z unstable-options --release --out-dir $(BUILD_DIR)/target;

$(KERNEL): $(BUILD_DIR)
	@echo "+ Building Kernel"
	@cd $(KERNEL_DIR); $(CARGO) build -Z unstable-options --release --out-dir $(BUILD_DIR)/target;

$(RUST_RELEASE_BIN): $(RUST_DEPS) $(BUILD_DIR)/initfs.cpio $(KERNEL)| $(BUILD_DIR)
	@echo "+ Building $@"
	@$(CARGO) build --target ${TARGET}.json --release

$(BUILD_DIR):
	@mkdir -p $@

$(BOOTLOADER).bin: $(RUST_RELEASE_BIN) | $(BUILD_DIR)
	@echo "+ Building $@ [objcopy $<]"
	@$(CROSS)-objcopy $(RUST_RELEASE_BIN) -O binary $@

clean:
	$(CARGO) clean
	cd $(KERNEL_DIR); $(CARGO) clean
	cd $(USERLAND_DIR); $(CARGO) clean
	rm -rf $(BUILD_DIR)

qemu-debug: $(BOOTLOADER).bin
	qemu-system-aarch64 -nographic -M raspi3 -serial null -serial mon:stdio -kernel $(BOOTLOADER).bin -gdb tcp::1234 -S

qemu: $(BOOTLOADER).bin
	qemu-system-aarch64 -nographic -M raspi3 -serial null -serial mon:stdio -kernel $(BOOTLOADER).bin

gdb:
	$(GDB)

