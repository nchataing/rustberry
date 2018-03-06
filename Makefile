TARGET ?= pi2
VERSION ?= release
KERNEL = kernel
SRC_DIR = src
BUILD_DIR = target/$(TARGET)/$(VERSION)

ASSEMBLY_OBJECTS = $(BUILD_DIR)/boot.o
KERNEL_RUST_LIB = $(BUILD_DIR)/librustberry.a
LINKER_SCRIPT = src/linker.ld

QEMU_OPTIONS = -M raspi2 -serial stdio -display none -d "int,cpu_reset,unimp,guest_errors"

ifeq ($(VERSION), release)
	XARGO_FLAGS = --release --features "$(TARGET)"
else
	XARGO_FLAGS = --features "$(TARGET)"
endif

all: $(BUILD_DIR)/$(KERNEL).img $(BUILD_DIR)/$(KERNEL).hex $(BUILD_DIR)/$(KERNEL).asm

run: $(BUILD_DIR)/$(KERNEL).elf
	qemu-system-arm $(QEMU_OPTIONS) -kernel $<

gdb: $(BUILD_DIR)/$(KERNEL).elf
	qemu-system-arm $(QEMU_OPTIONS) -kernel $< -s -S & \
	gdb-multiarch $< -ex 'target remote localhost:1234'

clean:
	rm -rf target

$(BUILD_DIR)/$(KERNEL).asm: $(BUILD_DIR)/$(KERNEL).elf
	arm-none-eabi-objdump -D $< > $@

$(BUILD_DIR)/$(KERNEL).hex: $(BUILD_DIR)/$(KERNEL).elf
	arm-none-eabi-objcopy $< -O ihex $@

$(BUILD_DIR)/$(KERNEL).img: $(BUILD_DIR)/$(KERNEL).elf
	arm-none-eabi-objcopy $< -O binary $@

$(BUILD_DIR)/$(KERNEL).elf: xargo $(ASSEMBLY_OBJECTS)
	arm-none-eabi-ld --gc-sections -T $(LINKER_SCRIPT) -o $@ $(ASSEMBLY_OBJECTS) $(KERNEL_RUST_LIB)

$(BUILD_DIR)/%.o: $(SRC_DIR)/%.s
	arm-none-eabi-as $(AS_FLAGS) $< -o $@

xargo:
	RUST_TARGET_PATH=$(shell pwd) xargo build --target $(TARGET) $(XARGO_FLAGS)

.PHONY: all clean run gdb xargo
