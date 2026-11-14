#!/usr/bin/env bash
# Konvertu la kernan ELF-on al plata `Image` per la `rust-objcopy` de la
# aktiva Rust-ilaro (la sistemo `objcopy` eble ne konas aarch64).
# Uzado: tools/elf-al-image.sh <elf> <bildo>

set -euo pipefail

ENIGO="${1:?uzado: $0 <elf> <bildo>}"
ELIGO="${2:?uzado: $0 <elf> <bildo>}"

SYSROOT="$(rustc --print sysroot)"
HOST="$(rustc -vV | sed -n 's/^host: //p')"
OBJCOPY="$SYSROOT/lib/rustlib/$HOST/bin/rust-objcopy"

if [[ ! -x "$OBJCOPY" ]]; then
    echo "rust-objcopy ne trovita sub $SYSROOT; instalu la komponanton llvm-tools" >&2
    exit 1
fi

# rust-objcopy uzas la kunhavigitan `libLLVM` de la ilaro.
export LD_LIBRARY_PATH="$SYSROOT/lib:$SYSROOT/lib/rustlib/$HOST/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
exec "$OBJCOPY" -O binary "$ENIGO" "$ELIGO"
