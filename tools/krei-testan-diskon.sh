#!/usr/bin/env bash
# Kreas malgrandan, determinisman testdiskon por virtio-blk: 64 sektoroj po
# 512 bajtoj. La sama enhavo estas uzata de la kerna memprovo.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-$ROOT/target/testdisko.img}"
mkdir -p "$(dirname "$OUT")"

python3 - "$OUT" <<'PY'
import sys

vojo = sys.argv[1]
with open(vojo, "wb") as dosiero:
    for sektoro in range(64):
        dosiero.write(bytes(((sektoro * 37 + indekso * 11) & 0xFF)
                            for indekso in range(512)))
PY

echo "Testdisko: $OUT (64 sektoroj)"
