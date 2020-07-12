CARGO := cargo
TARGET := aarch64-none-elf
CROSS := $(TARGET)
TARGET_FILE := ../$(TARGET).json
RUST_LIB := target/$(TARGET)/release/librpi3b.a
BUILD := build
LD_FLAGS := --gc-sections -static -nostdlib -nostartfiles --no-dynamic-linker

all: | $(BUILD)
	$(CARGO) xbuild --target=$(TARGET_FILE) --release
	$(CROSS)-ld $(LD_FLAGS) $(RUST_LIB) -o $(BUILD)/rpi3b.elf

$(BUILD):
	mkdir -p $(BUILD)

clean:
	rm -rf target build