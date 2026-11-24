#!/usr/bin/env bash
# Kontrolu, ke la sistemvokaj numeroj en `libmikerno` kaj la pruvokatalogo
# kongruas kun la dokumentaro.
#
# Uzado:
#   tools/dokumentaro.sh --kontroli
#
# La skripto eltiras ĉiun `pub const SYS_*: usize = N;` el
# `libmikerno/src/syscall.rs` kaj serĉas la parojn `SYS_X = N` aŭ
# `SYS_X` (N) en la dokumentoj. Malkongrua aŭ mankanta numero estas eraro.
# Poste ĝi komparas la nombron da vicoj en la pruvokatalogo de
# `docs/MEJLOSTONOJ.md` kun la nombro da `#[kani::proof]`-atributoj.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SYSCALL="$ROOT/libmikerno/src/syscall.rs"
DOKUMENTOJ=("$ROOT/README.md" "$ROOT/docs/VORTARO.md")
KATALOGO="$ROOT/docs/MEJLOSTONOJ.md"

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

# La pruvokatalogo estas la regiono inter la markiloj; ĉiu vico komenciĝas per
# sia numero. `git grep` legas nur la spuritajn fontojn, do `target/` ne
# duobligas la atributojn.
atributoj=$(git -C "$ROOT" grep -F '#[kani::proof]' -- '*.rs' | wc -l)
katalogo=$(
    awk '
        /<!-- pruvokatalogo: komenco -->/ { ene = 1; next }
        /<!-- pruvokatalogo: fino -->/ { ene = 0; next }
        ene && /^\| [0-9]+ \|/ { nombro++ }
        END { print nombro + 0 }
    ' "$KATALOGO"
)

if [[ "$katalogo" -eq 0 ]]; then
    echo "dokumentaro: la pruvokatalogo mankas en ${KATALOGO#"$ROOT"/}" >&2
    exit 1
fi
if [[ "$katalogo" -ne "$atributoj" ]]; then
    printf 'dokumentaro: la katalogo havas %s vicojn, sed la deponejo havas %s `#[kani::proof]`-atributojn\n' \
        "$katalogo" "$atributoj" >&2
    exit 1
fi

echo "dokumentaro: ${#kodo[@]} sistemvokoj kongruas kun README.md kaj docs/VORTARO.md"
echo "dokumentaro: $atributoj Kani-pruvoj kongruas kun la katalogo en docs/MEJLOSTONOJ.md"
