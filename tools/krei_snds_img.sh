#!/usr/bin/env bash
# Paku dosierujon en SNDS-bildon (Simpla NurLega Dosiersistemo).
#
# Uzado:
#   tools/krei_snds_img.sh [eniga_dosierujo] [eliga_dosiero] [nomo=vojo]...
#
# La defaŭltoj estas `mikerno/snds` kaj `target/mikerno.snds`. Pliaj argumentoj
# aldonas dosierojn al la radiko de la bildo (ekz. `horizonto.elf=…`), sen
# ŝanĝi la enigan dosierujon. La bildo estas determinisma: la enhavo estas
# pakata laŭ ordigo de la UTF-8-bajtoj, sen horindikoj. La skripto uzas nur
# python3, do GitHub Actions kaj WSL2 produktas identajn bildojn.
#
# Formato (ĉio little-endian, 512-bajtaj sektoroj):
#   sektoro 0        superbloko (32 bajtoj)
#   sektoro 1..      inoda tabelo (16 bajtoj po inodo)
#   datumsektoroj    dosierujaj eroj (64 bajtoj po ero) kaj dosieraj datumoj
#
# Superbloko: magio "SNDS", versio 1, bloka grando 512, inoda grando 16,
# nombro da inodoj, unua inoda sektoro, unua datuma sektoro, kapacito en
# sektoroj, radika inodo.
#
# Inodo: tipo (1 dosiero, 2 dosierujo), grando en bajtoj, unua sektoro,
# nombro da sektoroj. Dosieruja ero: inoda indekso kaj 60-bajta nomo kun
# fina NUL; la maksimuma nomo estas 59 bajtoj. La maksimuma dosiero estas
# 64 KiB.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FONTO="${1:-$ROOT/mikerno/snds}"
CELO="${2:-$ROOT/target/mikerno.snds}"

if [[ ! -d "$FONTO" ]]; then
    echo "SNDS: la eniga dosierujo ne ekzistas: $FONTO" >&2
    exit 1
fi

# Pliaj dosieroj: kopiu la enigan arbon al provizora dosierujo kaj aldonu ilin
# tie, por ke la python-pakaĵo restu senŝanĝa.
ARGUMENTOJ=("$@")
if [[ ${#ARGUMENTOJ[@]} -gt 2 ]]; then
    PROVIZORA="$(mktemp -d)"
    trap 'rm -rf "$PROVIZORA"' EXIT
    cp -a "$FONTO"/. "$PROVIZORA"/
    for ((indekso = 2; indekso < ${#ARGUMENTOJ[@]}; indekso++)); do
        spec="${ARGUMENTOJ[$indekso]}"
        nomo="${spec%%=*}"
        vojo="${spec#*=}"
        if [[ "$spec" == "$nomo" || -z "$nomo" || "$nomo" == */* || ! -f "$vojo" ]]; then
            echo "SNDS: nevalida aldonaĵo (atendata nomo=vojo): $spec" >&2
            exit 1
        fi
        cp "$vojo" "$PROVIZORA/$nomo"
    done
    FONTO="$PROVIZORA"
fi

mkdir -p "$(dirname "$CELO")"

python3 - "$FONTO" "$CELO" <<'PY'
import os
import stat
import struct
import sys

SEKTORO = 512
INODO = 16
ERO = 64
NOMO_MAKSIMO = 60  # 59 bajtoj + fina NUL
DOSIERO_MAKSIMO = 64 * 1024
TIPO_DOSIERO = 1
TIPO_DOSIERUJO = 2

fonto, celo = sys.argv[1], sys.argv[2]


def eraro(mesaĝo):
    print(f"SNDS: {mesaĝo}", file=sys.stderr)
    sys.exit(1)


class Nodo:
    def __init__(self, nomo, tipo, enhavo):
        self.nomo = nomo
        self.tipo = tipo
        self.enhavo = enhavo
        self.indekso = 0
        self.unua = 0
        self.sektoroj = 0


def legi_dosierujon(vojo):
    eroj = []
    try:
        nomoj = sorted(os.listdir(vojo), key=lambda nomo: nomo.encode("utf-8"))
    except OSError as kaŭzo:
        eraro(f"ne legeblas {vojo}: {kaŭzo}")
    for nomo in nomoj:
        plena = os.path.join(vojo, nomo)
        try:
            stato = os.stat(plena, follow_symlinks=False)
        except OSError as kaŭzo:
            eraro(f"ne legeblas {plena}: {kaŭzo}")
        if len(nomo.encode("utf-8")) > NOMO_MAKSIMO - 1:
            eraro(f"nomo tro longa (maksimume 59 bajtoj): {nomo!r}")
        if stat.S_ISDIR(stato.st_mode):
            eroj.append(Nodo(nomo, TIPO_DOSIERUJO, legi_dosierujon(plena)))
        elif stat.S_ISREG(stato.st_mode):
            with open(plena, "rb") as dosiero:
                datumoj = dosiero.read()
            if len(datumoj) > DOSIERO_MAKSIMO:
                eraro(f"dosiero tro granda (maksimume 64 KiB): {plena}")
            eroj.append(Nodo(nomo, TIPO_DOSIERO, datumoj))
        else:
            eraro(f"ne subtenata dosierotipo: {plena}")
    return eroj


radiko = Nodo("", TIPO_DOSIERUJO, legi_dosierujon(fonto))
inodoj = []


def indeksigi(nodo):
    nodo.indekso = len(inodoj)
    inodoj.append(nodo)
    if nodo.tipo == TIPO_DOSIERUJO:
        for ero in nodo.enhavo:
            indeksigi(ero)


indeksigi(radiko)


def dosieruja_enhavo(nodo):
    bufro = bytearray()
    for ero in nodo.enhavo:
        nomo = ero.nomo.encode("utf-8")
        bufro += struct.pack("<I", ero.indekso)
        bufro += nomo + b"\0" * (NOMO_MAKSIMO - len(nomo))
    return bytes(bufro)


tabelaj = (len(inodoj) * INODO + SEKTORO - 1) // SEKTORO
unua_datumo = 1 + tabelaj
sekva = unua_datumo

for nodo in inodoj:
    if nodo.tipo == TIPO_DOSIERUJO:
        nodo.enhavo = dosieruja_enhavo(nodo)
    nodo.unua = sekva
    nodo.sektoroj = (len(nodo.enhavo) + SEKTORO - 1) // SEKTORO
    sekva += nodo.sektoroj

# La bildo havas almenaŭ 128 sektorojn (64 KiB); la virtio-blk-memprovo
# skribas kaj restarigas la lastan sektoron.
kapacito = max(128, (sekva + 127) // 128 * 128)
bildo = bytearray(kapacito * SEKTORO)

struct.pack_into(
    "<4sHHHHIIIII",
    bildo,
    0,
    b"SNDS",
    1,
    SEKTORO,
    INODO,
    0,
    len(inodoj),
    1,
    unua_datumo,
    kapacito,
    0,
)

for nodo in inodoj:
    struct.pack_into(
        "<B3xIII",
        bildo,
        SEKTORO + nodo.indekso * INODO,
        nodo.tipo,
        len(nodo.enhavo),
        nodo.unua,
        nodo.sektoroj,
    )
    komenco = nodo.unua * SEKTORO
    bildo[komenco:komenco + len(nodo.enhavo)] = nodo.enhavo

with open(celo, "wb") as dosiero:
    dosiero.write(bildo)

dosieroj = sum(1 for nodo in inodoj if nodo.tipo == TIPO_DOSIERO)
dosierujoj = sum(1 for nodo in inodoj if nodo.tipo == TIPO_DOSIERUJO) - 1
pluralo = "" if dosierujoj == 1 else "j"
print(
    f"SNDS-bildo: {celo} ({dosieroj} dosieroj, {dosierujoj} dosierujo{pluralo}, "
    f"{len(inodoj)} inodoj, {kapacito} sektoroj)"
)
PY
