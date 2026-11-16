#!/usr/bin/env bash
# Startigu la arm64-kernon aŭ la arm64-ISO-on sub QEMU (`virt`) kaj ligi la
# serian konzolon al ĉi tiu terminalo.
# Uzado:
#   tools/run-qemu-arm.sh [debug|release]
#   tools/run-qemu-arm.sh <vojo-al-kerno>
#   tools/run-qemu-arm.sh --iso [<vojo-al-iso>]
#
# Ĉe la ISO la fenestro montras la UEFI/Limine-ekranon kaj poste la kerman
# frambufferion per `ramfb`; la kerna eligo ankoraŭ aperas sur la seria
# konzolo. Por eliri: Ctrl+A x. `QEMU_DISPLAY=none` donas senkapan ruladon.

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

# La testdisko servas al la virtio-blk-memprovo (M17). `-snapshot` certigas, ke
# la memprovo ne modifu la dosieron eĉ se la restarigo malsukcesus.
TESTDISKO="$ROOT/target/testdisko.img"
if [[ ! -f "$TESTDISKO" ]]; then
    "$ROOT/tools/krei-testan-diskon.sh" "$TESTDISKO" >/dev/null
fi
DISKA_ARGS=(
    -drive "file=$TESTDISKO,format=raw,if=none,id=disk0"
    -device "virtio-blk-device,drive=disk0"
    -global "virtio-mmio.force-legacy=false"
    -snapshot
)

if [[ "$ARG" == "--iso" ]]; then
    ISO="${2:-$ROOT/mikerno-arm.iso}"
    if [[ ! -f "$ISO" ]]; then
        echo "ISO ne trovita: $ISO (ruligu: just iso-arm)" >&2
        exit 1
    fi
    FIRMWARE="${QEMU_ARM_EFI:-}"
    if [[ -z "$FIRMWARE" ]]; then
        for kandidato in \
            /usr/share/qemu-efi-aarch64/QEMU_EFI.fd \
            /usr/share/edk2/aarch64/QEMU_EFI.fd \
            /usr/share/AAVMF/AAVMF_CODE.fd; do
            if [[ -f "$kandidato" ]]; then
                FIRMWARE="$kandidato"
                break
            fi
        done
    fi
    if [[ -z "$FIRMWARE" || ! -f "$FIRMWARE" ]]; then
        echo "UEFI-firmware ne trovita; instalu qemu-efi-aarch64 aŭ agordu QEMU_ARM_EFI=/vojo/al/firmware" >&2
        exit 1
    fi
    # `ramfb` donas framebuffer-on al la UEFI-firmware kaj al la kerno, do la
    # fenestro montras la UEFI/Limine-ekranon kaj poste la kerman fiksaĵon; la
    # kerna eligo restas sur la seria konzolo.
    # AAVMF (2025.11) blokiĝas sub TCG kun `-cpu max` antaŭ ol atingi la
    # praŝarĝilon; `cortex-a76` havas `PAN` kaj praŝarĝas la ISO-on.
    exec "$QEMU" \
        -M virt \
        -cpu "${QEMU_ARM_CPU:-cortex-a76}" \
        -m 256M \
        -bios "$FIRMWARE" \
        -cdrom "$ISO" \
        -device ramfb \
        "${DISKA_ARGS[@]}" \
        -serial mon:stdio \
        -display "${QEMU_DISPLAY:-sdl}" \
        -no-reboot
fi

if [[ -f "$ARG" ]]; then
    KERNEL="$ARG"
else
    # Oni pasigu la plataĵon `Image`, ne la ELF-on: QEMU liveras la DTB-on en
    # `x0` nur kiam ĝi rekonas la Linux-`Image`-kapon.
    case "$ARG" in
        debug)   KERNEL="$ROOT/target/aarch64-unknown-none/debug/mikerno.img" ;;
        release) KERNEL="$ROOT/target/aarch64-unknown-none/release/mikerno.img" ;;
        *) echo "uzado: $0 [debug|release|<kerno>|--iso [<iso>]]" >&2; exit 2 ;;
    esac
fi

if [[ ! -f "$KERNEL" ]]; then
    echo "kerna bildo ne trovita: $KERNEL (ruligu: just build-arm)" >&2
    exit 1
fi

INITRD="$ROOT/target/aarch64-unknown-none/release/horizonto"
INITRD_ARGS=()
if [[ -f "$INITRD" ]]; then
    INITRD_ARGS=(-initrd "$INITRD")
fi

# `mon:stdio` muxas la serian konzolon kaj la QEMU-monitoron. Por eliri:
#   Ctrl+A x   — tuj fini QEMU
#   Ctrl+A c   — ŝalti la monitoron (tie: `quit`)
# PSCI `SYSTEM_OFF` (la halto de la kerno) finas QEMU-on; `-no-shutdown` ne
# estas uzata, por ke tiu fino estu videbla al la testilo.
exec "$QEMU" \
    -M virt \
    -cpu "${QEMU_ARM_CPU:-max}" \
    -m 256M \
    -kernel "$KERNEL" \
    "${INITRD_ARGS[@]}" \
    -device ramfb \
    "${DISKA_ARGS[@]}" \
    -serial mon:stdio \
    -display none \
    -no-reboot
