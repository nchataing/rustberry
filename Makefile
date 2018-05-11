TARGET ?= pi2
VERSION ?= release

BUILD_DIR = $(abspath target/$(TARGET)/$(VERSION))
KERNEL = kernel
BOOTLOADER = bootloader

KERNEL_OBJECTS = $(BUILD_DIR)/kernel/boot.o $(BUILD_DIR)/kernel/exceptions.o \
				 $(BUILD_DIR)/librustberry_kernel.a
KERNEL_LINKER_SCRIPT = kernel/kernel_link.ld

BOOTLOADER_OBJECTS = $(BUILD_DIR)/bootloader/boot.o \
					 $(BUILD_DIR)/librustberry_bootloader.a
BOOTLOADER_LINKER_SCRIPT = bootloader/bootloader_link.ld

PROGRAMS = syscall_loop
PROGRAM_LINKER_SCRIPT = programs/prgm_link.ld

# Comma-separated list, use help to show the list of available options
QEMU_DEBUG = "unimp"
QEMU_OPTIONS = -M raspi2 -m 256 -serial stdio -display none -d $(QEMU_DEBUG)

ifeq ($(VERSION), release)
	VERSION_FLAG = --release
else ifeq ($(VERSION), debug)
	VERSION_FLAG =
else
	VERSION_FLAG = $(error Unknown VERSION: $(VERSION))
endif
XARGO_FLAGS = $(VERSION_FLAG) --features "$(TARGET) $(FEATURES)"

all: kernel bootloader programs

kernel: $(BUILD_DIR)/$(KERNEL).img $(BUILD_DIR)/$(KERNEL).asm

bootloader: $(BUILD_DIR)/$(BOOTLOADER).img $(BUILD_DIR)/$(BOOTLOADER).asm

programs: $(foreach prgm, $(PROGRAMS), $(BUILD_DIR)/prgm/$(prgm))

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

$(BUILD_DIR)/$(KERNEL).elf: $(KERNEL_OBJECTS)
	arm-none-eabi-ld --gc-sections -T $(KERNEL_LINKER_SCRIPT) -o $@ $^

$(BUILD_DIR)/$(BOOTLOADER).elf: $(BOOTLOADER_OBJECTS)
	arm-none-eabi-ld --gc-sections -T $(BOOTLOADER_LINKER_SCRIPT) -o $@ $^

$(BUILD_DIR)/%.o: %.s
	mkdir -p $(dir $@)
	arm-none-eabi-as $(AS_FLAGS) $< -o $@

-include $(BUILD_DIR)/lib*.d
$(BUILD_DIR)/librustberry_%.a:
	cd $* && CC=arm-none-eabi-gcc RUST_TARGET_PATH=$(shell pwd) \
		xargo build --target $(TARGET) $(XARGO_FLAGS)

$(BUILD_DIR)/lib%.a: programs/%/
	cd $< && RUST_TARGET_PATH=$(shell pwd) \
		xargo build --target $(TARGET) $(VERSION_FLAG)

$(BUILD_DIR)/prgm/%: $(BUILD_DIR)/lib%.a
	mkdir -p $(BUILD_DIR)/prgm/
	arm-none-eabi-ld --gc-sections -T $(PROGRAM_LINKER_SCRIPT) -o $@ $^

.PHONY: all kernel bootloader programs clean run gdb
