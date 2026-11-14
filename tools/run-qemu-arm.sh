#!/usr/bin/env bash
# Startigu la arm64-kernon sub QEMU (`virt`) kaj ligi la serian konzolon al
# ĉi tiu terminalo.
# Uzado:
#   tools/run-qemu-arm.sh [debug|release]
#   tools/run-qemu-arm.sh <vojo-al-kerno>
#   tools/run-qemu-arm.sh --iso [<vojo-al-iso>]

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARG="${1:-release}"

QEMU="${QEMU_ARM:-}"
if [[ -z "$QEMU" ]]; then
    if command -v qemu-system-aarch64 >/dev/null 2>&1; then
        QEMU="qemu-system-aarch64"
    else
        echo "qemu-system-aarch64 ne trovita; instalu ĝin (sudo apt install qemu-system-arm) aŭ agordu QEMU_ARM=/vojo/al/qemu-system-aarch64" >&2
        exit 1
    fi
fi

if [[ "$ARG" == "--iso" ]]; then
    ISO="${2:-$ROOT/mikerno-arm.iso}"
    if [[ ! -f "$ISO" ]]; then
        echo "ISO ne trovita: $ISO (ruligu: just iso-arm)" >&2
        exit 1
    fi
    FIRMWARE="${QEMU_ARM_EFI:-}"
    if [[ -z "$FIRMWARE" && -f /usr/share/qemu-efi-aarch64/QEMU_EFI.fd ]]; then
        FIRMWARE=/usr/share/qemu-efi-aarch64/QEMU_EFI.fd
    fi
    if [[ -z "$FIRMWARE" || ! -f "$FIRMWARE" ]]; then
        echo "UEFI-firmware ne trovita; instalu qemu-efi-aarch64 aŭ agordu QEMU_ARM_EFI" >&2
        exit 1
    fi
    exec "$QEMU" \
        -M virt \
        -cpu "${QEMU_ARM_CPU:-cortex-a53}" \
        -m 256M \
        -bios "$FIRMWARE" \
        -cdrom "$ISO" \
        -serial mon:stdio \
        -display none \
        -no-reboot
fi

if [[ -f "$ARG" ]]; then
    KERNEL="$ARG"
else
    case "$ARG" in
        debug)   KERNEL="$ROOT/target/aarch64-unknown-none/debug/mikerno" ;;
        release) KERNEL="$ROOT/target/aarch64-unknown-none/release/mikerno" ;;
        *) echo "uzado: $0 [debug|release|<kerno>|--iso [<iso>]]" >&2; exit 2 ;;
    esac
fi

if [[ ! -f "$KERNEL" ]]; then
    echo "kerna bildo ne trovita: $KERNEL (ruligu: just build-arm)" >&2
    exit 1
fi

# `mon:stdio` muxas la serian konzolon kaj la QEMU-monitoron. Por eliri:
#   Ctrl+A x   — tuj fini QEMU
#   Ctrl+A c   — ŝalti la monitoron (tie: `quit`)
# PSCI `SYSTEM_OFF` (la halto de la kerno) finas QEMU-on; `-no-shutdown` ne
# estas uzata, por ke tiu fino estu videbla al la testilo.
exec "$QEMU" \
    -M virt \
    -cpu "${QEMU_ARM_CPU:-cortex-a53}" \
    -m 256M \
    -kernel "$KERNEL" \
    -serial mon:stdio \
    -display none \
    -no-reboot
