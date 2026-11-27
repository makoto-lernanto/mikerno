# MiKerno 1.0 — anonco

> Eldona teksto por `v1.0.0`; la stabila publikigo okazos post la puŝita DI,
> rekonstruitaj ISO-oj kaj ilia QEMU-validigo.
>
> Ĝisdatigita por versio: 1.0.0 (Cargo-laborloko)

## Malgranda kerno, plena kontrolo

MiKerno estas mikrokomputila kerno verkita en `no_std` Rust, sen iu ajn
ekstera dependaĵo. Ĝi praŝarĝiĝas sur `x86_64` (Limine/Multiboot2, Xen-PVH aŭ
UEFI) kaj sur `arm64` (plata `Image` aŭ UEFI-ISO), kaj ĝia tuta fonto — kodo,
komentoj, dokumentaro kaj komitaj mesaĝoj — estas en Esperanto.

La celo de 1.0 estas simple esprimebla: uzanta procezo ruliĝu en Ring 3 kun
propra adresa spaco, atingu la kernon nur per malgranda dokumentita
sistemvoka ABI, kaj ne povu legi la kernon aŭ alian procezon.

## Kion alportas 1.0

- **Ring 3-izolado.** `horizonto` kaj la aliaj uzantaj bildoj ruliĝas en Ring 3
  (aŭ EL0) kun privataj paĝtabeloj. La scenaro «viktimo kaj spiono» pruvas, ke
  du procezoj kun la sama virtuala adreso vidas malsamajn fizikajn kadrojn, kaj
  ke la spiono ricevas `#PF` anstataŭ la sekretan paĝon. Ring 3-esceptoj
  (`#PF`, `#GP`, `#DE`, `#OF`, `#UD`, kaj la arm64-abortoj) mortigas nur la
  kulpan procezon.
- **Dokumentita sistemvoka ABI.** Dudek numeroj `SYS_SENDI` (1) ĝis
  `SYS_LANĜI` (20) kovras mesaĝojn, kapablojn, tempolimojn, la konsolon, la
  SNDS-dosiersistemon kaj la procezan ciklon. La numeroj kaj iliaj priskriboj
  estas kontrolataj per `tools/dokumentaro.sh`.
- **Kapabla IK.** Finpunktoj, teniloj kun du-nivela generacia kontrolo,
  rajtoj, revokaj vekoj kaj tempolimoj; purigo ĉe proceza detruo kaj ĉe
  mortiga escepto.
- **Memoro reuzebla.** Kadroj kaj privataj paĝtabeloj liberiĝas je proceza
  detruo; la reuzoprovo komparas la liberajn kadrojn antaŭ kaj post dek
  cikloj kaj postulas saman nombron.
- **Ripetebla pordego.** GitHub Actions konstruas la ISO-ojn, rulas `fmt`,
  `clippy`, la laborlokan konstruon, Miri, 52 Kani-pruvojn kaj kvin
  QEMU-praŝarĝojn (BIOS/UEFI, du memorgrandoj, ambaŭ arkoj).
- **Kompleta dokumentaro.** `README.md`, `docs/MEJLOSTONOJ.md`,
  `docs/VORTARO.md` kaj la sistemvoka referenco spegulas la kodon; la
  dokumenta kontrolilo malkongruon haltigas la pordegon.

## Provu la nunan eldonon

La plej lasta stabila eldono antaŭ `v1.0.0` estas `v0.9.2`; `v0.9.3` estis
antaŭeldono por testado. La ISO-oj estos elŝuteblaj de la
[GitHub-Eldonoj](https://github.com/makoto-lernanto/mikerno/releases), aŭ
rekonstrueblaj per `just iso` kaj `just iso-arm`. La ŝelo Ligo startas en
`/hejmo` kaj atendas la prompton `ligo:~>`; `helpu` montras la komandojn,
`listi` la SNDS-dosierojn, `iri` kaj `kie` moviĝas tra la arbo (relativaj
vojoj kaj `~` inkluzive) kaj `fini` finas la sesion.

## Vojo al la eldono

M19 finigas la formalan pruvaron kaj etikedas 1.0. M20 (IOMMU kaj aparata
izolado) restas post 1.0, ĉar ĝi dependas de la PCI/aparata tavolo.

**Eldona kontrolo por 1.0:**

1. `just kontroli` verda loke (versio, dokumentaj sistemvokoj, unu-testoj,
   `fmt`, ambaŭ `clippy`-vokoj, konstruo).
2. La tuta DI-matrico verda: `kontroli-kaj-konstrui`, `permesiloj-kontroli`,
   `miri-kontroli`, `kani-kontroli`, `paki-iso` kaj la kvin `qemu-kontroli`.
3. Ambaŭ ISO-oj konstruitaj el la puŝita komitero kaj praŝarĝitaj sen
   `MALBONE`.
4. La SHA-256-valoroj kaj la akceptaj linioj registritaj en
   `docs/MEJLOSTONOJ.md` kaj en la Eldono.
5. Ĉi tiu anonco reviziita, datita kaj publikigita kun la etikedo `v1.0.0`.

## Loka akcepto por la kandidata konstruo

- `just kontroli`: sukcesis; 83 host-testoj, 20 dokumentitaj sistemvokoj kaj
  52 Kani-pruvoj, kun formato, Clippy kaj la laborloka konstruo.
- La `x86_64` ISO finiĝis post 1011 tikoj; la rekta PVH-praŝarĝo post 1018.
- La `arm64` ISO finiĝis post 1018 tikoj; la rekta `Image`-praŝarĝo post 1008.
- La ISO-praŝarĝoj montris la ŝelon Ligo, la finan staton sen `MALBONE` kaj la
  saman piksela haketon `0xecf1f6dbe1421575`.

| ISO | SHA-256 (loka konstruo) |
| --- | --- |
| `mikerno.iso` (`x86_64`) | `c73307e1a196124f99d41b41f0d8cfa7b5fafc44c1ae9d741944f8ee3ef0b075` |
| `mikerno-arm.iso` (`arm64`) | `b2ae2f63b3fa60dd286aea0d51b4a6b78e3a0fe9939ecb46981a2ef0dd7a1d30` |

## Konataj limigoj

- Neniu `fork`: novaj procezoj venas per `SYS_LANĜI` aŭ `SYS_EXEC`, ne per
  kopiado de adresa spaco.
- Unuprocesora: neniu SMP aŭ CPU-afineco.
- La kernaj stakoj estas statika tabelo de ok blokoj.
- IOMMU-izolado venas en M20.
