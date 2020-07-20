CARGO := cargo
TARGET := aarch64-none-elf
CROSS := $(TARGET)
TARGET_FILE := ../$(TARGET).json
RUST_LIB := target/$(TARGET)/release/libinit_thread.a
BUILD := build
LD_FLAGS := --gc-sections -static -nostdlib -nostartfiles --no-dynamic-linker

SHELL_DIR := ../shell

.PHONY: $(BUILD)/shell.elf

all: $(BUILD)/shell.elf $(BUILD)
	$(CARGO) xbuild --target=$(TARGET_FILE) --release
	$(CROSS)-ld $(LD_FLAGS) $(RUST_LIB) -o $(BUILD)/init_thread.elf

$(BUILD)/shell.elf: $(BUILD)
	make -C $(SHELL_DIR)
	cp $(SHELL_DIR)/build/shell.elf $(BUILD)/shell.elf

$(BUILD):
	mkdir -p $(BUILD)

clean:
	rm -rf build target
	make clean -C ../shell