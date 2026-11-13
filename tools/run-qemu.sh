#!/usr/bin/env bash
# Startigu la kernon aŭ la ISO-on sub QEMU kaj ligi la serian konzolon al ĉi
# tiu terminalo.
# Uzado:
#   tools/run-qemu.sh [debug|release]
#   tools/run-qemu.sh <vojo-al-kerno>   (tio, kion pasigas `cargo run`)
#   tools/run-qemu.sh iso               (mikerno.iso)
#   tools/run-qemu.sh <vojo-al-iso>
#
# Por ISO la fenestro de QEMU montras la Limine-menuon; QEMU_DISPLAY=none
# ebligas senkapan ruladon.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARG="${1:-debug}"
# `cargo run` pasigas Windows-stilan vojon; normaligu ĝin por Git Bash.
ARG="${ARG//\\//}"

ISO=""
if [[ "$ARG" == "iso" ]]; then
    ISO="$ROOT/mikerno.iso"
elif [[ "$ARG" == *.iso ]]; then
    ISO="$ARG"
fi

if [[ -n "$ISO" ]]; then
    if [[ ! -f "$ISO" ]]; then
        echo "ISO ne trovita: $ISO (ruligu: just iso)" >&2
        exit 1
    fi
else
    if [[ -f "$ARG" ]]; then
        KERNEL="$ARG"
    else
        case "$ARG" in
            debug)   KERNEL="$ROOT/target/x86_64-unknown-none/debug/mikerno" ;;
            release) KERNEL="$ROOT/target/x86_64-unknown-none/release/mikerno" ;;
            *) echo "uzado: $0 [debug|release|iso|<kerno>|<iso>]" >&2; exit 2 ;;
        esac
    fi

    if [[ ! -f "$KERNEL" ]]; then
        echo "kerna bildo ne trovita: $KERNEL (ruligu: cargo build)" >&2
        exit 1
    fi

    # `cargo run` sendas ĉiun bildon de la cela triopo al ĉi tiu lanĉilo. Nur
    # la kerna bildo havas la Xen-PVH-noton; uzantaj bildoj (ekz. horizonto)
    # ne estas memstare praŝarĝeblaj per `qemu -kernel`.
    if ! grep -aq "Xen" "$KERNEL"; then
        if [[ "$(basename "$KERNEL")" == horizonto* ]]; then
            echo "La bildo estas uzanta (Ring 3) bildo, ne praŝarĝebla per QEMU -kernel." >&2
            echo "Konstruu ĝin per: cargo build -p horizonto" >&2
            echo "Lanĉu la kernon per: just run aŭ just run-iso" >&2
            exit 0
        fi
        echo "La bildo ne havas Xen-PVH-noton: $KERNEL" >&2
        echo "Ĝi ne estas praŝarĝebla per QEMU -kernel." >&2
        exit 1
    fi
fi

QEMU="${QEMU:-}"
if [[ -z "$QEMU" ]]; then
    if command -v qemu-system-x86_64 >/dev/null 2>&1; then
        QEMU="qemu-system-x86_64"
    elif [[ -x "/c/Program Files/qemu/qemu-system-x86_64.exe" ]]; then
        QEMU="/c/Program Files/qemu/qemu-system-x86_64.exe"
    else
        echo "qemu-system-x86_64 ne trovita; agordu QEMU=/vojo/al/qemu-system-x86_64" >&2
        exit 1
    fi
fi

# SMEP/SMAP: la implicita `qemu64` ne havigas ilin. Por la firmigaĵoj (M13)
# ni petas ilin eksplicite; `QEMU_CPU=qemu64` permesas kontroli la gracian
# degradon sur procesoro sen la trajtoj.
CPU_ARGS=(-cpu "${QEMU_CPU:-qemu64,+smep,+smap}")

# `mon:stdio` muxas la serian konzolon kaj la QEMU-monitoron sur la sama
# terminalo. Tio necesas ĉar Windows-QEMU forprenas `ENABLE_PROCESSED_INPUT`
# de la konzolo: Ctrl+C ne plu finas la procezon, ĝi nur sendiĝas al la guest
# kiel bajto 0x03. Por eliri uzu:
#   Ctrl+A x   — tuj fini QEMU
#   Ctrl+A c   — ŝalti la monitoron (tie: `quit`)
if [[ -n "$ISO" ]]; then
    # SDL evitas la GTK-temajn SVG-avertojn de la Windows-eldono de QEMU.
    DISPLAY_ARGS=(-display "${QEMU_DISPLAY:-sdl}")
    exec "$QEMU" \
        -cdrom "$ISO" \
        -m 256M \
        "${CPU_ARGS[@]}" \
        -serial mon:stdio \
        "${DISPLAY_ARGS[@]}" \
        -no-reboot \
        -no-shutdown
fi

# La uzanta bildo apud la kerna estas liverita kiel PVH-modulo (`-initrd`).
UZANTA="$(dirname "$KERNEL")/horizonto"
INITRD=()
if [[ -f "$UZANTA" ]]; then
    INITRD=(-initrd "$UZANTA")
fi

exec "$QEMU" \
    -kernel "$KERNEL" \
    "${INITRD[@]}" \
    -m 256M \
    "${CPU_ARGS[@]}" \
    -serial mon:stdio \
    -display none \
    -no-reboot \
    -no-shutdown
