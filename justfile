# MiKerno: konstruado, PVH-rulado kaj ISO-rulado.
# Postulas bash, QEMU (`qemu-system-x86`) kaj, por `iso`, la sisteman `xorriso`.
# Por `run-arm` kaj `run-iso-arm` ankaŭ necesas `qemu-system-aarch64`; por la
# ISO krome UEFI-firmware.

kerno := "target/x86_64-unknown-none/release/mikerno"
bildo := "mikerno.iso"
bildo_arm := "target/aarch64-unknown-none/release/mikerno.img"
bildo_arm_iso := "mikerno-arm.iso"

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
    bash tools/run-qemu.sh {{kerno}}

# Kunmetu praŝarĝeblan ISO-on (Limine + Multiboot2).
iso: build-kerno
    @command -v xorriso >/dev/null || { echo "xorriso ne trovita; instalu ĝin: sudo apt install xorriso" >&2; exit 1; }
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
    xorriso -as mkisofs -b limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table --efi-boot limine-uefi-cd.bin -efi-boot-part --efi-boot-image --protective-msdos-label iso_root -o {{bildo}}

# Rulu la ISO-on en QEMU; la fenestro montras la Limine-menuon.
run-iso: iso
    bash tools/run-qemu.sh iso

# Rulu la antaŭe konstruitan arm64-kernon sub QEMU (`virt`).
# La bildo venas el la `arm64`-branĉo (`just build-arm`) kaj nur estas startigata.
run-arm:
    bash tools/run-qemu-arm.sh {{bildo_arm}}

# La ISO devenas el la `arm64`-branĉo (`just iso-arm`) kaj nur estas startigata.
# Rulu la arm64-ISO-on en QEMU kun fenestro kaj seria konzolo.
run-iso-arm:
    bash tools/run-qemu-arm.sh --iso {{bildo_arm_iso}}

# Kontrolu, ke la dokumenta versio kongruas kun `Cargo.toml`.
kontroli-version:
    bash tools/versio.sh --kontroli

# Kompleta kvalita pordego: formato + lint + konstruo + versio + formala pruvo.
# Kani ne subtenas tutmondan asemblaĵon; la pruvoj ne atingas ĝin.
verify: kontroli-version
    cargo fmt --all -- --check
    cargo clippy --workspace --release -- -D warnings
    cargo build --workspace --release
    cargo kani -p mikerno -Z unstable-options --ignore-global-asm
