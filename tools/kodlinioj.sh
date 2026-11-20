#!/usr/bin/env bash
# Nombru la liniojn de la Ring 0-kodo (`mikerno/src` kaj `kernologiko/src`).
#
# Uzado:
#   tools/kodlinioj.sh            # homa superrigardo
#   tools/kodlinioj.sh --linio    # unu vico por la tabelo en MEJLOSTONOJ.md
#
# «Kodo» estas ĉiu ne-malplena linio, kiu ne komenciĝas per `//` aŭ `/*`.
# Liniaj komentoj post kodo estas nombritaj kiel kodo. La blokoj
# `#[cfg(test)]` ne apartenas al la Ring 0-bildo kaj restas ekster la nombro.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FONTOJ=("$ROOT/mikerno/src" "$ROOT/kernologiko/src")
VERSIO="$(sed -n '/^\[workspace\.package\]/,/^\[/s/^version = "\([^"]*\)".*/\1/p' \
    "$ROOT/Cargo.toml" | head -n 1)"

nombri() {
    local dosieroj linioj kodo komento malplena
    mapfile -t dosieroj < <(find "${FONTOJ[@]}" -type f -name '*.rs' | sort)
    read -r linioj kodo komento malplena < <(
        awk '
            atestado { if ($0 ~ /^}/) atestado = 0; next }
            /^#\[cfg\(test\)\]/ { atestado = 1; next }
            { total++ }
            /^[[:space:]]*$/ { blank++; next }
            /^[[:space:]]*(\/\/|\/\*|\*)/ { comment++; next }
            { code++ }
            END { print total+0, code+0, comment+0, blank+0 }
        ' "${dosieroj[@]}"
    )
    printf '%s %s %s %s %s %s\n' \
        "${#dosieroj[@]}" "$linioj" "$kodo" "$komento" "$malplena" "$VERSIO"
}

read -r DOSIEROJ LINIOJ KODO KOMENTO MALPLENA _ < <(nombri)

if [[ "${1:-}" == "--linio" ]]; then
    printf '| %s | %s | %s | %s | %s | %s | %s |\n' \
        "$(date +%F)" "$VERSIO" "$DOSIEROJ" "$LINIOJ" "$KODO" "$KOMENTO" "$MALPLENA"
    exit 0
fi

printf 'Ring 0 (mikerno/src kaj kernologiko/src), versio %s\n' "$VERSIO"
printf '  dosieroj %s\n' "$DOSIEROJ"
printf '  linioj   %s\n' "$LINIOJ"
printf '  kodo     %s\n' "$KODO"
printf '  komento  %s\n' "$KOMENTO"
printf '  malplena %s\n' "$MALPLENA"
