# MiKerno 0.9.3 — eldonkandidato al 1.0

> **Antaŭeldono, ne la fina eldono.** `v0.9.3` estas kandidato por `v1.0.0`;
> ĝi ne anstataŭas la finan eldonon de M19 Fazo 5.

M19 Fazo 4 preparas la eldonkandidaton antaŭ la fina etikedo `v1.0.0`.
La versio 0.9.3 konservas la frostigitan ABI 2, la katalogon de 52 Kani-pruvoj
kaj la reviziitan DI-pordegon. La kandidataj ISO-oj enhavas la samajn
`x86_64`- kaj `arm64`-praŝarĝajn vojojn kaj atingas la Horizonto-ŝelon.

## Akcepto

- `just kontroli`: sukcesis; 83 host-testoj, dokumenta kontrolo de 20
  sistemvokoj kaj 52 pruvoj, formato, Clippy kaj la laborloka konstruo.
- La puŝita DI sukcesis: ĉiuj 62 taskoj estas verdaj
  ([run 45392220055](https://github.com/makoto-lernanto/mikerno/actions/runs/45392220055)).
- `x86_64` ISO: `[MiKerno] Saluton, mi estas MiKerno v0.9.3!`;
  `[kerno] fina stato: 7 mortintaj, 0 pretaj, 0 blokitaj; 1020 tikoj`.
- `arm64` ISO: `[MiKerno] Saluton, mi estas MiKerno v0.9.3!`;
  `[kerno] fina stato: 7 mortintaj, 0 pretaj, 0 blokitaj; 1017 tikoj`.
- La rekta `x86_64` PVH-praŝarĝo atingis la finan staton post 1018 tikoj;
  la rekta arm64 `Image`-praŝarĝo atingis ĝin post 1013 tikoj.
- Ambaŭ ISO-oj atingis `Ligo 0.1 — objekta ŝelo de Horizonto · ABI 2`,
  montris la piksela fiksaĵo `0xecf1f6dbe1421575` kaj pasis
  `tools/kontroli-qemu.sh` sen `MALBONE`.

## SHA-256 de la ISO-oj

| ISO | SHA-256 |
| --- | --- |
| `mikerno.iso` | `41f79d17ee44206beb4fe7bcb05d8326fad4cf340821a70f5bad2f3e56c55a48` |
| `mikerno-arm.iso` | `eefab95b908d152adf51f772e35055c2749428b3ebfe2d9dab28b382cb2beccb` |

La ISO-oj estis rekonstruitaj el la puŝita koda komitero `fda45d9` post la
verda DI. Ĉi tiu eldono estas antaŭeldono por testado antaŭ la fina etikedo
`v1.0.0`.
