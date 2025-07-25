AS := nasm
ASFLAGS := -felf64

PROJDIRS := kernel kernel_loader kernel_shared multiboot std acpi

RUST_SRC_FILES := $(shell find $(PROJDIRS) -type f -name "*.rs") config.toml

ASM_SRC_DIR := kernel_loader/src/arch/x86_64/asm
ASM_OBJ_DIR := target/asm

ASM_SRC_FILES := $(wildcard $(ASM_SRC_DIR)/*.asm)
ASM_OBJ_FILES := $(patsubst %.asm, target/asm/%.o, $(notdir $(ASM_SRC_FILES)))

BIN_FILE := target/isofiles/boot/rustyos
LOADER_FILE = target/isofiles/boot/rustyos-loader

GRUB_FILES := kernel_loader/src/arch/x86_64/boot
LIB_FILE := target/x86_64-unknown-rustyos/release/libkernel.a
LOADER_LIB_FILE := target/x86_64-unknown-rustyos/release/libkernel_loader.a
ISO_FILE := target/rustyos.iso


KERNEL_FEATURES := $(shell tomlq '[to_entries[] | . as $$outer_entry | $$outer_entry.value | to_entries[] | {($$outer_entry.key + "/" + .key): .value }] | add | to_entries[] | select(.value).key' config.toml)

run: $(ISO_FILE)
	qemu-system-x86_64 \
				-drive file=$(ISO_FILE),format=raw \
				-display gtk,show-tabs=on -m 256M \
				-cpu qemu64-v1,pdpe1gb \
				-no-reboot \
				-machine pc-q35-10.0 \
				-serial stdio

clean: 
	cargo clean

$(ISO_FILE): $(BIN_FILE) $(LOADER_FILE) $(wildcard $(GRUB_FILES)/**/*)
	cp -r $(GRUB_FILES)/ target/isofiles
	grub-mkrescue -o $(ISO_FILE) target/isofiles

$(BIN_FILE): $(RUST_SRC_FILES) kernel/layout.ld
	cargo build --release --package kernel $(if $(KERNEL_FEATURES), --features $(KERNEL_FEATURES))
	mkdir -p target/isofiles/boot
	ld -n --no-gc-sections --no-warn-rwx-segment \
		-Tkernel/layout.ld -o $(BIN_FILE) \
		$(LIB_FILE)

$(LOADER_FILE): $(LOADER_LIB_FILE) $(ASM_OBJ_FILES) kernel_loader/layout.ld
	mkdir -p target/isofiles/boot
	ld -n --gc-sections --no-warn-rwx-segment \
		-Tkernel_loader/layout.ld -o $(LOADER_FILE) \
		$(ASM_OBJ_FILES) $(LOADER_LIB_FILE)

$(LOADER_LIB_FILE): $(RUST_SRC_FILES) kernel_loader/layout.ld
	cargo build --release --package kernel_loader $(if $(KERNEL_FEATURES), --features $(KERNEL_FEATURES))

$(ASM_OBJ_DIR)/%.o: $(ASM_SRC_DIR)/%.asm
	mkdir -p $(shell dirname $@)
	$(AS) $(ASFLAGS) $< -o $@