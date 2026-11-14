#!/usr/bin/env bash
# Kreas UEFI-booteblan `mikerno-arm.iso` per Limine kaj la plata `Image`.
# Postulas `xorriso`, `mkfs.fat` (dosfstools) kaj `mmd`/`mcopy` (mtools).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGO="$ROOT/target/aarch64-unknown-none/release/mikerno.img"
RADIKO="$ROOT/iso_root_arm"
EFI_IMAGO="$ROOT/efi-arm.img"
ELIGO="$ROOT/mikerno-arm.iso"

for ilo in xorriso mkfs.fat mmd mcopy; do
    if ! command -v "$ilo" >/dev/null 2>&1; then
        echo "mankas $ilo; instalu xorriso, dosfstools kaj mtools" >&2
        exit 1
    fi
done

if [[ ! -f "$IMAGO" ]]; then
    echo "kerna bildo ne trovita: $IMAGO (ruligu: just build-arm)" >&2
    exit 1
fi

rm -rf "$RADIKO"
mkdir -p "$RADIKO/EFI/BOOT"
cp "$IMAGO" "$RADIKO/mikerno.img"
cp "$ROOT/mikerno/boot/limine-arm.conf" "$RADIKO/limine.conf"
cp "$ROOT/tools/limine/EFI/BOOT/BOOTAA64.EFI" "$RADIKO/EFI/BOOT/BOOTAA64.EFI"

dd if=/dev/zero of="$EFI_IMAGO" bs=1M count=4 status=none
mkfs.fat -F 12 "$EFI_IMAGO" >/dev/null
mmd -i "$EFI_IMAGO" ::/EFI ::/EFI/BOOT
mcopy -i "$EFI_IMAGO" "$ROOT/tools/limine/EFI/BOOT/BOOTAA64.EFI" ::/EFI/BOOT/
cp "$EFI_IMAGO" "$RADIKO/efi.img"

xorriso -as mkisofs -o "$ELIGO" \
    -e efi.img -no-emul-boot -eltorito-platform efi \
    "$RADIKO"
