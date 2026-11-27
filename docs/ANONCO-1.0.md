# MiKerno 1.0 — anonco

> Stabila eldono publikigita la 2026-11-27 kiel `v1.0.0`.
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

La plej lasta stabila eldono estas `v1.0.0`; `v0.9.3` estis ĝia antaŭeldono
por testado. La ISO-oj estas elŝuteblaj de la
[GitHub-Eldono v1.0.0](https://github.com/makoto-lernanto/mikerno/releases/tag/v1.0.0),
aŭ rekonstrueblaj per `just iso` kaj `just iso-arm`. La ŝelo Ligo startas en
`/hejmo` kaj atendas la prompton `ligo:~>`; `helpu` montras la komandojn,
`listi` la SNDS-dosierojn, `iri` kaj `kie` moviĝas tra la arbo (relativaj
vojoj kaj `~` inkluzive) kaj `fini` finas la sesion.

## Eldona kontrolo

M19 finiĝis per la stabila `v1.0.0`. M20 (IOMMU kaj aparata izolado) restas
post 1.0, ĉar ĝi dependas de la PCI/aparata tavolo.

La loka pordego, la kompleta DI-matrico kaj la kvar QEMU-praŝarĝoj sukcesis.
La ISO-oj estis rekonstruitaj el la DI-verda komitero; iliaj SHA-256-valoroj
kaj la akceptaj linioj estas registritaj ĉi tie kaj en
`docs/MEJLOSTONOJ.md`. Ĉi tiu anonco kaj ambaŭ ISO-oj estas en la stabila
Eldono `v1.0.0`.

## Akcepto de v1.0.0

- La puŝita DI ĉe `80c68b5` sukcesis: 62 el 62 taskoj verdaj.
- `just kontroli`: 83 host-testoj; dokumenta kontrolo de 20 sistemvokoj kaj
  52 pruvoj; formato, Clippy kaj la laborloka konstruo sukcesis.
- La `x86_64` ISO finiĝis post 1017 tikoj; la rekta PVH-praŝarĝo post 1024.
- La `arm64` ISO finiĝis post 1020 tikoj; la rekta `Image`-praŝarĝo post 1021.
- Ambaŭ ISO-praŝarĝoj montris Ligon, atingis la finan staton sen `MALBONE`
  kaj havis la saman pikselan haketon `0xecf1f6dbe1421575`.

| ISO | SHA-256 |
| --- | --- |
| `mikerno.iso` (`x86_64`) | `096cbde4bc8a5790bb4455455d80d3ceae626655deba758ddd0de0f73a66aff8` |
| `mikerno-arm.iso` (`arm64`) | `721038a2bf0351c718c3dcd47495d4667f7a8988cfd2242ac1e3b495ad9e87d9` |

## Konataj limigoj

- Neniu `fork`: novaj procezoj venas per `SYS_LANĜI` aŭ `SYS_EXEC`, ne per
  kopiado de adresa spaco.
- Unuprocesora: neniu SMP aŭ CPU-afineco.
- La kernaj stakoj estas statika tabelo de ok blokoj.
- IOMMU-izolado venas en M20.
