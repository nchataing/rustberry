TARGET ?= pi2
VERSION ?= release

BUILD_DIR = target/$(TARGET)/$(VERSION)
KERNEL = kernel

KERNEL_ASSEMBLY_OBJECTS = $(BUILD_DIR)/kernel/boot.o
KERNEL_RUST_LIB = $(BUILD_DIR)/librustberry_kernel.a
KERNEL_LINKER_SCRIPT = kernel/linker.ld

QEMU_OPTIONS = -M raspi2 -serial stdio -display none -d "int,cpu_reset,unimp,guest_errors"

FEATURES ?= mini_uart
ifeq ($(VERSION), release)
	VERSION_FLAG = --release
else
	VERSION_FLAG =
endif
XARGO_FLAGS = $(VERSION_FLAG) --features "$(TARGET) $(FEATURES)"

all: $(BUILD_DIR)/$(KERNEL).img $(BUILD_DIR)/$(KERNEL).asm

run: $(BUILD_DIR)/$(KERNEL).elf
	qemu-system-arm $(QEMU_OPTIONS) -kernel $<

gdb: $(BUILD_DIR)/$(KERNEL).elf
	qemu-system-arm $(QEMU_OPTIONS) -kernel $< -s -S & \
	gdb-multiarch $< -ex 'target remote localhost:1234'

clean:
	rm -rf target

%.asm: %.elf
	arm-none-eabi-objdump -D $< > $@

%.hex: %.elf
	arm-none-eabi-objcopy $< -O ihex $@

%.img: %.elf
	arm-none-eabi-objcopy $< -O binary $@

$(BUILD_DIR)/$(KERNEL).elf: xargo/kernel $(KERNEL_ASSEMBLY_OBJECTS)
	arm-none-eabi-ld --gc-sections -T $(KERNEL_LINKER_SCRIPT) -o $@ $(KERNEL_ASSEMBLY_OBJECTS) $(KERNEL_RUST_LIB)

$(BUILD_DIR)/%.o: %.s
	mkdir -p $(dir $@)
	arm-none-eabi-as $(AS_FLAGS) $< -o $@

xargo/%:
	cd $(notdir $@) && RUST_TARGET_PATH=$(shell pwd) xargo build --target $(TARGET) $(XARGO_FLAGS)

.PHONY: all kernel clean run gdb xargo/*
