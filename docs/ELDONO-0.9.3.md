# MiKerno 0.9.3 — eldonkandidato al 1.0

> **Stato:** prepara priskribo; la lokaj ISO-oj ne estas publikigitaj.
> Rekonstrui la ISO-ojn el la puŝita komitero kaj anstataŭigi la haketojn post
> verda DI antaŭ la alŝuto.

M19 Fazo 4 preparas la eldonkandidaton antaŭ la fina etikedo `v1.0.0`.
La versio 0.9.3 konservas la frostigitan ABI 2, la katalogon de 52 Kani-pruvoj
kaj la reviziitan DI-pordegon. La kandidataj ISO-oj enhavas la samajn
`x86_64`- kaj `arm64`-praŝarĝajn vojojn kaj atingas la Horizonto-ŝelon.

## Loka akcepto

- `just kontroli`: sukcesis; 83 host-testoj, dokumenta kontrolo de 20
  sistemvokoj kaj 52 pruvoj, formato, Clippy kaj la laborloka konstruo.
- `x86_64` ISO: `[MiKerno] Saluton, mi estas MiKerno v0.9.3!`;
  `[kerno] fina stato: 7 mortintaj, 0 pretaj, 0 blokitaj; 1022 tikoj`.
- `arm64` ISO: `[MiKerno] Saluton, mi estas MiKerno v0.9.3!`;
  `[kerno] fina stato: 7 mortintaj, 0 pretaj, 0 blokitaj; 1006 tikoj`.
- La rekta `x86_64` PVH-praŝarĝo atingis la finan staton post 1018 tikoj;
  la rekta arm64 `Image`-praŝarĝo atingis ĝin post 1013 tikoj.
- Ambaŭ ISO-oj atingis `Ligo 0.1 — objekta ŝelo de Horizonto · ABI 2`,
  montris la piksela fiksaĵo `0xecf1f6dbe1421575` kaj pasis
  `tools/kontroli-qemu.sh` sen `MALBONE`.

## Loka SHA-256

| ISO | SHA-256 |
| --- | --- |
| `mikerno.iso` | `6e7d6acc3ed2beded8a5e3369a1f694d6f113309dda87a9ce4e12422f4a1b95d` |
| `mikerno-arm.iso` | `a32a35e1e4791737fdc57d683e8035a615d8fd0b27667fd4590237c6abfba965` |

La valoroj supre apartenas al la loka, nepuŝita konstruaĵo. Ili ne estas
eldonaj haketoj: post la puŝo kaj verda DI oni devas rekonstrui, praŝarĝi kaj
rekalkuli ilin antaŭ ol krei aŭ ĝisdatigi la Eldonon.
