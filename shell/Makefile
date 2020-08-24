CARGO := cargo
TARGET := aarch64-none-elf
CROSS := $(TARGET)
TARGET_FILE := ../$(TARGET).json
RUST_LIB := target/$(TARGET)/release/shell
BUILD := build
LD_FLAGS := --gc-sections -static -nostdlib -nostartfiles --no-dynamic-linker

all: | $(BUILD)
	$(CARGO) xbuild --target=$(TARGET_FILE) --release
	cp $(RUST_LIB) $(BUILD)/shell.elf

$(BUILD):
	mkdir -p $(BUILD)

clean:
	rm -rf target build