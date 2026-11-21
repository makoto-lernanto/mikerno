#!/usr/bin/env bash
# Kontrolu, ke la sistemvokaj numeroj en `libmikerno` kongruas kun la
# dokumentaro (README.md kaj docs/VORTARO.md).
#
# Uzado:
#   tools/dokumentaro.sh --kontroli
#
# La skripto eltiras ĉiun `pub const SYS_*: usize = N;` el
# `libmikerno/src/syscall.rs` kaj serĉas la parojn `SYS_X = N` aŭ
# `SYS_X` (N) en la dokumentoj. Malkongrua aŭ mankanta numero estas eraro.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SYSCALL="$ROOT/libmikerno/src/syscall.rs"
DOKUMENTOJ=("$ROOT/README.md" "$ROOT/docs/VORTARO.md")

if [[ "${1:-}" != "--kontroli" ]]; then
    echo "uzado: $0 --kontroli" >&2
    exit 2
fi

declare -A kodo=()
declare -A dokumentita=()
eraro=0

while read -r nomo numero; do
    kodo["$nomo"]="$numero"
done < <(
    sed -nE 's/^pub const (SYS_[^:]+): usize = ([0-9]+);$/\1 \2/p' "$SYSCALL"
)

if [[ "${#kodo[@]}" -eq 0 ]]; then
    echo "dokumentaro: neniuj sistemvokoj trovitaj en $SYSCALL" >&2
    exit 1
fi

# Eltiru `SYS_X = N` kaj `SYS_X` (N) el la dokumentoj.
eltiri() {
    local dosiero="$1"
    {
        grep -oP 'SYS_[^ =`()]+`?\s*=\s*[0-9]+' "$dosiero" |
            sed -E 's/`//g; s/ *= */ /'
        grep -oP 'SYS_[^ =`()]+`?\s*\([0-9]+\)' "$dosiero" |
            sed -E 's/`//g; s/ *\(([0-9]+)\)/ \1/'
    } || true
}

for dosiero in "${DOKUMENTOJ[@]}"; do
    while read -r nomo numero; do
        if [[ -z "${kodo[$nomo]+jes}" ]]; then
            printf 'dokumentaro: %s mencias %s, sed ĝi mankas en libmikerno\n' \
                "${dosiero#"$ROOT"/}" "$nomo" >&2
            eraro=1
        elif [[ "${kodo[$nomo]}" != "$numero" ]]; then
            printf 'dokumentaro: %s diras %s = %s, sed libmikerno diras %s\n' \
                "${dosiero#"$ROOT"/}" "$nomo" "$numero" "${kodo[$nomo]}" >&2
            eraro=1
        else
            dokumentita["$nomo"]=1
        fi
    done < <(eltiri "$dosiero")
done

for nomo in "${!kodo[@]}"; do
    if [[ -z "${dokumentita[$nomo]+jes}" ]]; then
        printf 'dokumentaro: %s (%s) ne estas dokumentita kun sia numero\n' \
            "$nomo" "${kodo[$nomo]}" >&2
        eraro=1
    fi
done

if [[ "$eraro" -ne 0 ]]; then
    exit 1
fi

echo "dokumentaro: ${#kodo[@]} sistemvokoj kongruas kun README.md kaj docs/VORTARO.md"
