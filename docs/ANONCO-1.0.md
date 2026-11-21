# MiKerno 1.0 — anonco (preparo)

> **Stato:** prepara teksto por la eldono 1.0. La eldono venos kun la mejloŝtono
> M20 (plena pruvaro); ĝis tiam la nuna eldono estas 0.8.4. Ĉi tiu dokumento
> ne estas publikigita anonco.
>
> Ĝisdatigita por versio: 0.8.4 (Cargo-laborloko)

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
  `clippy`, la laborlokan konstruon, Miri, 43 Kani-pruvojn kaj kvin
  QEMU-praŝarĝojn (BIOS/UEFI, du memorgrandoj, ambaŭ arkoj).
- **Kompleta dokumentaro.** `README.md`, `docs/MEJLOSTONOJ.md`,
  `docs/VORTARO.md` kaj la sistemvoka referenco spegulas la kodon; la
  dokumenta kontrolilo malkongruon haltigas la pordegon.

## Provu la nunan eldonon

La nuna eldono 0.8.4 enhavas ĉion supre escepte la lastan formalan pruvaron.
Elŝutu `mikerno.iso` aŭ `mikerno-arm.iso` el la
[GitHub-Eldono](https://github.com/makoto-lernanto/mikerno/releases), aŭ
konstruu loke per `just iso` kaj `just iso-arm`. La ŝelo Ligo atendas la
prompton `ligo:~>`; `helpu` montras la komandojn, `listi` la SNDS-dosierojn kaj
`fini` finas la sesion.

## Vojo al la eldono

M20 finigas la formalan pruvaron kaj etikedas 1.0. M19 (IOMMU kaj aparata
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

## Konataj limigoj

- Neniu `fork`: novaj procezoj venas per `SYS_LANĜI` aŭ `SYS_EXEC`, ne per
  kopiado de adresa spaco.
- Unuprocesora: neniu SMP, afineco aŭ prioritatoj.
- La kernaj stakoj estas statika tabelo de ok blokoj.
- IOMMU-izolado venas en M19.
