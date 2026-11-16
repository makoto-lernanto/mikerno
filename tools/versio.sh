#!/usr/bin/env bash
# Kontrolu aŭ ĝisdatigu la dokumentan version kontraŭ `Cargo.toml`.
#
# La sola fonto de la versio estas `[workspace.package] version`. La kerno
# legas ĝin per `env!("CARGO_PKG_VERSION")` kaj `limine.conf` ne portas
# version, do nur la dokumentaj lokoj kaj la retejo bezonas ĉi tiun skripton.
#
# La `arm64`-ŝpuro uzas antaŭeldonojn `0.x.y-arm.N`; ankaŭ tiuj formoj estas
# akceptataj kaj ĝisdatigataj.
#
# Uzado:
#   tools/versio.sh --kontroli    # malsukcesas se iu loko malkongruas
#   tools/versio.sh --ĝisdatigi  # anstataŭigas la version en la dokumentoj

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO="$ROOT/Cargo.toml"
README="$ROOT/README.md"
MEJLOSTONOJ="$ROOT/docs/MEJLOSTONOJ.md"
AGENTS="$ROOT/AGENTS.md"
RETEJO="$ROOT/docs/index.html"
KERNO="$ROOT/mikerno/src/kerno/mod.rs"
LIMINE="$ROOT/mikerno/boot/limine.conf"

VERSIO="$(sed -n '/^\[workspace\.package\]/,/^\[/s/^version = "\([^"]*\)".*/\1/p' "$CARGO" | head -n 1)"

if [[ -z "$VERSIO" ]]; then
    echo "versio: [workspace.package] version mankas en $CARGO" >&2
    exit 1
fi

# Eltiru la version el unu dosiero per la donita sed-esprimo; redonu ĝin.
eltiri() {
    local dosiero="$1" esprimo="$2"
    sed -nE "$esprimo" "$dosiero" | head -n 1
}

# Kontrolu unu lokon; redonu 1 kaj raportu se ĝi malkongruas.
kontroli() {
    local dosiero="$1" esprimo="$2" priskribo="$3" valoro
    valoro="$(eltiri "$dosiero" "$esprimo")"
    if [[ "$valoro" != "$VERSIO" ]]; then
        printf 'versio: %s enhavas «%s», sed Cargo.toml diras «%s»\n' \
            "$priskribo" "${valoro:-<mankas>}" "$VERSIO" >&2
        return 1
    fi
}

kontroli_ĉiujn() {
    local eraro=0
    kontroli "$README" \
        's/^- \*\*Versio:\*\* ([0-9]+\.[0-9]+\.[0-9]+(-arm\.[0-9]+)?) \(workspace\)$/\1/p' \
        "README.md (Versio)" || eraro=1
    kontroli "$README" \
        's/^\[MiKerno\] Saluton, mi estas MiKerno v([0-9]+\.[0-9]+\.[0-9]+(-arm\.[0-9]+)?)!$/\1/p' \
        "README.md (ekzempla saluto)" || eraro=1
    kontroli "$MEJLOSTONOJ" \
        's/^> Stato: .* · Versio de la workspace: ([0-9]+\.[0-9]+\.[0-9]+(-arm\.[0-9]+)?)$/\1/p' \
        "docs/MEJLOSTONOJ.md" || eraro=1
    kontroli "$AGENTS" \
        's/^Nuna versio: `([0-9]+\.[0-9]+\.[0-9]+(-arm\.[0-9]+)?)`.*$/\1/p' \
        "AGENTS.md" || eraro=1
    kontroli "$RETEJO" \
        's/^<html lang="eo" data-versio="([0-9]+\.[0-9]+\.[0-9]+(-arm\.[0-9]+)?)">$/\1/p' \
        "docs/index.html (retejo)" || eraro=1
    if grep -oE 'data-versio-montri>[^<]+' "$RETEJO" |
        sed 's/.*>//' | grep -qvxF "$VERSIO"; then
        echo "versio: docs/index.html montras alian videblan version ol $VERSIO" >&2
        eraro=1
    fi
    if ! grep -q 'env!("CARGO_PKG_VERSION")' "$KERNO"; then
        echo "versio: la saluta linio ne legas env!(\"CARGO_PKG_VERSION\") en $KERNO" >&2
        eraro=1
    fi
    if grep -Eq '^/.* v[0-9]+\.[0-9]+\.[0-9]+(-arm\.[0-9]+)?' "$LIMINE"; then
        echo "versio: limine.conf ankoraŭ enhavas version en la menuo" >&2
        eraro=1
    fi
    return "$eraro"
}

ĝisdatigi() {
    sed -E -i "s/^(- \*\*Versio:\*\* )[0-9]+\.[0-9]+\.[0-9]+(-arm\.[0-9]+)?/\1$VERSIO/" "$README"
    sed -E -i "s/^(\[MiKerno\] Saluton, mi estas MiKerno v)[0-9]+\.[0-9]+\.[0-9]+(-arm\.[0-9]+)?!/\1$VERSIO!/" "$README"
    sed -E -i "s/^(> Stato: .* · Versio de la workspace: )[0-9]+\.[0-9]+\.[0-9]+(-arm\.[0-9]+)?$/\1$VERSIO/" "$MEJLOSTONOJ"
    sed -E -i "s/^(Nuna versio: \`)[0-9]+\.[0-9]+\.[0-9]+(-arm\.[0-9]+)?(\`)/\1$VERSIO\3/" "$AGENTS"
    sed -E -i "s/^(<html lang=\"eo\" data-versio=\")[0-9]+\.[0-9]+\.[0-9]+(-arm\.[0-9]+)?(\">)$/\1$VERSIO\3/" "$RETEJO"
    sed -E -i "s#(data-versio-montri>)[0-9]+\.[0-9]+\.[0-9]+(-arm\.[0-9]+)?(</span>)#\1$VERSIO\3#g" "$RETEJO"
    sed -E -i "s#(href=\"[^\"]*/v)[0-9]+\.[0-9]+\.[0-9]+(-arm\.[0-9]+)?#\1$VERSIO#g" "$RETEJO"
}

case "${1:-}" in
    --kontroli)
        if kontroli_ĉiujn; then
            echo "versio: ĉiuj dokumentaj lokoj kongruas kun $VERSIO"
        else
            exit 1
        fi
        ;;
    --ĝisdatigi)
        ĝisdatigi
        echo "versio: dokumentoj ĝisdatigitaj al $VERSIO"
        kontroli_ĉiujn
        ;;
    *)
        echo "uzado: $0 --kontroli | --ĝisdatigi" >&2
        exit 2
        ;;
esac
