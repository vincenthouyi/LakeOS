TARGET ?= aarch64-none-elf
CROSS ?= $(TARGET)

CC := $(CROSS)-gcc
AS := $(CROSS)-as
GDB := $(CROSS)-gdb
PI_TTY ?= /dev/tty.SLAB_USBtoUART
CCFLAGS ?= -Wall -O2 -nostdlib -nostartfiles -ffreestanding -pie -fpie
LDFLAGS ?= --gc-sections -static -nostdlib -nostartfiles --no-dynamic-linker
CARGO ?= CARGO_INCREMENTAL=0 cargo

LD_LAYOUT := layout.ld

RUST_BINARY := $(shell cat Cargo.toml | grep name | cut -d\" -f 2 | tr - _)
RUST_BUILD_DIR := target/$(TARGET)
RUST_DEBUG_LIB := $(RUST_BUILD_DIR)/debug/lib$(RUST_BINARY).a
RUST_RELEASE_LIB := $(RUST_BUILD_DIR)/release/lib$(RUST_BINARY).a

RUST_LIB_DEPS = ../lib/pi/src/* ../lib/pi/src/*/**

RUST_DEPS = Cargo.toml $(LD_LAYOUT) $(RUST_LIB_DEPS) $(shell find src -type f)
EXT_DEPS = $(BUILD_DIR)/init.o

BUILD_DIR := build
KERNEL := $(BUILD_DIR)/$(RUST_BINARY)
RUST_LIB := $(BUILD_DIR)/$(RUST_BINARY).a

INIT_THREAD_DIR := $(PWD)/../init_thread
INIT_THREAD_TARGET := $(INIT_THREAD_DIR)/target/$(TARGET)/release/init_thread

.PHONY: all test clean check install $(RUST_LIB) $(BUILD_DIR)/init_thread

VPATH = ext

all: $(KERNEL).bin $(RUST_LIB)

check:
	@$(XARGO) check --target=$(TARGET)

test:
	@$(CARGO) test

install: $(KERNEL).bin
	$(TTYWRITE) -i $< $(PI_TTY)

tftp: $(KERNEL).bin
	cp $(KERNEL).bin /private/tftpboot/

picocom:
	picocom -b 115200 $(PI_TTY)

$(RUST_DEBUG_LIB): $(RUST_DEPS) $(BUILD_DIR)/init_thread
	@echo "+ Building $@ [xargo]"
	@$(CARGO) build --target=$(TARGET).json

$(RUST_RELEASE_LIB): $(RUST_DEPS) $(BUILD_DIR)/init_thread
	@echo "+ Building $@ [xargo --release]"
	@$(CARGO) build --target ${TARGET}.json --release
#	@$(CARGO) xbuild --target ${TARGET}.json
#	@$(XARGO) build --release --target=$(TARGET)

#$(EXT_DEPS): ext/* ${BUILD_DIR}
#	make -C ext

$(BUILD_DIR)/%.o : ext/%.S | $(BUILD_DIR)
	@echo "+ Building $@ [as $<]"
	$(CC) $(CCFLAGS) -c $< -o $@

ifeq ($(DEBUG),1)
$(RUST_LIB): $(RUST_DEBUG_LIB) | $(BUILD_DIR)
	@cp $< $@
else
$(RUST_LIB): $(RUST_RELEASE_LIB) | $(BUILD_DIR)
	@cp $< $@
endif

$(BUILD_DIR):
	@mkdir -p $@

$(BUILD_DIR)/%.o: %.c | $(BUILD_DIR)
	@echo "+ Building $@ [cc $<]"
	@$(CC) $(CCFLAGS) -c $< -o $@

$(BUILD_DIR)/%.o: %.s | $(BUILD_DIR)
	@echo "+ Building $@ [as $<]"
	@$(CC) $(CCFLAGS) -c $< -o $@

$(KERNEL).elf: $(EXT_DEPS) $(RUST_LIB) $(LD_LAYOUT) | $(BUILD_DIR)
	@echo "+ Building $@ [ld $^]"
	$(CROSS)-ld $(LDFLAGS) -T$(LD_LAYOUT) $^ -o $@

$(KERNEL).hex: $(KERNEL).elf | $(BUILD_DIR)
	@echo "+ Building $@ [objcopy $<]"
	@$(CROSS)-objcopy $< -O ihex $@

$(KERNEL).bin: $(KERNEL).elf | $(BUILD_DIR)
	@echo "+ Building $@ [objcopy $<]"
	@$(CROSS)-objcopy $< -O binary $@

clean:
	make clean -C ../init_thread
	$(CARGO) clean
	rm -rf $(BUILD_DIR)

qemu-debug: $(KERNEL).bin
	qemu-system-aarch64 -nographic -M raspi3 -serial null -serial mon:stdio -kernel $(KERNEL).bin -gdb tcp::1234 -S

qemu: $(KERNEL).bin
	qemu-system-aarch64 -nographic -M raspi3 -serial null -serial mon:stdio -kernel $(KERNEL).bin

$(BUILD_DIR)/init_thread:
	make -C $(INIT_THREAD_DIR)
	cp $(INIT_THREAD_DIR)/build/init_thread $(BUILD_DIR)/init_thread

gdb:
	$(GDB)

