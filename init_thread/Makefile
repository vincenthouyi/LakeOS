CARGO := cargo
TARGET := aarch64-none-elf
CROSS := $(TARGET)
TARGET_FILE := ../$(TARGET).json
RUST_LIB := target/$(TARGET)/release/libinit_thread.a
BUILD := build
LD_FLAGS := --gc-sections -static -nostdlib -nostartfiles --no-dynamic-linker

RPI3B_DIR := ../rpi3b

all: $(BUILD)/rpi3b.elf | $(BUILD) 
	$(CARGO) xbuild --target=$(TARGET_FILE) --release
	$(CROSS)-ld $(LD_FLAGS) $(RUST_LIB) -o $(BUILD)/init_thread.elf

$(BUILD)/rpi3b.elf:
	make -C $(RPI3B_DIR)
	cp $(RPI3B_DIR)/build/rpi3b.elf $(BUILD)

$(BUILD):
	mkdir -p $(BUILD)
