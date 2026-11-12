# MiKerno: konstruado, PVH-rulado kaj ISO-rulado.

kerno := "target/x86_64-unknown-none/release/mikerno"
bildo := "mikerno.iso"

default: build

# Konstruu la tutan workspace (kerno, libmikerno, horizonto).
build-all:
    cargo build --workspace --release

# Konstruu nur la kernon.
build-kerno:
    cargo build -p mikerno --release

# Kongruo kun la antaŭa nomo.
build: build-kerno

# Rulu la kernon rekte per QEMU (PVH, sen ISO).
run: build-kerno
    cargo run -p mikerno --release

# Kunmetu praŝarĝeblan ISO-on (Limine + Multiboot2).
iso: build-kerno
    cargo build -p horizonto --release
    rm -rf iso_root
    mkdir -p iso_root
    cp {{kerno}} iso_root/mikerno.elf
    cp target/x86_64-unknown-none/release/horizonto iso_root/horizonto.elf
    cp mikerno/boot/limine.conf iso_root/limine.conf
    cp tools/limine/limine-bios.sys iso_root/
    cp tools/limine/limine-bios-cd.bin iso_root/
    cp tools/limine/limine-uefi-cd.bin iso_root/
    mkdir -p iso_root/EFI/BOOT
    cp tools/limine/EFI/BOOT/BOOTX64.EFI iso_root/EFI/BOOT/
    tools/xorriso/xorriso.exe -as mkisofs -b limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table --efi-boot limine-uefi-cd.bin -efi-boot-part --efi-boot-image --protective-msdos-label iso_root -o {{bildo}}

# Rulu la ISO-on en QEMU; la fenestro montras la Limine-menuon.
run-iso: iso
    bash tools/run-qemu.sh iso

# Pruvu la kernajn algoritmojn per Kani (formala verifikado).
verify:
    cargo kani -p mikerno
