# MiKerno: konstruado, PVH-rulado kaj ISO-rulado.
# Postulas bash, python3 (SNDS-bildo), QEMU (`qemu-system-x86`) kaj, por `iso`,
# la sisteman `xorriso`.
# Por `ruli-arm` kaj `ruli-iso-arm` ankaŭ necesas `qemu-system-aarch64`; por la
# ISO krome UEFI-firmware.

kerno := "target/x86_64-unknown-none/release/mikerno"
kerno_arm := "target/aarch64-unknown-none/release/mikerno"
bildo_arm := "target/aarch64-unknown-none/release/mikerno.img"
bildo_arm_iso := "mikerno-arm.iso"
bildo := "mikerno.iso"
# La gastiganta trinomo, sur kiu la puraj moduloj testiĝas kun `std`.
host := `rustc -vV | sed -n 's/^host: //p'`

default: konstrui

# Konstruu la tutan laborlokon (kerno, libmikerno, horizonto, saluto).
konstrui-ciujn:
    cargo build --workspace --release

# Konstruu nur la kernon.
konstrui-kerno:
    cargo build -p mikerno --release

# Kongrua alinomo: `konstrui` konstruas nur la kernon.
konstrui: konstrui-kerno

# Paku `mikerno/snds/` en determinsman SNDS-bildon.
snds:
    bash tools/krei_snds_img.sh mikerno/snds target/mikerno.snds

# Rulu la kernon rekte per QEMU (PVH, sen ISO).
ruli: konstrui-ciujn snds
    bash tools/run-qemu.sh {{kerno}}

# Konstruu la kernon por arm64 (plata `Image`) kaj la uzantajn bildojn.
konstrui-arm:
    cargo build -p mikerno --release --target aarch64-unknown-none
    cargo build -p horizonto --release --target aarch64-unknown-none
    cargo build -p saluto --release --target aarch64-unknown-none
    bash tools/elf-al-image.sh {{kerno_arm}} {{bildo_arm}}

# Rulu la arm64-kernon sub QEMU (`virt`).
ruli-arm: konstrui-arm
    bash tools/run-qemu-arm.sh {{bildo_arm}}

# Kunmetu UEFI-booteblan ISO-on por arm64 (Limine + plata `Image`).
iso-arm: konstrui-arm
    bash tools/krei-iso-arm.sh

# Rulu la arm64-ISO-on sub QEMU kun UEFI-firmware.
ruli-iso-arm: iso-arm
    bash tools/run-qemu-arm.sh --iso {{bildo_arm_iso}}

# Kunmetu praŝarĝeblan ISO-on (Limine + Multiboot2).
iso: konstrui-kerno
    @command -v xorriso >/dev/null || { echo "xorriso ne trovita; instalu ĝin: sudo apt install xorriso" >&2; exit 1; }
    cargo build -p horizonto --release
    cargo build -p saluto --release
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
ruli-iso: iso
    bash tools/run-qemu.sh iso

# Kontrolu, ke la dokumenta versio kongruas kun `Cargo.toml`.
kontroli-version:
    bash tools/versio.sh --kontroli

# Rulu la unu-testojn de la arkitekture sendependaj moduloj de `kernologiko`
# sur la gastiganto (ne en la barela celo).
testoj:
    cargo test -p kernologiko --target {{host}}

# Postulas noktan ilĉenon:
#   rustup toolchain install nightly --profile minimal --component miri --component rust-src

# Rulu la unu-testojn de `kernologiko` sub Miri (detekto de nedifinita konduto).
miri:
    cargo +nightly miri test -p kernologiko --target {{host}}

# Permesila kaj dependaĵa kontrolo laŭ `deny.toml`. Postulas `cargo-deny`
# (`cargo install cargo-deny --locked` aŭ la antaŭkonstruitan binaraĵon);
# en DI ĝi ruliĝas kiel aparta tasko.
permesiloj:
    cargo deny check

# Kani kaj Miri ruliĝas nur en DI: Kani kiel aparta tasko por ĉiu pruvo kaj
# Miri kiel aparta tasko (vidu di.yml).

# Loka kvalita pordego: versio + unu-testoj + formato + statika analizo
# (ambaŭ arkoj) + konstruo.
# La arm64-analizo kovras ankaŭ `horizonto` (la Ligo-komandojn) kaj `saluto`.
kontroli: kontroli-version testoj
    cargo fmt --all -- --check
    cargo clippy --workspace --release -- -D warnings
    cargo clippy -p mikerno --release --target aarch64-unknown-none -- -D warnings
    cargo clippy -p horizonto -p saluto --release --target aarch64-unknown-none -- -D warnings
    cargo build --workspace --release
