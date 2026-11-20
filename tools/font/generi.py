#!/usr/bin/env python3
"""Rasterigu la tiparojn de la frambufa konzolo.

El DejaVu Sans Mono ni produktas kvar memstarajn bitmapajn "strekojn"
(8x16, 12x24, 16x32, 32x64). Ĉiu streko enhavas 117 glifojn en fiksa ordo:
ASCII 0x20..0x7E, poste la Esperantaj literoj, la citiloj kaj la simboloj
`· — → ─ │ ✕`, kiujn uzas la konzolo kaj Ligo.

La eligo estas kruda bita tabelo: por ĉiu glifo, `alto` vicoj, ĉiu vico
`streko = ceil(larĝo/8)` bajtoj, plej supra bito = plej maldekstra pikselo.

Uzado:
    python tools/font/generi.py --font /path/to/DejaVuSansMono.ttf \
        --output mikerno/src/ekrano/fonto

La fontodosiero mem ne estas konservata en la deponejo; la licenco estas
en third_party/dejavu/LICENSE.txt.
"""

from __future__ import annotations

import argparse
import hashlib
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

# (dosiernomo, larĝo, alto, grando, bazlinio, sojlo)
STREKOJ = [
    ("tiparo-8x16.bin", 8, 16, 12, 12, 64),
    ("tiparo-12x24.bin", 12, 24, 19, 18, 96),
    ("tiparo-16x32.bin", 16, 32, 25, 24, 96),
    ("tiparo-32x64.bin", 32, 64, 51, 48, 96),
]

ALDONAJ = "ĉĈĝĜĥĤĵĴŝŜŭŬ«»‹›·—→─│✕"
SIGNOJ = [chr(kodo) for kodo in range(0x20, 0x7F)] + list(ALDONAJ)

KANDIDATOJ = [
    Path("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf"),
    Path("/usr/share/fonts/dejavu/DejaVuSansMono.ttf"),
    Path("/usr/share/fonts/TTF/DejaVuSansMono.ttf"),
    Path.home() / ".local/share/fonts/DejaVuSansMono.ttf",
]
DEFAULT_FONT = next((p for p in KANDIDATOJ if p.is_file()), KANDIDATOJ[0])


def rasterigi(fonto: ImageFont.FreeTypeFont, signo: str, larĝo: int, alto: int,
              bazlinio: int, sojlo: int) -> bytes:
    bildo = Image.new("L", (larĝo, alto), 0)
    ImageDraw.Draw(bildo).text(
        (0, bazlinio), signo, font=fonto, fill=255, anchor="ls"
    )

    streko = (larĝo + 7) // 8
    datumoj = bytearray()
    for y in range(alto):
        for bloko in range(streko):
            bajto = 0
            for bito in range(8):
                x = bloko * 8 + bito
                if x < larĝo and bildo.getpixel((x, y)) >= sojlo:
                    bajto |= 0x80 >> bito
            datumoj.append(bajto)
    konekti_liniojn(signo, datumoj, larĝo, alto)
    return bytes(datumoj)


def konekti_liniojn(signo: str, datumoj: bytearray, larĝo: int, alto: int) -> None:
    """Plenigu la strekon de `─` aŭ `│` ĝis la randoj de la ĉelo.

    La rastrumilo fortranĉas la lastan kolumnon aŭ vicon, ĉar ties kovrado
    estas sub la sojlo; tiam apudaj ĉeloj ne tuŝus unu la alian kaj la linio
    aspektus punktita. Ni elektas la vicojn (respektive kolumnojn), kiuj jam
    estas almenaŭ duone plenaj, kaj plenigas ilin tute.
    """
    streko = (larĝo + 7) // 8

    def bito(x: int, y: int) -> bool:
        return bool(datumoj[y * streko + x // 8] & (0x80 >> (x % 8)))

    def starigi(x: int, y: int) -> None:
        datumoj[y * streko + x // 8] |= 0x80 >> (x % 8)

    if signo == "─":
        for y in range(alto):
            if 2 * sum(bito(x, y) for x in range(larĝo)) >= larĝo:
                for x in range(larĝo):
                    starigi(x, y)
    elif signo == "│":
        for x in range(larĝo):
            if 2 * sum(bito(x, y) for y in range(alto)) >= alto:
                for y in range(alto):
                    starigi(x, y)


def kontroli_klipadon(fonto: ImageFont.FreeTypeFont, larĝo: int, alto: int,
                      bazlinio: int) -> list[str]:
    problemoj = []
    for signo in SIGNOJ:
        x0, y0, x1, y1 = fonto.getbbox(signo, anchor="ls")
        # `─` (U+2500) etendiĝas unu pikselon maldekstren, por ke la linioj
        # tuŝu unu la alian inter la ĉeloj; la rastrumilo fortranĉas tiun
        # kolumnon kaj la resto plenigas la ĉelon.
        maldekstra = -1 if signo == "─" else 0
        if x0 < maldekstra or x1 > larĝo or bazlinio + y0 < 0 or bazlinio + y1 > alto:
            problemoj.append(
                f"{signo!r}: bbox=({x0}, {y0}, {x1}, {y1}) "
                f"kontraŭ {larĝo}x{alto} ĉe bazlinio {bazlinio}"
            )
    return problemoj


def antaŭrigardo(larĝo: int, alto: int, glifoj: bytes, celo: Path) -> None:
    streko = (larĝo + 7) // 8
    kolumnoj = 16
    vicoj = (len(SIGNOJ) + kolumnoj - 1) // kolumnoj
    bildo = Image.new("L", (kolumnoj * larĝo + kolumnoj, vicoj * alto + vicoj), 0)
    for indekso in range(len(SIGNOJ)):
        gx = (indekso % kolumnoj) * (larĝo + 1)
        gy = (indekso // kolumnoj) * (alto + 1)
        bazo = indekso * alto * streko
        for y in range(alto):
            for x in range(larĝo):
                bajto = glifoj[bazo + y * streko + x // 8]
                if bajto & (0x80 >> (x % 8)):
                    bildo.putpixel((gx + x, gy + y), 255)
    bildo.save(celo)


def main() -> int:
    parse = argparse.ArgumentParser(description=__doc__)
    parse.add_argument("--font", type=Path, default=DEFAULT_FONT,
                       help="DejaVuSansMono.ttf")
    parse.add_argument("--output", type=Path,
                       default=Path("mikerno/src/ekrano/fonto"))
    parse.add_argument("--preview-dir", type=Path,
                        help="se donita, skribu antaŭrigardajn PNG-ojn tie")
    argumentoj = parse.parse_args()

    if not argumentoj.font.is_file():
        print(f"tiparo ne trovita: {argumentoj.font}", file=sys.stderr)
        return 1

    haŝo = hashlib.sha256(argumentoj.font.read_bytes()).hexdigest()
    print(f"fontdosiero: {argumentoj.font}")
    print(f"sha256:      {haŝo}")
    argumentoj.output.mkdir(parents=True, exist_ok=True)
    if argumentoj.preview_dir:
        argumentoj.preview_dir.mkdir(parents=True, exist_ok=True)

    for dosiernomo, larĝo, alto, grando, bazlinio, sojlo in STREKOJ:
        fonto = ImageFont.truetype(str(argumentoj.font), grando)
        problemoj = kontroli_klipadon(fonto, larĝo, alto, bazlinio)
        if problemoj:
            print("KLIPADO-DETRENDO:", file=sys.stderr)
            for problemo in problemoj:
                print(f"  {problemo}", file=sys.stderr)
            return 1

        glifoj = b"".join(
            rasterigi(fonto, signo, larĝo, alto, bazlinio, sojlo)
            for signo in SIGNOJ
        )
        celo = argumentoj.output / dosiernomo
        celo.write_bytes(glifoj)
        print(f"{dosiernomo}: {len(glifoj)} bajtoj, {len(SIGNOJ)} glifoj")

        if argumentoj.preview_dir:
            antaŭrigardo(larĝo, alto, glifoj,
                         argumentoj.preview_dir / f"{celo.stem}.png")

    return 0


if __name__ == "__main__":
    sys.exit(main())
