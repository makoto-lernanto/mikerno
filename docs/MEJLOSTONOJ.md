# Mejloŝtonoj de MiKerno

> Stato: 2026-11-19 · Versio de la Cargo-laborloko: 0.7.4
>
> Ĉi tiu dokumento registras kion MiKerno jam atingis, per kio tio estas
> pruvita, kaj kio venas poste. Ĝi estas la inĝeniera taglibro de la projekto,
> dum [README](../README.md) priskribas la nunan staton.

---

## 1. Superrigardo

| ID | Mejloŝtono | Stato | Ĉefa pruvo |
| --- | --- | --- | --- |
| M0 | Laborloko, tri kestoj, konstrua ĉeno | Plenumita | `cargo build --workspace --release` |
| M1 | Du praŝarĝaj protokoloj kaj 32→64-bita eniro | Plenumita | QEMU PVH + Limine/ISO startoj |
| M2 | Praŝarĝa memora mapo kaj bitmapa kadro-alokilo | Plenumita | 7 regionoj, 255 MiB, funkciantaj paĝtabeloj |
| M3 | 4-nivela paĝado kaj propra adresa spaco | Plenumita | CR3-ŝalto, frambufa mapado |
| M4 | GDT, IDT, TSS kaj escepta raportado | Plenumita | sen neatenditaj esceptoj; 100 Hz-vektoro |
| M5 | xAPIC kaj PIT-kalibrita temporizilo | Plenumita | 1000 tikoj en ≈10 s |
| M6 | Frambufo-konzolo kaj COM1 | Plenumita | `1280x800, 32 bitoj/pikselo` (ISO) |
| M7 | Antaŭokupa planisto kaj kuntekstŝanĝo | Plenumita | 3 fadenoj interplektitaj kaj finitaj |
| M8 | Mesaĝa IK kun kapabloj | Plenumita | mesaĝoj liveritaj, fadeno 3 vekita |
| M9 | Kvalita pordego: fmt, clippy, Kani-pruvoj | Parte | fmt/clippy verdaj; 26 pruvoj maŝine plenumitaj kaj verdaj |
| M10 | Uzantspaca kontrakto (`libmikerno`, `horizonto`) | Plenumita | ABI difinita; `horizonto` plenumiĝas en Ring 3 |
| M11 | Ring 3 kaj sistemvokoj | Plenumita | `int 0x80` el Ring 0 kaj Ring 3, negativaj kazoj, ambaŭ praŝarĝaj vojoj |
| M12 | Per-procesaj adresspacoj | Plenumita | W^X laŭ sekcioj, `CR0.WP`, proceza izolado (spiono `#PF`, viktimo pluvivas), egalaj kadroj |
| M13 | Kerna memoro-firmigo: SMEP, SMAP, KASLR | Plenumita | SMEP, SMAP kaj KASLR pruvitaj per Ring 0-provoj kaj tri startoj |
| M14 | IK-firmigo: objektoj, rajtoj, tempolimoj | Plenumita | Ring 0/Ring 3-scenaroj kaj AC/CR3/EOI-provoj verdaj (ABI 2) |
| M15 | `arm64`-malantaŭo: EL0, ABI 2, `ramfb`, `PAN` | Plenumita | identa piksela haketo sur ambaŭ arkoj |
| M16 | Dormantaj temporiziloj, prioritatoj, SMP-esploro | Parte | dormo 100 ms = 10 tikoj; minimumo ≥ 17 veturoj; arm64-stabileco |
| M17 | Konservado, dosiersistemo, ŝelo | Plenumita | `virtio-blk` legas/skribas 64 sektorojn; SNDS (6 dosieroj, 1 dosierujo; PCI kaj MMIO); `SYS_BRK`, `SYS_EXEC` kaj `SYS_LANĜI` en Ring 3; Ligo kun `listi`, `procezoj`, `mortigi`, `ruli`; `virtio-blk` MMIO-vojo funkcias sur `arm64`; `#[allow]`-purgado por M17 plenigita |
| M18 | DI, testoj, plena pruvaro | Parte | GitHub Actions verde (fmt, clippy, konstruo, Kani); testoj kaj plena pruvaro mankas |
| M19 | Aparata izolado: IOMMU, PCI/DMA | Post 1.0 | — |
| M20 | Plena pruvaro kaj 1.0 | Venonta | — |

### 1.1. La `arm64`-ŝpuro (A0–A4), kunfandita en M15

`arm64` disvolviĝis kiel aparta ŝpuro kun fazoj A0–A4. Ĝi estis kunfandita en
`main` la 2026-11-15 kaj fariĝis la mejloŝtono M15 (`0.5.0`); la antaŭe planitaj
M15–M18 translokiĝis al M16–M19. La plano, la kouplingo al `x86_64` kaj la cela
dosieruja strukturo estas en [ARM.md](ARM.md). Dum la ŝpuro la versio estis
antaŭeldono (`0.4.3-arm.N`).

A0 (arkitektura refaktoro, sen kondutŝanĝo) estas plenumita la 2026-11-14 en
`0.4.3`: `Kunteksto`, `InterrompaKadro` kaj la escepta klasifiko translokiĝis
al la malantaŭo; `SistemvokaKadro`, `FirmigaRaporto` kaj `ELF_MAŜINO` eniris la
fasadon; `privataj_pdptoj` fariĝis `privataj_subtabeloj`. `just kontroli`, la
PVH-vojo kaj la ISO-vojo restas verdaj.

A1 (arm64-skeleto) estas plenumita la 2026-11-14 en `0.4.3-arm.1`: plata
`Image` kun la 64-bajta kapo, EL2→EL1 per `HCR_EL2.RW`, PL011-konzolo,
minimuma FDT-analizilo (memoro, initrd, seria adreso), identeca MMU per
L0/L1-tabeloj kaj PSCI `SYSTEM_OFF`. `just ruli-arm` presas la version, la
memoran linion kaj la serian konzolon, kaj paniko haltigas la maŝinon per
PSCI. `just iso-arm` aldone kunmetas UEFI-booteblan `mikerno-arm.iso` per
Limine 12.9.6 (`BOOTAA64.EFI`); sub UEFI la memora linio venas el la
UEFI-memormapo en `/chosen`, do ĝi raportas konvenan memoron (229 MiB el
256 MiB). La versio estas presata de la kerno, same kiel ĉe `x86_64`. La
`x86_64`-konstruo kaj ties scenaro restas senŝanĝaj.

A2 (arm64) estas plenumita la 2026-11-14 en `0.4.3-arm.2`: la vektora tabelo
`VBAR_EL1` (16 eniroj po 128 bajtoj) kun plena kadro `x0`–`x30`, `ELR_EL1`,
`SPSR_EL1`, `ESR_EL1` kaj `FAR_EL1`; GICv2 (distribuilo `0x0800_0000`,
procesora interfaco `0x0801_0000`, PPI 30, EOI antaŭ la traktilo); la ĝenerala
temporizilo per `CNTFRQ_EL0`/`CNTP_TVAL_EL0` je 100 Hz; `DAIF`-gardo; kaj la
kuntekstŝanĝo de la planisto (`x19`–`x30`, `sp`, `DAIF`, unua eniro per
trampolino). La komuna `kerno::planisto` kaj `interrompo` nun kompiliĝas sur
ambaŭ arkoj per `cfg`-gardoj, sen ŝanĝi la `x86_64`-konduton. La enira stako
translokiĝis ekster `.bss`, do la nula buklo de la eniro ne tuŝas ĝin.
Akcepto (`just ruli-arm`):

```text
[horloĝo] tiko 100
[fadeno 1] 100 tikoj
…
[fadeno 3] vekis fadenon 2 ĉe 501 tikoj
[fadeno 2] vekita ĉe 502 tikoj
…
[fadeno 1] finiĝis post 1000 tikoj
[kerno] fina stato: 3 mortintaj, 0 pretaj, 0 blokitaj; 1000 tikoj
```

A3 (arm64) estas plenumita la 2026-11-15 en `0.4.3-arm.3`: `uzanta.rs` eniras
`EL0` per `eret` (`SPSR_EL1 = EL0t`, `SP_EL0 = stakpinto`, `x0`/`x1` =
reĝimo/identigilo), `sistemvoko.rs` liveras `svc #0` kun `x8` kiel voknumero
kaj `x0`–`x3` kiel argumentoj, kaj la vektora ŝelko dispetĉas `SVC64` al la
komuna `kerno::sistemvoko::voki`. La per-procezaj spacoj havas propran
`TTBR0_EL1`: la kernaj identecaj eroj (`0..4 GiB`) restas kunhavataj, la
uzanta fenestro ricevas privatajn `L2`/`L3`-tabelojn, kaj `horizonto`
konstruiĝas por `aarch64-unknown-none` (ELF-maŝino `183`). La komuna kerno
raportas `TTBR0`-radikon, `EL0t`-eniron kaj uzantajn datenajn abortojn; la
frambufo kaj la APIC/SMAP-provoj restas nur `x86_64`. La `horizonto`-bildo
liveriĝas kiel `-initrd` (`-kernel`-vojo) aŭ kiel ISO-modulo
(`module_path: boot():/horizonto.elf`). Akcepto — la samaj IK-linioj kiel
`x86_64`, inkluzive `-32`, la sekretan paĝon kaj la spionan datenan aborton:

```text
[kerno] horizonto 4 pretas: EL0t, SP_EL0=0x101000000, SPSR=0x0, enirpunkto 0x1000006ec
[eraro] procezo 5: datena aborto ĉe rip=0x0000000100000400, far=0x0000000200001000, esr=0x92000007; la procezo estas mortigata
[memoro] izolado (bone): 64998 kadroj antaŭ, 64998 post
[kerno] fina stato: 5 mortintaj, 0 pretaj, 0 blokitaj; 1000 tikoj
```

Sub UEFI la kadraj nombroj estas pli malgrandaj (`58465`) pro la memoro, kiun
la firmvaro rezervas; la ceteraj linioj kongruas. La `x86_64`-konstruo kaj
ties scenaro restas senŝanĝaj, kaj `fmt`, `clippy` (ambaŭ celoj) kaj
`cargo build --workspace --release` estas verdaj.

A4 (arm64) estas plenumita la 2026-11-15 en `0.4.3-arm.4`: `fw_cfg.rs` legas
la dosierujon de `fw_cfg`, trovas `etc/ramfb` kaj skribas per DMA 28-bajtan
`RAMFBCfg` (1280×800, `XR24`, paŝo 5120) al kontiguaj, identece mapitaj
kadroj; `ekrano` liveras la pikselan fiksaĵon (`testi_pikseloj`: gradiento,
teksto kaj FNV-1a 64-haketo); `PXN`/`UXN` kaj `PAN` estas ŝaltataj, kaj la
vektora ŝelko raportas `PAN`-malobservon. La FDT-analizilo nuligas la
`reg`-staton ĉe ĉiu nodkomenco, do la PL011 ne plu heredas la `fw_cfg`-adreson.
La akcepto estas la identa frambufa haketo sur ambaŭ arkoj:

```text
[kerno] PAN subtenata: jes
[kerno] frambufo 1280x800, 32 bitoj/pikselo
[ekrano] piksela fiksaĵo 1280x800, 32 bitoj/pikselo, tiparo 12x24; haketo 0xecf1f6dbe1421575
[eraro] PAN-malobservo: EL1 provis legi uzantan paĝon ĉe 0x0000000100000000 (esr=0x9600000f)
```

`just ruli-arm` (plata `Image`) kaj `just ruli-iso-arm` (Limine + AAVMF) liveras
la saman haketon kiel la `x86_64`-ISO. La ISO-vojo implicite uzas
`cortex-a76`, ĉar AAVMF 2025.11 blokiĝas sub TCG kun `-cpu max`.

### 1.2. Grando de la Ring 0-kodo

`bash tools/kodlinioj.sh --linio` mezuras la Ring 0-fonton (`mikerno/src`,
`*.rs`) antaŭ ĉiu komito. «Kodo» malatentas malplenajn kaj komentajn liniojn.

| Dato | Versio | Dosieroj | Linioj | Kodo | Komento | Malplena |
| --- | --- | --- | --- | --- | --- | --- |
| 2026-11-14 | 0.4.3-arm.1 | 41 | 8792 | 6780 | 1143 | 869 |
| 2026-11-14 | 0.4.3-arm.2 | 45 | 9511 | 7343 | 1224 | 944 |
| 2026-11-15 | 0.4.3-arm.3 | 47 | 10238 | 7903 | 1334 | 1001 |
| 2026-11-15 | 0.4.3-arm.4 | 48 | 10731 | 8239 | 1455 | 1037 |
| 2026-11-15 | 0.5.0 | 48 | 11818 | 9048 | 1669 | 1101 |
| 2026-11-16 | 0.6.0 | 49 | 12167 | 9334 | 1704 | 1129 |
| 2026-11-16 | 0.6.0 | 49 | 12167 | 9333 | 1705 | 1129 |
| 2026-11-16 | 0.6.0 | 49 | 12172 | 9331 | 1710 | 1131 |
| 2026-11-16 | 0.6.0 | 49 | 12172 | 9331 | 1710 | 1131 |
| 2026-11-16 | 0.6.1 | 49 | 12394 | 9543 | 1703 | 1148 |
| 2026-11-16 | 0.6.1 | 49 | 12394 | 9543 | 1703 | 1148 |
| 2026-11-16 | 0.6.1 | 49 | 12394 | 9543 | 1703 | 1148 |
| 2026-11-16 | 0.6.2 | 49 | 12536 | 9673 | 1704 | 1159 |
| 2026-11-16 | 0.6.2 | 49 | 12554 | 9687 | 1707 | 1160 |
| 2026-11-16 | 0.6.2 | 49 | 12554 | 9687 | 1707 | 1160 |
| 2026-11-16 | 0.7.0 | 54 | 13568 | 10549 | 1753 | 1266 |
| 2026-11-17 | 0.7.0 | 54 | 13568 | 10549 | 1753 | 1266 |
| 2026-11-17 | 0.7.0 | 54 | 13568 | 10549 | 1753 | 1266 |
| 2026-11-18 | 0.7.1 | 55 | 14043 | 10972 | 1771 | 1300 |
| 2026-11-18 | 0.7.1 | 55 | 14042 | 10971 | 1771 | 1300 |
| 2026-11-18 | 0.7.1 | 55 | 14042 | 10971 | 1771 | 1300 |
| 2026-11-19 | 0.7.2 | 56 | 14345 | 11204 | 1818 | 1323 |
| 2026-11-19 | 0.7.2 | 56 | 14345 | 11204 | 1818 | 1323 |
| 2026-11-19 | 0.7.2 | 56 | 14345 | 11204 | 1818 | 1323 |
| 2026-11-19 | 0.7.3 | 56 | 14977 | 11705 | 1915 | 1357 |
| 2026-11-19 | 0.7.3 | 56 | 14977 | 11705 | 1915 | 1357 |
| 2026-11-19 | 0.7.3 | 56 | 14977 | 11705 | 1915 | 1357 |
| 2026-11-20 | 0.7.4 | 56 | 14988 | 11727 | 1904 | 1357 |

---

## 2. Plenumitaj mejloŝtonoj

### M0 — Laborloko kaj konstrua ĉeno

**Celo.** Unu Cargo-laborloko kun klara apartigo inter kerno, komuna ABI kaj
uzantspaco, riproduktebla per unu komando.

**Efektivigo.**

- `Cargo.toml` — laborloko kun `mikerno`, `libmikerno`, `horizonto`;
  `edition = "2024"`, `rust-version = "1.85"` (altigita al `1.100` la
  2026-11-13; vidu §4.5), `resolver = "2"`.
- `.cargo/config.toml` — cela triopo `x86_64-unknown-none`, `rust-lld`,
  `relocation-model=pic`, kaj la agordo `runner` al `bash tools/run-qemu.sh` (WSL2/Linux).
- `justfile` — `konstrui-ciujn`, `konstrui-kerno`, `konstrui`, `ruli`,
  `iso`, `ruli-iso`, `kontroli`.
- `build.rs` en la kerno kaj en `horizonto` — ĉiu kesto liveras sian propran
  `linker.ld`, do la kerna skripto ne aplikiĝas al la uzanta bildo.
- `unexpected_cfgs` permesas `cfg(kani)` sen avertoj.

**Pruvo.** `cargo build --workspace --release` finiĝas sen avertoj; la kerno
estas 72 KiB, `horizonto` 12 KiB.

**Resto.** Neniu DI; la konstrua ĉeno estas permana.

---

### M1 — Du praŝarĝaj protokoloj kaj longa reĝimo

**Celo.** La sama kerna bildo ŝarĝiĝu kaj per `qemu -kernel` (Xen-PVH) kaj per
norma ISO (Limine + Multiboot2).

**Efektivigo.** `arkitekturo/x86_64/eniro.rs`:

- Multiboot2-kapo petas memormapon (etikedo 6) kaj 32-bitan frambufon
  (etikedo 5); magio `0x36d7_6289`.
- Xen-PVH-noto `XEN_ELFNOTE_PHYS32_ENTRY`; la eniro ricevas
  `hvm_start_info` en `EBX`; magio `0x336e_c578`.
- 32-bita ŝelko: nuligo de 6 paĝoj, identeca mapo de 4 GiB per 2 MiB paĝoj,
  `CR3`, `CR4.PAE`, `EFER.LME`, `CR0.PG|PE`, GDT, malproksima salto al 64-bita
  kodo.
- `.boot_bss` (paĝtabeloj + 16 KiB stako) estas aparta de `.bss`, ĉar la
  longreĝima eniro nuligas `.bss` uzante tiujn paĝojn.
- La liga skripto metas la bildon je 1 MiB.

**Pruvo.** Ambaŭ vojoj printas la saluton kaj plenumas la plenan testan
scenaron; vidu §4.

**Resto.** La enirŝelko estas 32-bita pro komprenebleco, ne pro neceso; EFI
memstara eniro (sen Limine) ne ekzistas.

---

### M2 — Memora mapo kaj kadro-alokilo

**Celo.** Akiri la fizikan memoran mapon de la praŝarĝilo kaj provizi
kadrojn al la paĝado.

**Efektivigo.**

- `memoro/mod.rs` — regionoj, tipoj (`Uzebla`, `Rezervita`, `Acpi`, `Nvs`,
  `Neuzebla`, `Malŝaltita`, `Pmem`, `Nekonata`), `MemoraMapo`, `EniraInformo`,
  `EniraEraro`.
- `mb2.rs` / `pvh.rs` — analizo de ambaŭ informaj strukturoj, inkluzive de
  memmapo, frambufo, RSDP kaj rezervendaj regionoj.
- `memoro/kadro.rs` — bitaro por 4 GiB (1 048 576 kadroj, 128 KiB da stato).
  Ĉio estas komence okupata; la uzeblaj regionoj estas malmarkitaj; la malalta
  1 MiB, la kerna bildo kaj la praŝarĝaj strukturoj restas rezervitaj.
- `Kadro` portas nur indekson; `aloki()` skanas de monotona `sekva` kaj neniam
  redonas la saman kadron.

**Pruvo.** QEMU raportas 7 regionojn kaj 255 MiB uzeblajn; la paĝtabeloj kaj
frambufa mapado sukcese alokas kadrojn.

**Resto.** Neniu liberigo/Reuzo; neniu NUMA, neniu regiona agordo.

---

### M3 — 4-nivela paĝado kun propra adresa spaco

**Celo.** Anstataŭigi la 2 MiB-identecmapon de la ŝelko per detalaj 4 KiB-paĝoj
kaj ŝalti proprietan `CR3`.

**Efektivigo.** `arkitekturo/x86_64/pagado.rs`:

- `X86AdresaSpaco` posedas radikan kadron; `krei`, `ŝalti`, `mapigi`, `traduki`.
- `PML4 → PDPT → PD → PT`; subaj tabeloj estas alokitaj kaj nuligitaj laŭbezone.
- `traduki` komprenas kaj 4 KiB foliojn kaj 2 MiB grandegajn paĝojn.
- `mapigi` rifuzas nealiniiĝintajn adresojn kaj jam mapitajn foliojn.
- Permesoj (0.2.5): `KERNA_KODO` (R+X), `KERNA_LEGA` (R+NX, ankaŭ por
  `.rodata` kaj moduloj), `KERNA_DATUMOJ` (RW+NX), `SKRIBA` (APIC-MMIO),
  `SKRIBA_SENKAŜE` (frambufo, `PCD`); `CR0.WP` malpermesas skribon al
  nurlegaj folioj ankaŭ el Ring 0.
- `EFER.NXE` estas ŝaltita en `pravalorizi`.

**Pruvo.** Post `CR3`-ŝalto la kerno daŭre uzas la frambufon (ISO) kaj la
APIC-on; neniu paĝa eraro aperas dum la tuta testa scenaro.

**Resto.** La identeca mapo jam respektas W^X laŭ sekcioj (0.2.5), SMEP
(0.3.0) kaj SMAP (0.3.1) baras Ring 0 de uzantaj folioj, kaj KASLR (0.3.2)
hazardigas la kernan bazadreson per ET_DYN-relokado; la per-procesa mapado
venis en M12. IOMMU-izolado restas en M19.

---

### M4 — GDT, IDT, TSS kaj esceptoj

**Celo.** Difini la privilegiajn tablojn por longa reĝimo kaj ricevi
kompreneblajn esceptojn.

**Efektivigo.**

- `gdt.rs` — nula ero, kerna kodo `0x08` (L=1), datumoj `0x10`, TSS `0x18`;
  reŝargo de `CS` per `retfq`.
- `idt.rs` — 256 vektoroj, apartaj ŝablonoj por vektoroj kun erarkodo (8,
  10–14, 17, 21, 29, 30), komuna konservo de 15+1 registroj, Rust-dispetĉo,
  `iretq`. La kongruo kun `InterrompaKadro` estas `const`-asertita.
- `tss.rs` — 104-bajta TSS, `RSP0` al 16 KiB kerna stako, `ltr`.
- `interrompo/mod.rs` — registrado de traktiloj, `InterrompaGardo` por
  `RFLAGS.IF`, escepta raporto kun `CR2` por paĝa eraro.
- La aparata `EOI` estas sendita antaŭ la traktilo (por vektoroj ≥ `0x20`),
  ĉar traktilo rajtas ŝanĝi la kuntekston.

**Pruvo.** Neniu escepto aperas dum la starto; la temporizila vektoro `0x20`
funkcias; `[tss] TR ŝargita` estas printita.

**Resto.** Esceptoj haltigas la sistemon; ne ekzistas rekaptebla paĝa eraro,
demando-paĝado aŭ uzantspaca escepta izolado.

---

### M5 — xAPIC kaj temporizilo

**Celo.** Anstataŭigi la malnovan 8259-PIC-on per la loka APIC kaj havigi
antaŭokupan tikadon.

**Efektivigo.** `apic.rs`, `temporizilo.rs`:

- maski 8259 (`0x21`, `0xA1`), legi `IA32_APIC_BASE`, mapi la MMIO-on,
  ŝalti la APIC-on kun spuria vektoro `0xFF`;
- 100 Hz perioda temporizilo, divido 16;
- kalibrado de la APIC-buso kontraŭ PIT-kanalo 2 dum ≈50 ms; se ĝi
  malsukcesas, uzi la norman 100 MHz-buson (`6 250 000` kalkuloj/s);
- la traktilo nombradas tikojn kaj vokas `planisto::planumi()`.

**Pruvo.** QEMU raportis ≈62,6M kalkulojn/s; `[horloĝo] tiko 1000` kaj
`fina stato ... 1000 tikoj` konfirmas 100 Hz (≈10 s).

**Resto.** Neniu unuopa aŭ dormanta temporizilo, neniu TSC, neniu HPET, neniu
SMP-loka APIC.

---

### M6 — Frambufo-konzolo kaj COM1

**Celo.** Legi la frambufon de Multiboot2 kaj prezenti tekston sen
eksteraj tiparoj.

**Efektivigo.**

- `seria.rs` — COM1, 115200 8N1, FIFO, `\r\n`.
- `ekrano/mod.rs` — 32-bita frambufa konzolo, bitmapaj glifoj desegnitaj pikselo post
  pikselo, kolormaskoj el la frambufa informo, titolo
  «MiKerno: La estonteco nun.», malplenigo ĉe la fundo.
- `ekrano/fonto.rs` kaj `ekrano/fonto/*.bin` — kvar antaŭe rasterigitaj strekoj
  (8×16, 12×24, 16×32, 32×64) derivitaj el **DejaVu Sans Mono**. Ĉiu streko
  havas 111 glifojn: 95 ASCII, la dek du ĉapelitaj Esperantaj literoj (`ĉ Ĉ ĝ Ĝ
  ĥ Ĥ ĵ Ĵ ŝ Ŝ ŭ Ŭ`) kaj la citiloj `« » ‹ ›`. `glifo(tiparo, signo)` liveras
  la glifon; nur signoj mankantaj en la tiparo fariĝas `?`.
- `ekrano/mod.rs::elekti_tiparon` elektas la plej grandan strekon, kiu donas
  almenaŭ 80 kolumnojn kaj 24 vicojn, kun nombro da vicoj plej proksima al 33;
  tiel la teksto ne estas skalata kaj restas akra ĉe ĉiu kutima distingivo.
- La frambufo kaj COM1 nun uzas la saman UTF-8-tekston.
- La strekoj estas generataj per `tools/font/generi.py` (Python 3 + Pillow);
  la DejaVu-permesilo estas en `third_party/dejavu/LICENSE.txt`.

**Pruvo.** La ISO-starto raportas `[kerno] frambufo 1280x800, 32 bitoj/pikselo` kaj la
kerno uzas ĝin sen eraro; laŭ la elekta regulo tio donas 12×24 (106×33). La
2026-11-12 piksela kontrolo per `screendump` de QEMU konfirmis, ke la ĉapelitaj
literoj kaj la kvar citiloj desegniĝas sen interkovro; la glifoj restas 1-bitaj
bitmapoj en 32-bita frambufo sen kontraŭaliasingo. La generilo rifuzas
produkti strekon, se iu glifo elirus sian ĉelon.

**Resto.** Nur 32 bitoj/pikselo, malplenigo anstataŭ rulumo, neniu kursoro aŭ koloraj
atributoj; ne-ASCII-signoj ekster la 16 aldonoj fariĝas `?`.

---

### M7 — Antaŭokupa planisto

**Celo.** Plenumi plurajn fadenojn kun antaŭokupa rondiga planado kaj
kunlabora cedo.

**Efektivigo.** `kerno/planisto.rs`, `kerno/objekto.rs`,
`arkitekturo/x86_64/planado.rs`:

- 8 fadenoj + ripoza fadeno `0`; `Preta`, `Kuranta`, `Blokitaj`, `Mortinta`;
- cirkla preta vico; 10 ms-tikoj;
- kuntekstŝanĝo konservas 18 registreojn; la unua eniro uzas
  `fadeno_trampolino`, kiu ŝaltas `IF` kaj transdonas la id-on;
- `cedi`, `bloki`, `veki`, `morti`, `lanĉi`;
- la ripoza kunteksto konservas sian interrompan kadron sur la stako, do ĝi
  povas reveni tra la dispetĉo kaj `iretq`.

**Pruvo.** La protokolo montras interplektitajn liniojn de la tri fadenoj,
ĉiuj tri finiĝas, la planisto revenas al la ripoza kunteksto, kaj la fina
raporto estas `3 mortintaj, 0 pretaj, 0 blokitaj`.

**Resto.** Neniu prioritato, kvanto, afineco aŭ dormanta stato; fadenoj estas
statikaj kaj havas fiksan id-spacon.

---

### M8 — Mesaĝa IK kun kapabloj

**Celo.** Provizi minimuman, kapablo-bazan mesaĝadon inter fadenoj.

**Efektivigo.** `kerno/ik.rs`:

- `FinaPunkto` kaj `Kapablo`; permesoj `SENDI`, `RICEVI`, `ĈIUJ`;
- `Mesago` kun `tipo` kaj 64-bajta kopiata datumaro;
- unu vico de 16 mesaĝoj por fadeno, FIFO inter la sama fonto;
- `sendi_mesagon` vekas blokitan celon; `ricevi_mesagon(fonto)` blokas;
- ĉiuj operacioj sub `InterrompaGardo`;
- 6 Kani-pruvoj pri la mesaĝujo (enhavo en limoj, plena vico, FIFO, malsamaj
  fontoj, malplena vico, invarianto post forpreno).

**Pruvo.** Fadeno 2 ricevas la praŝarĝan mesaĝon; fadeno 1 sendas al fadeno 3,
kiu vekiĝas kaj printas «vekiĝu, fadeno 3».

**Resto.** Finpunkta id == fadena id; ne ekzistas kapablotabelo, revoko,
tempolimoj, sendblokado aŭ apartaj kernaj objektoj.

---

### M10 — Uzantspaca kontrakto

**Celo.** Fiksi la ABI antaŭ ol efektivigi Ring 3.

**Havigita.**

- `libmikerno` — `no_std`, sen dependaĵoj, `#[repr(C)]` mesaĝoj, kapabloj,
  permesoj, sistemvokaj numeroj (`SENDI=1`, `RICEVI=2`, `SKRIBI=3`, `FINI=4`,
  `CEDI=5`) kaj la registra ABI (`rax`, `rdi`, `rsi`, `rdx`, `r10`,
  `int 0x80`).
- `horizonto` — unua uzanta bildo, ligita super 4 GiB (`0x1_0000_0000`), kun
  `horizonto_komenco`, maldika `biblioteko` kaj propra `linker.ld`.
- La kerno liveras `horizonto` kiel praŝarĝan modulon (Multiboot2 tipo 3 aŭ
  PVH-modlisto) kaj plenumas ĝin en Ring 3 (vidu M11).

---

### M11 — Ring 3 kaj sistemvokoj

**Celo.** Lanĉi `horizonto` en Ring 3 kaj plenumi la unuan sistemvokon.

**Havigita.**

1. GDT/TSS-aranĝo: kerna kodo `0x08`, kernaj datumoj `0x10`, uzantaj datumoj
   DPL3 `0x18`, uzanta kodo DPL3 `0x20`, TSS `0x28`/`0x30` (16 bajtoj).
   `iretq` uzas `CS=0x23` kaj `SS=0x1B`; la aranĝo taŭgas ankaŭ por estonta
   `syscall`/`sysret`.
2. `int 0x80`-pordego kun DPL3 (interrompa pordego `0xEE`), ŝanĝebla interrompa
   kadro kaj EOI limigita al la vektoroj `0x20..0x80`.
3. Sistema dispetĉo kaj la kvin vokoj `SENDI`, `RICEVI`, `SKRIBI`, `FINI`,
   `CEDI`; unueca `Mesago` el `libmikerno`; montrila kontrolo de uzantaj
   bufroj (bildo kaj stako); rezulto en `rax`; `SYS_FINI` neniam revenas.
4. `horizonto` liverata kiel Multiboot2-modulo (tipo 3) aŭ per la PVH-modlisto;
   la ELF64 `PT_LOAD`-segmentoj estas muntitaj super 4 GiB (bazo
   `0x1_0000_0000`), kun 64 KiB da uzanta stako je bazo + 16 MiB kaj la bito
   `U` tra ĉiuj paĝtabelaj niveloj.
5. Transiro per `iretq` (`CS=0x23`, `SS=0x1B`, `RFLAGS=0x202`); per-fadena
   `TSS.RSP0` restis M12-tasko (plenumita en 0.2.2).
6. Ambaŭ praŝarĝaj vojoj kaj la negativaj kazoj el Ring 3 verifitaj.

**Progreso.**

| Dato | Tasko | Versio | Pruvo |
| --- | --- | --- | --- |
| 2026-11-12 | 1. GDT/TSS-aranĝo, `VORTARO.md` | 0.1.0 | QEMU PVH: `TR ŝargita`, neniuj esceptoj |
| 2026-11-12 | 2. DPL3-pordego, ŝanĝebla kadro, EOI-limigo | 0.1.1 | Ring 0-memtesto: konata `-38`, nekonata `-22` |
| 2026-11-12 | 3. Dispetĉo, kvin sistemvokoj, unueca `Mesago`, montrila kontrolo | 0.1.2 | Ring 0-memtesto: `skribo=0`, `nekonata=-2`, `nula=-1`, `troa=-1`, `cedo=0`; fina stato `3 mortintaj` |
| 2026-11-12 | 4. PVH/Multiboot2-modulo, ELF64-ŝargo super 4 GiB, stako, bito `U` | 0.1.3 | Ambaŭ vojoj: bildo `0x100000000..0x10000102c`, enirpunkto `0x100000020`, 8/8 kontroloj `bone`, fina stato `3 mortintaj` |
| 2026-11-12 | 5. `iretq` (`CS=0x23`, `SS=0x1B`, `RFLAGS=0x202`), uzanta fadeno | 0.1.4 | Ambaŭ vojoj: `horizonto: la unua procezo vekis en Ring 3.`, `[planisto] fadeno 4 finiĝis`, fina stato `4 mortintaj`; 0 esceptoj |
| 2026-11-12 | 6. Ambaŭ vojoj kaj negativaj kazoj el Ring 3 | 0.1.5 | `[ring3] negativaj: nekonata=-2, nula=-1, kerna=-1, troa=-1`; bildo `0x100000000..0x100001082`, enirpunkto `0x1000000d0`; 0 esceptoj |

**Akceptkriterioj — pruvitaj.**

- `horizonto` printas `horizonto: la unua procezo vekis en Ring 3.` kaj eliras
  per `fini(0)` sen escepto.
- Post la eliro la planisto revenas al la ripoza kunteksto kaj raportas
  `4 mortintaj, 0 pretaj, 0 blokitaj`.
- `int 0x80` el Ring 3 (`SKRIBI`, `FINI`, negativaj kazoj) kaj el Ring 0
  (memtesto) funkcias.
- Neniu paĝa eraro aŭ `#GP` dum la tuta scenaro (0 `escepto`-linioj).

---

### M12 — Per-procesaj adresspacoj

**Celo.** Izoli uzantajn procezojn: ĉiu procezo havu privatan `PML4`-on, la
kerno restu kunhavata, kaj neniu procezo povu legi la bildon aŭ la datumojn de
alia.

**Taskoj.** Nova `PML4` por ĉiu procezo; kopii la kernan duonon; mapi uzantajn
foliojn nur en la propra spaco; ŝalti `CR3` dum kuntekstŝanĝo; validigi
uzantajn montrilojn; malmapi kaj liberigi kadrojn ĉe eliro.

**Progreso.**

| Dato | Fazo | Versio | Pruvo |
| --- | --- | --- | --- |
| 2026-11-12 | 1. `liberigi`, kaskada `detruu`, provo ĉe starto | 0.2.0 | PVH: `[memoro] liberigo (bone): 65148 kadroj antaŭ, 65148 post`; 0 esceptoj |
| 2026-11-12 | 2. proceza abstrakto, `krei_kunhavantan`, `mapi_uzantan` | 0.2.1 | PVH/ISO: `[procezo] 4: Preta, kerna stako 0x0, kerna mapo kunhavata, uzanta fenestro privata`; 0 esceptoj |
| 2026-11-12 | 3. `CR3`-ŝalto, per-procesa `RSP0`, bildo en la proceza spaco | 0.2.2 | PVH: `kerna stako 0x161d60, PML4 0x204000, kerna mapo kunhavata, bildo nur en la proceza spaco jes`; ISO: PML4 `0x209000`; 0 esceptoj, 1000 tikoj |
| 2026-11-12 | 4. privataj `PDPT`-oj, detruo ĉe `FINI`/`#PF`, reuzoprovo kaj du procezoj | 0.2.3 | PVH: `reuzo (bone): 10 cikloj, 64984 kadroj antaŭ, 64984 post`; `[procezo] 4 detruita: 1 privataj PDPT-oj … 27 kadroj liberigitaj`; `[eraro] procezo 5: paĝa eraro … CR2=0xdeadbeef`; ISO: sama, 64982 kadroj; `fina stato: 5 mortintaj` |
| 2026-11-12 | 5. samtempeco, sekreta paĝo, interproceza izolado | 0.2.4 | PVH: `du procezoj pretaj; unu planista voko`; sama adreso `0x200000000`, id `4`/`5`; `erarkodo=0x4, CR2=0x200001000`; `pluvivas post la spiono`; `izolado (bone): 64979 kadroj antaŭ, 64979 post`; ISO: 64977 egalaj |
| 2026-11-12 | 6. W^X laŭ sekcioj, `CR0.WP`, foliaj permesoj en sistemvokoj | 0.2.5 | PVH/ISO BIOS/ISO UEFI: `[memoro] W^X (bone): .text RX, .rodata R NX, .data/.bss RW NX, modulo R NX`; `enir_rx=true stako_rw=true`; `[eraro] procezo 4: montrilo 0x100001840 … ne havas la postulatan folian permeson`; `[ring3] negativaj: … nurlega=-1`; izolado `64979`, ISO `64977`, UEFI `63507` — ĉiuj egalaj |

**Akceptkriterioj — pruvitaj.**

| Kriterio | Stato | Pruvo |
| --- | --- | --- |
| Du procezoj ne povas legi unu la alian | **Pruvita** | Procezoj `4` kaj `5` kuradas samtempe. Ĉiu mapas sekretan paĝon ĉe la sama virtuala adreso `0x2_0000_0000` sed en malsama kadro: `[ring3] procezo 4 … id 4` kaj `[ring3] procezo 5 … id 5`. La viktimo mapas duan paĝon `0x2_0000_1000`; la spiono legas ĝin kaj ricevas `#PF` (`erarkodo=0x4`, `CR2=0x200001000`). |
| Paĝa eraro en unu procezo ne haltigas la kernon | Pruvita | `[eraro] procezo 5: paĝa eraro …`; la procezo estas mortigata kaj detruata, procezo `4` daŭrigas tra la rondoj `2`–`6`, finiĝas per `SYS_FINI`, kaj la sistemo atingas 1000 tikojn. |
| La kadro-alokilo reuzas liberigitajn kadrojn | Pruvita | `[memoro] reuzo (bone): 10 cikloj, 64979 kadroj antaŭ, 64979 post`; post la scenaro `[memoro] izolado (bone): 64979 kadroj antaŭ, 64979 post` (ISO `64977`, UEFI `63507`). Ĉiu detruo raportas `1 privataj PDPT-oj … 32/33 kadroj liberigitaj`. |
| La kerno kaj uzantspaco respektas W^X | Pruvita | `W^X (bone)` legas la realajn foliajn permesojn: `.text` RX, `.rodata` R NX, datumoj RW NX, moduloj R NX. La ŝargilo rifuzas `W+X` segmenton (`NevalidaWX`); Ring 3 ne povas skribi en la enirpaĝon (`ne havas la postulatan folian permeson`, `nurlega=-1`). |

**Fazo 1 (0.2.0).** `KadraAlokilo::liberigi` malmarkas la biton, nuligas la
kadron kaj retreatigas `sekva`; duobla liberigo estas senefika. `detruu`
malmapas la uzantan fenestron, liberigas la foliojn kaj la paĝtabelojn, kiuj
restas malplenaj, plus la radikan kadron. La forigo estas limigita al
`UZANTA_BAZO..UZANTA_FINO`, ne al la malaltaj 256 PML4-eroj: la identeca mapo
de la unuaj 4 GiB kuŝas en PML4[0], do laŭvorta «nur malaltaj 256 eroj»
forigus ankaŭ la kernajn mapojn. Tabeloj kun kernaj eroj restas, ĉar ili ne
malpleniĝas. Grandega folio en la fenestro kaŭzas `Konflikto` anstataŭ
danĝeran liberigon. Ĉe starto la nukleo kreas provizan spacon, mapas unu paĝon,
detruas ĝin kaj komparas la nombrojn de liberaj kadroj.

**Fazo 2 (0.2.1).** `Procezo` (`kerno/procezo.rs`) kunigas identigilon,
privatan `Spaco`-on, estontan kernan stakpinton kaj staton (`Preta`,
`Kuranta`, `Mortinta`); la tabelo tenas 8 procezojn kaj la nukleo restas
procezo `0`. `Fadeno` portas `procezo`-idon, kaj la planisto spegulas la
statojn. `X86AdresaSpaco::krei_kunhavantan` alokas freŝan PML4-on kaj, por
ĉiu ĉeesta ero, inspektas la suban PDPT-on: eroj kun `U=1` — aŭ eroj, kiuj
kovras parton de la uzanta fenestro — ricevas privatan PDPT-on kun nur la
`U=0`-eroj; la ceteraj PDPT-kadroj estas kunhavataj. Tio kongruas kun la
regulo «dinamike kontroli la biton `U`», sed ne eblas kunhavigi `PML4[0]`:
en MiKerno la fenestro kuŝas sub ĝi, do novaj uzantaj mapoj poluus la fontan
spacon. La protokolo ne uzas `PML4[511]`-rekursion. `mapi_uzantan` kaj
`mapi_uzantan_regionon` (trait `PaĝaTablo`) kontrolas `[UZANTA_BAZO,
UZANTA_FINO)` per `checked_add` kaj devigas `U=1`; la ELF-ŝargilo kaj la
stako nun uzas ilin.

**Decido (starta vojo).** `krei()` restas «freŝa, malplena radiko» por la
nukleo: ĝi estas vokata antaŭ ol ekzistas kerna spaco, dum `CR3` ankoraŭ
montras la praŝarĝilan tabularon kun 2 MiB paĝoj. La kunhava konstruilo do
estas aparta metodo, `krei_kunhavantan(&self, …)`: ĝi ekzakte plenumas la
taskon 2.2 (dinamika identigo de kernaj PDPT-eroj) sen antaŭtempe kunhavigi
la praŝarĝajn tabelojn aŭ perdi la 4 KiB-identecan mapon de la kerno.

**Fazo 3 (0.2.2).** La planisto nun ŝaltas la adresan spacon de la celata
fadeno. `procezo::kerna_stako_pinto` donas al ĉiu procezo propran 16 KiB
kernostakon el statika tabelo `KERNAJ_STAKOJ` en `.bss` (`repr(align(16))`,
pinto = bazo + 16384, 16-bajte aliniiĝinta sed ne paĝe aliniiĝinta); la tabelo
evitas novan kontiguan alokan primitivon kaj la alokilon entute, do ĝi ankaŭ
ne influas la Kani-modelon. La malnova `UZANTA_FADENA_STAKO` forfalis: la uzanta
fadeno ekas sur la proceza kernostako, kies pinton `RSP0` poste reuzas — la
kernaj kadroj estas mortaj post `iretq`. `kerno/mod.rs` registras la kernan
radikon per `procezo::agordi_kernan_spacon`; `procezo::aktivigi(id)` komparas
`arkitekturo::aktiva_adreso_spaco()` kun la celata `radika_adreso()` kaj faras
`mov cr3` nur se ili diferencas, poste `agordi_kernostakon(pinto)` (procezo `0`
revenas al la defaŭlta TSS-stako). `planisto::ŝanĝi` vokas `aktivigi` antaŭ
`ŝalti_kuntekston`; tio estas sekura ĉar ĉiuj fadenaj `Kunteksto`-j, la
planista stato, la kerna bildo kaj la kernostakoj kuŝas en la kerna mapo, kiun
ĉiu proceza spaco kunhavas, kaj `ŝanĝi` ĉiam estas vokata kun `IF=0`.
`uzantspaco::pravalorizi` nun ŝarĝas la ELF-on rekte en la procezan spacon, do
la kerno ne plu gastigas kopion de la bildo; la memtesto de la marko portempe
ŝaltas la procezan `CR3` sub `InterrompaGardo` kaj restarigas la antaŭan
radikon. La starta raporto pruvas la izoladon: la kerna spaco ne havas mapon ĉe
`bilda_komenco`, la proceza havas la kernan mapon kaj la bildon, kaj la nombro
de liberaj kadroj restas egala antaŭ/post la memtesto.

**Fazo 4 (0.2.3).** `detruu` nun liberigas ankaŭ la privatajn `PDPT`-ojn.
`X86AdresaSpaco` tenas 512-bitan bitaron `privataj_pdptoj`, kiun
`kopii_kernajn` plenigas por ĉiu `PML4`-ero, kiun ĝi privatigis. Post la
fenestra liberigo `detruu` trairas nur tiujn indeksojn, forigas la eron el la
radika tabelo kaj redonas la kadron. La kerno ne liberigas blinde la subtableojn
de `PML4[0]`: tie kuŝas la identeca mapo de la unuaj 4 GiB, do `PML4[0]` estas
ĉiam privatigita, sed la `U=0`-eroj en ĝia `PDPT` estas kopioj de la kernaj
mapoj, ne apartenaj kadroj. Se post la fenestra liberigo restus `U=1`-ero,
`detruu` rifuzas per `Konflikto` anstataŭ riski kernan mapon.
`kerno/procezo::detrui` reŝaltas la kernan `CR3` se la proceza estas aktiva —
la nuna kodo kaj la kernaj stakoj kuŝas en la kunhavata kerna mapo — kaj
raportas la antaŭan kaj postan nombrojn de liberaj kadroj. Ĉar MiKerno ne uzas
PCID, tiu `mov cr3` ankaŭ malplenigas la TLB; neniu malnova mapo restas videbla.

La planisto nun detruas la procezon en `morti`: `SYS_FINI` kaj mortiga Ring
3-escepto sekvas la saman vojon. La escepta dispetĉo kontrolas `CS & 3 == 3`
kaj, por vektoro 14 aŭ 13, printas `[eraro] procezo N …` kun `rip`,
`erarkodo` kaj `CR2`, poste vokas `planisto::morti()`; `#PF`/`#GP` el Ring 0
kaj ĉiuj aliaj esceptoj restas fatalaj kaj haltigas la sistemon.

Por pruvi la vojon la praŝarĝo rulas `testi_procezan_reuzon`: dek fojojn
`krei` + `munti` + `detrui` de proceza spaco kun la reala `horizonto`-bildo,
kaj komparas `liberaj()` antaŭ/post (PVH `64979`, ISO `64977` — egalaj).

**Fazo 5 (0.2.4).** La lasta akceptkriterio fermiĝis per du samtempaj procezoj:

1. `UzantaBildo` — la bildaj limoj kaj la enirpunkto — apartenas al `Procezo`
   (`agordi_bildon`, `bildaj_limoj`, `enirpunkto`), ne al tutmonda stato. La
   sistemvoka montrila kontrolo kaj la Ring 3-eniro legas la nunan procezon.
2. La proceza identigilo estas dua `iretq`-argumento (`RSI`), do `horizonto`
   ricevas `(reĝimo, identigilo)`; la kerno transdonas `REĜIMO_SEKRETO` aŭ
   `REĜIMO_SPIONO` en `RDI`.
3. `uzantspaco::mapi_sekreton` alokas freŝan nuligitan kadron, mapas ĝin kiel
   `U=1`-folion ĉe `SEKRETA_ADRESO = 0x2_0000_0000` en ĉiu procezo, kaj skribas
   la magian numeron `0x4D69_4B65_726E_6F21` plus la procezan id-on per la
   identeca mapo. La virtuala adreso samas, la kadroj malsamas — la spiono legas
   sian propran id-on, ne la viktiman.
4. La viktimo krome mapas `SEKRETA_DUA_ADRESO = 0x2_0000_1000`; la spiono ne.
   La legado de la spiono trafas `#PF` (`erarkodo=0x4`, `CR2=0x200001000`).
5. La praŝarĝo preparas ambaŭ procezojn kun interrompoj malŝaltitaj kaj vokas
   `planisto::lanĉi()` unufoje; `horizonto` mem cedas (`SYS_CEDI`), do la
   protokolo interplektiĝas anstataŭ ruli sinsekve. La viktimo printas la
   rondojn `1`–`6`, pluvivas la spionon kaj finiĝas per `SYS_FINI`.
6. Ĉiu elformata linio en Ring 3 uziĝas per unu `SYS_SKRIBI` (256-bajta bufro
   en `horizonto/src/biblioteko.rs`), do linioj ne dispecetiĝas inter la
   procezoj.
7. `[memoro] izolado (bone)` komparas la liberajn kadrojn antaŭ/post la
   scenaro: PVH `64979 = 64979`, ISO `64977 = 64977`; ambaŭ privataj `PDPT`-oj
   kaj ĉiuj folioj revenas al la alokilo.

**Fazo 6 (0.2.5).** La origina Fazo 4-postulo pri W^X fermiĝis:

1. `linker.ld` donas sekciajn limojn (`__kerna_kodo_komenco/fino`,
   `__kerna_rodata_komenco/fino`, `__kerna_datumoj_komenco`, `__kerna_fino`) kaj
   kunigas la `code-model=large`-sekciojn `.ltext`/`.ltext.*` en `.text`; sen
   tio ili fariĝas orfaj kaj finiĝas en la nurlega antaŭkodo.
2. `idente_mapigi` mapas laŭ tiuj limoj: `.text` RX, `.rodata` R NX, `.data`,
   `.bss`, `.boot_bss` kaj la cetera uzebla ĉefmemoro RW NX. La praŝarĝaj moduloj estas
   mapitaj **antaŭ** la identeca mapo kiel R NX, ĉar poste la jam mapitaj folioj
   ne plu ŝanĝiĝas.
3. `PaĝaPermesoj` nun havas `kodo`/`rodata`/`datumoj`-konstantojn kaj
   `permesoj(adreso)`-kontrakton; `testi_w_x` legas la realajn PTE-ojn kaj
   printas `[memoro] W^X (bone)`.
4. `CR0.WP` estas ŝaltita en `pravalorizi`, do eĉ Ring 0 ne povas skribi tra
   nurlega folio.
5. La ELF-ŝargilo rifuzas segmenton, kiu postulas samtempe `W` kaj `X`
   (`ŜargaEraro::NevalidaWX`), kaj donas al skribeblaj folioj `NX`.
6. `SYS_SKRIBI`/`SYS_SENDI` postulas legeblan folion, `SYS_RICEVI` skribeblan;
   `kontroli_uzantan` kontrolas ĉiun folion de la intervalo, ne nur la regionajn
   limojn. Ring 3-provokoj raportas `ne havas la postulatan folian permeson`.
7. `horizonto` aldonas la negativan kazon `SYS_RICEVI` en la enirpaĝon:
   `nurlega=-1`.

---

### M13 — Kerna memoro-firmigo

**Celo.** Firmigi la limon inter Ring 0 kaj Ring 3 kaj malfaciligi la
ekspluaton de konataj kernaj adresoj.

**Efektivigo.** Tri sendependaj tavoloj:

1. **SMEP (0.3.0)** — CPUID-provo, `CR4.SMEP`, malpermesi plenumon de uzantaj
   paĝoj el Ring 0; se la procesoro ne subtenas ĝin, la kerno bone faldas.
2. **SMAP (0.3.1)** — CPUID-provo, `CR4.SMAP`, `STAC`/`CLAC`-fenestroj ĉirkaŭ
   ĉiu kerna aliro al uzantaj bufroj (RAII-gardo `StacGardo`); revizio de
   `SYS_SKRIBI`, `SYS_SENDI` kaj `SYS_RICEVI`.
3. **KASLR (0.3.2)** — la bildo estas ligita kiel ET_DYN
   (`relocation-model=pic`, `-Bsymbolic`) ĉe la sama 1 MiB. La ligilo produktas
   nur `R_X86_64_RELATIVE`-relokojn. `hazardigo` legas la memormapon, elektas
   2 MiB-aliniiĝintan bazon en uzebla ĉefmemoro (≥ 2 MiB, sub 4 GiB, ekster la
   devena bildo, la moduloj, la praŝarĝaj strukturoj kaj la frambufo), kopias
   la bildon per `rep movsb`, aplikas la relokojn al la kopio kaj saltas en ĝin
   per `kerna_salto_al_kopio`. La 64-bita eniro aplikas la relokojn ankaŭ al la
   devena bildo antaŭ la unua Rust-voko, ĉar la ligilo lasas la RELATIVE-celojn
   nulaj (la aldonaĵo estas nur en `.rela.dyn`). La 32-bita ŝelko trovas sian
   bazon per `call`/`pop` kaj uzas nur nombrajn deŝovojn; la praŝarĝa stako
   translokiĝis al `.bss`, ĉar `.boot_text` estas nurlega post la unua
   `CR3`-ŝalto. Ĉiuj limoj (`__kerna_komenco`, `__kerna_fino`,
   `__bss_komenco`, `__bss_fino`, `__rela_komenco`, `__rela_fino`, sekciaj
   limoj) kuŝas ene de sekcioj, do ili ricevas la saman RELATIVE-relokon.
   Sen `RDRAND` la hazardo venas el du `RDTSC`-legoj miksitaj per SplitMix64.
   La kadro-alokilo rezervas ankaŭ la devenan bildon ĉe `origino` ĝis la unua
   `CR3`-ŝalto.

**Progreso.**

| Dato | Fazo | Versio | Pruvo |
| --- | --- | --- | --- |
| 2026-11-13 | 1. SMEP: CPUID-provo, `CR4.SMEP`, Ring 0-provo | 0.3.0 | PVH `-cpu qemu64,+smep,+smap`: `[kerno] SMEP subtenata: jes`, `[kerno] SMEP ŝaltita`, post la 1000 tikoj `[memoro] SMEP-provo … ĉe 0x100000000` kaj `[eraro] SMEP-malobservo … (erarkodo=0x11)`; la tuta M12-scenaro antaŭe verda (`izolado 64980 = 64980`). Sen SMEP (`-cpu qemu64`): `subtenata: ne`, provo preterlasita, 1000 tikoj, neniuj esceptoj |
| 2026-11-13 | 2. SMAP: CPUID-provo, `CR4.SMAP`, `STAC`/`CLAC`-gardo en sistemvokoj kaj ŝargilo | 0.3.1 | PVH `-cpu qemu64,+smep,+smap`: `SMAP subtenata: jes`, `SMAP ŝaltita`, M12 verda (`izolado 64978 = 64978`), Ring 3-linioj kaj `[uzanto] memoro: marko … legita …` pruvas la pozitivan fenestron, fine `[memoro] SMAP-provo … ĉe 0x100000000` kaj `[eraro] procezo 0: SMAP-malobservo … (erarkodo=0x1)`. ISO BIOS `64976 = 64976`, UEFI/OVMF `63506 = 63506`, VirtualBox EFI `60216 = 60216`. Sen la trajtoj (`-cpu qemu64`, VirtualBox): `subtenata: ne`, provo preterlasita, 1000 tikoj, neniuj esceptoj |
| 2026-11-13 | 3. KASLR: ET_DYN, relokebla bildo, hazarda bazo | 0.3.2 | PVH `-cpu qemu64,+smep,+smap`: tri startoj montras `0x5400000`, `0xd200000` kaj `0x3000000`; `[memoro] W^X (bone)` kaj `izolado 64839 = 64839`; ISO BIOS `0x4200000` (`64837 = 64837`), UEFI/OVMF `0x4a00000` (`63367 = 63367`); sen la trajtoj (`-cpu qemu64`) `0x8c00000`, SMEP/SMAP-provo preterlasita, 1000 tikoj |

**Akceptkriterioj — pruvitaj.**

| Kriterio | Stato | Pruvo |
| --- | --- | --- |
| Ring 0 ne povas plenumi uzantan paĝon (SMEP) | **Pruvita** | `[eraro] SMEP-malobservo … (erarkodo=0x11)` post la Ring 0-provo |
| Ring 0 legas uzantan bufron nur en `STAC`-fenestro | **Pruvita** | `[uzanto] memoro: marko … legita …`; sen `STAC` `[eraro] procezo 0: SMAP-malobservo … (erarkodo=0x1)` |
| Du startoj montras malsamajn bazadresojn | **Pruvita** | Tri PVH-startoj: `0x5400000`, `0xd200000`, `0x3000000`; ISO BIOS `0x4200000`, UEFI/OVMF `0x4a00000` |
| Ĉiuj antaŭaj scenaroj restas verdaj | **Pruvita** | `W^X (bone)`, `reuzo (bone): 10 cikloj`, `izolado 64839 = 64839`, 5 fadenoj mortintaj, 1000 tikoj |

**Versioj.** 0.3.0 (SMEP), 0.3.1 (SMAP), 0.3.2 (KASLR).

---

### M15 — `arm64`-malantaŭo (kunfando de la ŝpuro)

**Celo.** La sama kerno, la sama ABI 2 kaj la sama bildo sur `arm64`; nur la
enirmekanismo kaj la registra aranĝo estas platformaj.

**Efektivigo.** Kvar antaŭeldonoj en la branĉo `arm64` (A0–A4) kaj la
kunfando:

- A0 (`0.4.3`, sen kondutŝanĝo) — `Kunteksto`, `InterrompaKadro` kaj la
  escepta klasifiko translokiĝis al la malantaŭo; `SistemvokaKadro`,
  `FirmigaRaporto` kaj `ELF_MAŜINO` eniris la fasadon.
- A1 (`0.4.3-arm.1`) — plata `Image` kun 64-bajta kapo, EL2→EL1 per
  `HCR_EL2.RW`, PL011, minimuma FDT-analizilo, identeca MMU per 1 GiB-blokoj,
  PSCI `SYSTEM_OFF` kaj UEFI-ISO per Limine (`BOOTAA64.EFI`).
- A2 (`0.4.3-arm.2`) — `VBAR_EL1` kun plena kadro, GICv2, ĝenerala
  temporizilo je 100 Hz, `DAIF`-gardo kaj kuntekstŝanĝo.
- A3 (`0.4.3-arm.3`) — `EL0` per `eret`, `svc #0` (`x8` = numero, `x0`–`x3` =
  argumentoj), per-procezaj `TTBR0_EL1`-spacoj kaj ELF-maŝino `183`.
- A4 (`0.4.3-arm.4`) — `ramfb` per `fw_cfg`-DMA, piksela fiksaĵo,
  `PXN`/`UXN`/`PAN` kaj `PAN`-malobserva raporto.
- Kunfando (`0.5.0`, 2026-11-15) — la ABI 2-plena `voki` (kvar argumentoj) en
  ambaŭ eniroj, `#[cfg]`-gardoj por la `x86_64`-nuraj pruvoj (`AC`, `CR3`,
  `EOI`, SMEP/SMAP), `just kontroli` kun `clippy` por `aarch64-unknown-none`,
  kaj la dokumentaro unuigita.

**Pruvo.** Ambaŭ akceptoj estas verdaj; la frambufferio de `arm64` (per
`-kernel` kaj per la UEFI-ISO) kaj la `x86_64`-ISO donas la saman haketon:

```text
[kerno] PAN subtenata: jes
[kerno] frambufo 1280x800, 32 bitoj/pikselo
[ekrano] piksela fiksaĵo 1280x800, 32 bitoj/pikselo, tiparo 12x24; haketo 0xecf1f6dbe1421575
[eraro] PAN-malobservo: EL1 provis legi uzantan paĝon ĉe 0x0000000100000000 (esr=0x9600000f)
```

La `x86_64`-vojoj (PVH kaj ISO) restas senŝanĝaj; `just kontroli` inkluzivas la
`arm64`-lintadon kaj la Kani-pruvojn. Limigoj: SMP, IOMMU, KASLR kaj
Kani-pruvoj por `arm64` ne estas parto de M15 (vidu [ARM.md](ARM.md) §2).

**Versio.** 0.5.0 (kunfando de A0–A4).

---

## 3. Partaj mejloŝtonoj

### M9 — Kvalita pordego
**Celo.** Ĉiu ŝanĝo estu pruvebla per ripeteblaj komandoj.

**Havigita.**

- `cargo fmt --all -- --check` — pura.
- `cargo clippy --workspace --release` — sen avertoj.
- `cargo build --workspace --release` — sukcesa.
- 26 Kani-pruvoj: 6 en `memoro/kadro.rs`, 20 en `kerno/ik.rs`; ĉiuj maŝine
  pruvitaj per Kani 0.68.0 (CBMC 6.11.0) en WSL2 Ubuntu; vidu §4.6.
- `just kontroli` difinas la formalan pordegon.

**Pruvoj kaj `unwind`-limoj.** La pruvoj grupiĝas laŭ la pruvata tavolo; `—`
signifas la defaŭltan `unwind`-valoron.

| Dosiero | Pruvoj | `unwind` |
| --- | --- | --- |
| `memoro/kadro.rs` | intervalo, preterpaso de uzataj kadroj, duobla liberigo, reuzo, sen duobla alokado, marki/malmarki | 70 (krom la lasta) |
| `kerno/ik.rs` | mesaĝujo: ena longo, pleneco, preno, FIFO, FIFO inter fontoj, malpleno | 70/20/20/8/8/— |
| `kerno/ik.rs` | teniloj kaj finpunktoj: kodigo, generacioj, referencoj, libera fendo | —/—/—/20 |
| `kerno/ik.rs` | kapabloj: rajtoj de `SENDI`/`RICEVI`, `DONI`, transproceza izolado | — |
| `kerno/ik.rs` | revoko, premrego, senda bito, fonta filtro | —/—/—/8 |
| `kerno/ik.rs` | tempolimoj: politiko de `r10`, minimuma tiko | — |

**Mankanta.**

- La pruvoj spegulas la logikon per 64-kadra modelo anstataŭ pruvi la
  produktan `KadraAlokilo`.
- La formala pordego nun rulas en GitHub Actions (vidu §4.7); `cargo kani` ne
  plu ruliĝas loke. Ankoraŭ mankas unuopa aŭ bitnivela testo, `miri` kaj
  `loom`.

**Sekva paŝo.** Alproksimigi la pruvojn al la produkta kodo per etaj
abstraktaĵoj kaj aldoni testojn por la arkitekture sendependaj partoj
(`MemoraMapo`, `Mesagujo`, `PretaVico`, `VirtuaAdreso`).

---

## 4. Pruvprotokolo de 2026-11-12

Medio: WSL2 Ubuntu, Rust 1.99.1, QEMU 256 MiB. (La protokolo estis unue
farita sur Windows per Git Bash; la komandoj samas en WSL2.)

### 4.1 Statika kontrolo

| Komando | Rezulto |
| --- | --- |
| `cargo build --workspace --release` | Sukceso, sen avertoj |
| `cargo clippy --workspace --release` | Sukceso, sen avertoj |
| `cargo fmt --all -- --check` | Sen diferencoj |
| `cargo kani --version` | `Kani Rust Verifier 0.68.0 (cargo plugin)`, `CBMC 6.11.0` |
| `just iso` | ISO kreita kaj startita |

### 4.2 PVH-starto

Komando (senkapa QEMU, 25 s):

```sh
qemu-system-x86_64 -kernel target/x86_64-unknown-none/release/mikerno \
  -initrd target/x86_64-unknown-none/release/horizonto \
  -m 256M -serial stdio -display none -no-reboot -no-shutdown
```

Elĉerpaĵo:

```text
[MiKerno] Saluton, mi estas MiKerno v0.2.5!
[kerno] memoro: 7 regionoj, 255 MiB uzeblaj
[kerno] frambufo ne havebla; nur COM1
[memoro] W^X (bone): .text RX, .rodata R NX, .data/.bss RW NX, modulo R NX
[tss] TR ŝargita; kernostako pinto 0x148d90
[temporizilo] kalibrado: 62561224 kalkuloj/s, komenca 625612
[memoro] reuzo (bone): 10 cikloj, 64979 kadroj antaŭ, 64979 post
...
[procezo] 4: Preta, kerna stako 0x138d90, PML4 0x207000, kerna mapo kunhavata, bildo nur en la proceza spaco jes, sekreta paĝo 0x200000000
[kerno] horizonto 4 pretas: CS=0x23, SS=0x1B, RFLAGS=0x202, enirpunkto 0x100001840
[procezo] 5: Preta, kerna stako 0x13cd90, PML4 0x228000, kerna mapo kunhavata, bildo nur en la proceza spaco jes, sekreta paĝo 0x200000000
[kerno] horizonto 5 pretas: CS=0x23, SS=0x1B, RFLAGS=0x202, enirpunkto 0x100001840
[kerno] du procezoj pretaj; unu planista voko
horizonto: procezo vekis en Ring 3.
[eraro] procezo 5: montrilo 0x1000 (longo 8) estas ekster la anoncita regiono
[eraro] procezo 5: montrilo 0x100001840 (longo 96, skriba true) ne havas la postulatan folian permeson
[ring3] negativaj: nekonata=-2, nula=-1, kerna=-1, troa=-1, nurlega=-1
[ring3] procezo 5: sekreta paĝo kongruas (magio 0x4d694b65726e6f21, id 5)
[ring3] procezo 5: spionas la viktimon
horizonto: procezo vekis en Ring 3.
[eraro] procezo 4: montrilo 0x1000 (longo 8) estas ekster la anoncita regiono
[eraro] procezo 4: montrilo 0x100001840 (longo 96, skriba true) ne havas la postulatan folian permeson
[ring3] negativaj: nekonata=-2, nula=-1, kerna=-1, troa=-1, nurlega=-1
[ring3] procezo 4: sekreta paĝo kongruas (magio 0x4d694b65726e6f21, id 4)
[ring3] procezo 4: laboras (fojo 1)
[ring3] procezo 5: legas la duan sekretan paĝon ĉe 0x200001000

[eraro] procezo 5: paĝa eraro ĉe rip=0x00000001000014ce, erarkodo=0x4, CR2=0x0000000200001000; la procezo estas mortigata
[procezo] 5 detruita: 1 privataj PDPT-oj, PML4 0x228000, 32 kadroj liberigitaj
[ring3] procezo 4: laboras (fojo 2)
...
[ring3] procezo 4: laboras (fojo 6)
[ring3] procezo 4: pluvivas post la spiono; finas per SYS_FINI
[procezo] 4 detruita: 1 privataj PDPT-oj, PML4 0x207000, 33 kadroj liberigitaj
[memoro] izolado (bone): 64979 kadroj antaŭ, 64979 post
[kerno] planisto revenis al la ripoza fadeno
...
[kerno] fina stato: 5 mortintaj, 0 pretaj, 0 blokitaj; 1000 tikoj
```

### 4.3 ISO-starto

Komando (BIOS):

```sh
qemu-system-x86_64 -cdrom mikerno.iso -m 256M -serial stdio \
  -display none -no-reboot -no-shutdown
```

Komando (UEFI, OVMF):

```sh
qemu-system-x86_64 -machine q35 \
  -drive if=pflash,format=raw,readonly=on,file=target/edk2-x86_64-code.fd \
  -cdrom mikerno.iso -m 256M -serial stdio -display none \
  -no-reboot -no-shutdown
```

Elĉerpaĵo (BIOS):

```text
[MiKerno] Saluton, mi estas MiKerno v0.2.5!
[kerno] memoro: 7 regionoj, 255 MiB uzeblaj
[kerno] frambufo 1280x800, 32 bitoj/pikselo
[memoro] W^X (bone): .text RX, .rodata R NX, .data/.bss RW NX, modulo R NX
[memoro] reuzo (bone): 10 cikloj, 64977 kadroj antaŭ, 64977 post
...
[procezo] 4: Preta, kerna stako 0x138d90, PML4 0x20f000, … sekreta paĝo 0x200000000
[procezo] 5: Preta, kerna stako 0x13cd90, PML4 0x230000, … sekreta paĝo 0x200000000
[kerno] du procezoj pretaj; unu planista voko
...
[eraro] procezo 5: montrilo 0x100001840 (longo 96, skriba true) ne havas la postulatan folian permeson
[ring3] negativaj: nekonata=-2, nula=-1, kerna=-1, troa=-1, nurlega=-1
[ring3] procezo 4: sekreta paĝo kongruas (magio 0x4d694b65726e6f21, id 4)
[ring3] procezo 5: sekreta paĝo kongruas (magio 0x4d694b65726e6f21, id 5)
[ring3] procezo 5: legas la duan sekretan paĝon ĉe 0x200001000
[eraro] procezo 5: paĝa eraro ĉe rip=0x00000001000014ce, erarkodo=0x4, CR2=0x0000000200001000; la procezo estas mortigata
[procezo] 5 detruita: 1 privataj PDPT-oj, PML4 0x230000, 32 kadroj liberigitaj
...
[ring3] procezo 4: pluvivas post la spiono; finas per SYS_FINI
[procezo] 4 detruita: 1 privataj PDPT-oj, PML4 0x20f000, 33 kadroj liberigitaj
[memoro] izolado (bone): 64977 kadroj antaŭ, 64977 post
[kerno] fina stato: 5 mortintaj, 0 pretaj, 0 blokitaj; 1000 tikoj
```

La UEFI-starto montras la samajn W^X, folipermesajn kaj izoladajn liniojn;
nur la frambufo kaj la liberaj kadroj malsamas (`izolado (bone): 63507
kadroj antaŭ, 63507 post`).

### 4.4 Trovaĵoj de la kontrolo

1. **Dispecetigaj linioj (riparebla).** La unua PVH-starto de 0.2.4 montris
   `[ring3] procezo horizonto: procezo vekis…` — `format_args!` eligis unu
   `SYS_SKRIBI` po fragmento, kaj la alia procezo kuris inter ili. La riparo
   estas 256-bajta bufro en `horizonto/src/biblioteko.rs`, kiu kunigas la tutan
   linion antaŭ unu voko; poste ĉiu linio restas aparta.
2. **Ordo de la scenaro (riparebla).** En la unua versio la viktimo finiĝis
   antaŭ ol la spiono eraris, do la protokolo ne montris ĝian pluvivon. La
   viktimo nun havas ses rondojn kun `SYS_CEDI` kaj la spiono unu antenaŭ la
   fremda legado; la spiono mortas post la dua rondo de la viktimo.
3. **Malaktuala ISO (grava).** Historia trovaĵo de 0.1.x: `iso_root/` kaj
   `mikerno.iso` estas git-ignoritaj, do malnova ISO povas montri malnovan
   version. Oni ĉiam rulu `just iso` post kerna aŭ dokumenta ŝanĝo.
4. **Versio hardkodita (riparita en 0.4.3).** `kerno/mod.rs` kaj
   `mikerno/boot/limine.conf` iam portis la nunan version sendepende. Nun la
   kerno legas `env!("CARGO_PKG_VERSION")`, la menuo ne plu montras version,
   kaj `tools/versio.sh` kontrolas la dokumentojn; la sola fonto restas
   `Cargo.toml`.
5. **Piksela kontrolo nur per `screendump`.** La frambufo estis vidita per
   QEMU `screendump` (2026-11-12): la titolo, la ĉapelitaj literoj kaj la
   citiloj desegniĝas ĝuste. La alta distingivo (16×32/32×64) ne estis provita
   en QEMU; ĝi dependas nur de la sama elekta funkcio, sed meritas apartan
   starton kun pli granda frambufo. La aspekto sur reala aparato ankoraŭ
   ne estas kontrolita.
6. **`just ruli-iso` malfermas SDL-fenestron**; por senkapa kontrolo oni voku
   QEMU rekte aŭ uzu `QEMU_DISPLAY=none`.
7. **Orfaj `.ltext`-sekcioj (grava, 0.2.5).** Kun `code-model=large` LLVM metas
   multajn funkciojn en `.ltext.*`; la unua W^X-mapado lasis ilin ekster
   `.text`, do ili falis en la nurlegan antaŭkodon kaj la unua alvoko trioble
   fuŝis. La riparo kunigas `*(.ltext .ltext.*)` en `.text`; oni kontrolu la
   sekcian aranĝon per `llvm-readobj --sections` post ŝanĝoj de la linker-skripto.

### 4.5 Ripeto per Rust 1.100.0 (2026-11-13)

La minimuma subtenata Rust-versio altiĝis de `1.85` al `1.100`
(`Cargo.toml`, `workspace.package`). La projekto celas unu aktualan eldonilon
kaj ne subtenas malnovajn ilojn, kiujn neniu el la testmedioj uzas.

La tuta laborloko estis rekompilita de nulo per Rust 1.100.0:

| Kontrolo | Rezulto |
| --- | --- |
| `cargo build --workspace --release` | Sukceso en 5.76 s |
| `cargo clippy --workspace --release` | Sukceso, sen avertoj (post riparo de 12 novaj avertoj) |
| `cargo fmt --all -- --check` | Neniu diferenco |
| PVH-starto (`qemu64,+smep,+smap`) | Verda: `KASLR: kerna bazo 0x800000`, `reuzo (bone): 10 cikloj, 64839 kadroj antaŭ, 64839 post`, `izolado (bone): 64839 kadroj antaŭ, 64839 post`, 5 mortintaj fadenoj, `SMAP-malobservo` mortigas nur la Ring 3-procezon |
| ISO-starto (Limine, BIOS, `qemu64,+smep,+smap`) | Verda: `KASLR: kerna bazo 0x7800000`, frambufo `1280x800, 32 bitoj/pikselo`, `izolado (bone): 64837 kadroj antaŭ, 64837 post`, 5 mortintaj fadenoj, 1000 tikoj |

La novaj lintoj estis `manual_is_multiple_of` (9) kaj `collapsible_if` (3); ili
estis riparitaj per `is_multiple_of()` kaj `let`-ĉenoj, do la konduto ne
ŝanĝiĝis. La ISO estis regenerita per la sama ilaro; la Eldona aktivaĵo
`mikerno.iso` (v0.3.2) anstataŭiĝis (SHA-256
`7e8c2fb764cb1e55598fbfa5984d6c21b53e188286fd33453c5ed46df8d1674a`) kaj la
ISO-BIOS-starto pruvas la novan bildon.

### 4.6 Formala pordego per Kani (2026-11-13)

Medio: WSL2 Ubuntu, deponejo kopiita al la hejma dosierujo (ext4; `/mnt/d` estas
9p kaj tro malrapida por la kompilo kaj CBMC), Kani 0.68.0, CBMC 6.11.0.
`just kontroli` vokas
`cargo kani -p mikerno -Z unstable-options --ignore-global-asm`; la lasta
preterlasas la tutmondan `asm!`-eniron de la 64-bita starto, kiun CBMC ne
modelas.

Rezulto: `Complete - 26 successfully verified harnesses, 0 failures, 26 total.`
La kovrado spegulas la mesaĝujon (ena longo, pleneco, FIFO, malpleno), la
tenilan kodigon kaj finpunktajn generaciojn, la kapablajn permesojn kaj
transprocezan izoladon, la revokan invarianton, la premregan politikon kun la
senda bito, la fontan filtron kaj la tempoliman politikon (`u32::MAX` → senfina,
`0` → nebloka, alie `ceil(ms/10)` kun almenaŭ unu tiko). La unuopaj
`unwind`-limoj estas en la tabelo de §3.

**Kial la pruvoj estas skribitaj tiel.** La unua plena rulo malsukcesis per
«CBMC appears to have run out of memory» ĉe `fifo_ordo_inter_fontoj` — ne per
malvera aserto. La kaŭzo estis la SAT-formulo: `Mesago::nova` kopias ĝis 64
bajtojn per buklo, kiun `unwind 70` malvolvas por ĉiu mesaĝo, kaj la vico tenas
16 erojn da 96-bajtaj mesaĝoj. Tri ŝanĝoj tenas la formulon memorebla sen perdi
la pruvatan enhavon:

1. La vicaj pruvoj uzas **malplenan datumaron**: `iu_mesago` fariĝis
   `Mesago::nova(tipo, 0, 0, &[])`. La vica ordo dependas nur de `tipo` kaj de
   la fonto; simbolaj bajtoj de la datumaro nur pligrandigis la formulon.
2. `unwind` malkreskis al **8** en la du FIFO-pruvoj (maksimume tri mesaĝoj kaj
   tri `preni_de`-paŝoj) kaj al **20** kie la pruvo trairas la plenajn 16
   fendojn (`plena_mesagujo_rifuzas`, `preno_konservas_la_invarianton`).
3. `plenlonga_mesago` kun `unwind 70` aparte pruvas la plenan 64-bajtan buferan
   limon, do la malplena datumaro ne malstreĉas tiun garantion.

Post la ŝanĝoj la pinto de CBMC ĉe la antaŭe kadukiĝinta pruvo malaltiĝis de
pli ol 3 GB al malpli ol 1 GB, kaj la tuta pordo fariĝas verda.

### 4.7 Nuba DI kaj forlaso de loka Kani (2026-11-15; reviziita 2026-11-16)

**Kial.** Plena `cargo kani` en WSL2 elĉerpis la memon de la gastiganto kaj
kelkfoje diskonektis la VM-on. La formala pordego translokiĝis al GitHub
Actions; loke oni ne plu rulas Kani-on.

**Konfiguro.** `.github/workflows/di.yml` (`DI`) ruliĝas ĉe `push` al `main`
kaj ĉe `pull_request`, kun nurlega permeso al la deponeja enhavo. Tri taskoj:

- `kontroli-kaj-konstrui` — `actions/checkout@v4`; Rust 1.100.0 per
  `dtolnay/rust-toolchain` kun la komponantoj `clippy`, `rustfmt` kaj
  `llvm-tools-preview` kaj la celoj `x86_64-unknown-none`,
  `aarch64-unknown-none`; `just` per `taiki-e/install-action`; do `just kontroli` (sen Kani) kaj `just konstrui-arm`, kiu ligas la arm64-kernon kaj
  konvertas ĝin al la plata `Image`.
- `kani-pretigi` (`needs: kontroli-kaj-konstrui`) — instalas Rust 1.100.0 kaj Kani,
  do `cargo kani -p mikerno … list --format json`; la plenaj nomoj de la
  pruvoj el `kani-list.json` pasas al la matrico kiel JSON.
- `kani-kontroli` (`needs: kani-pretigi`, matrica, `fail-fast: false`) — unu
  tasko por ĉiu pruvo: `timeout -k 10 600 cargo kani -p mikerno …
  --exact --harness <pruvo>` (10 min por pruvo) sub `timeout-minutes: 30`.
  La protokolo supreniĝas per `actions/upload-artifact@v4` eĉ ĉe malsukceso,
  do blokiĝinta pruvo ne forprenas la rezultojn de la ceteraj.

**Trovitaĵoj.**

- `dtolnay/rust-toolchain` instalas la minimuman profilon; sen eksplicitaj
  `components` mankus `rustfmt` kaj `clippy`, do DI petas ilin eksplicite.
- `just kontroli` lintas la kernon ankaŭ por `aarch64-unknown-none`; tial
  `kontroli-kaj-konstrui` instalas ambaŭ celojn (la Kani-taskoj nur
  `x86_64-unknown-none`). `just kontroli` tamen ne ligas la arm64-kernon, do
  `kontroli-kaj-konstrui` rulas ankaŭ `just konstrui-arm`; `llvm-tools-preview` provizas
  la `rust-objcopy`, kiun `tools/elf-al-image.sh` bezonas.
- La oficiala Kani-ago internŝaltas la defaŭltan ilarĉenon al `stable`; la
  paŝoj antaŭ `just kontroli` kaj `cargo kani` do restarigas `1.100.0`.
- Timita konflikto inter `rust-version = "1.100"` kaj la `nightly` de Kani
  0.68.0 ne okazis; `--ignore-rust-version` ne necesis.
- La elŝuto de la kruda protokolo per la Actions-interfaco rifuziĝis sen administraj
  rajtoj, do la stato de la runo estas legata el la taskaj kaj paŝaj statoj de
  la interfaco.

**Rezulto.** Runo
[#43725539875](https://github.com/makoto-lernanto/mikerno/actions/runs/43725539875)
ĉe komito `7e31929` (2026-11-15): ambaŭ tiamaj taskoj verdaj.
`kontroli-kaj-konstrui` daŭris ≈14 s; `kani-kontroli` ≈2 min 40 s (instalo de Kani
19 s, `just kontroli` 2 min 10 s). La matrico de §4.8 anstataŭigis tiun
unuopaĵan paŝon.

**Nova regulo.** Loke oni ne plu rulas `cargo kani`; WSL2 elĉerpiĝas je
memo. `just kontroli` nun enhavas nur la sen-Kani-ajn kontrolojn de §9.2 kaj
estas loke rulebla; Kani ruliĝas nur en DI, kiel la matrico de §4.8.

### 4.8 Nuligo de la Kani-tasko pro mempremo (2026-11-16)

**Simptomo.** Tri sinsekvaj `kani-kontroli`-taskoj (`just kontroli` ĉe la F1-puŝoj)
finiĝis per `##[error]The runner has received a shutdown signal` kaj «recipe
'verify' was terminated on line 78 by signal 15», 3 min 44 s–7 min 25 s post la
komenco de `just kontroli`. Neniu pruvo malsukcesis per aserto, kaj
`kontroli-kaj-konstrui` restis verda en ≈14 s; la antaŭa verda pordo (29 pruvoj)
daŭris 2 min 15 s.

**Kaŭzo.** La tri novaj kupaj pruvoj generis tro grandan simbolan staton: la
protokolo de la dua provo montras `60420 VCC(s), 23706 remaining after
simplification`, `Runtime Symex: 97.97 s`, kaj la nuligon ≈37 s poste, meze de
la solvado de `reenmeto_kaj_forigo`. La memlimo de `ubuntu-latest` (16 GB)
klarigas la signalon: komparebla Kani-projekto ligis la saman
`shutdown signal`/exit 143 al CBMC-pinto de 11.8 GB
([pulseengine/rivet#839](https://github.com/pulseengine/rivet/issues/839)).
Temas do pri mempremo de la gastiganto, ne pri uzanta nuligo nek pri pruva
regreso.

**Riparo.** Du tavoloj:

- La baza stato de la pruvoj fariĝis determinisma (`BAZAJ`); nur la reenmetaj
  limdatoj restas arbitraj (`u8`), kaj la kontrolo `kohera` restas. Tio
  malgrandigas la simbolan staton de la kupaj pruvoj.
- La Kani-pordo fariĝis matrica: `kani-pretigi` eltiras la pruvliston per
  `cargo kani … list --format json`, kaj `kani-kontroli` rulas unu pruvon por
  tasko kun `timeout -k 10 600` (10 min) sub `timeout-minutes: 30`. Pruvo,
  kiu blokiĝas aŭ konsumas tro da memoro, estas nuligita de `timeout` kaj
  ĝia protokolo supreniĝas kiel artefakto; la ceteraj pruvoj finiĝas sendepende
  (`fail-fast: false`). `just kontroli` restas la loka pordego sen Kani.

**Rezulto.** La unua matrica rulado ([#43819515923](https://github.com/makoto-lernanto/mikerno/actions/runs/43819515923),
komitero `a777e1e`, 2026-11-16) estis tute verda: `kontroli-kaj-konstrui` kaj
`kani-pretigi` sukcesis, kaj ĉiuj 32 pruvoj de la matrico sukcesis en 9 min 17 s
(06:50:15Z–06:59:32Z), kun po unu protokolo `kani-0`…`kani-31` kiel artefakto.
La plej longa pruvo estis `kerno::vekotemporizilo::kani_proofs::reenmeto_kaj_forigo`
(400 s, sub la limo de 600 s); la sekvaj restis sub 90 s
(`fifo_ordo_inter_fontoj` 89 s, `fonta_filtro_konservas_fremdajn` 78 s,
`fifo_ordo` 59 s). La antaŭa, haltigita rulado
([#43815916181](https://github.com/makoto-lernanto/mikerno/actions/runs/43815916181))
estis devige nuligita kaj restas `completed/cancelled`; ĝia modloko ne plu
ĝenas la pordon.

### 4.9 La retejo (2026-11-16)

**Celo.** Prezenti MiKernon per unu memstara paĝo, kiu uzas la samajn FluoU
0.1-ĵetonojn kiel la vidmontraĵo, sen nova koloro, mezuro aŭ materialo.

**Lokoj.**

- `docs/index.html` — la fonta paĝo (Esperanto).
- `docs/retejo.css` — la FluoU-ĵetonoj, la fenestra modelo, la materialoj kaj
  la respondecaj reguloj.
- `docs/retejo.js` — temo, akcentoj, lingva mapado, fenestraj regiloj,
  `Ctrl+E`-Eniro (provizora; estonte `Mito`), logaj langetoj, kopiado kaj la
  movoj.
- `docs/tradukoj/angla.js` — la angla mapada tavolo; la fontaj tekstoj restas
  Esperantaj kaj la ĉina ne eniras la retejan fonton.
- `docs/.nojekyll` — malŝaltas la Ĝekilan antaŭtrakton por la statikaj dosieroj.

**Enhavo.** La paĝo montras la realajn ISO-logojn de ambaŭ arkoj, la
frambufan haketon, la ses tavolojn, la mejloŝtonojn M0–M18, la elŝutojn kun
SHA-256 kaj la FluoU-komponantojn. La versio venas el `data-versio` kaj estas
kontrolata de `tools/versio.sh`; la elŝutaj ligiloj ankaŭ anstataŭigas la
version per JavaScript.

**Publikigo.** La celita adreso estas
[`https://makoto-lernanto.github.io/mikerno/`](https://makoto-lernanto.github.io/mikerno/).
Ĝi bezonas GitHub Pages agordita por publikigi el branĉo, kun branĉo `main` kaj dosierujo
`/docs`; tiu agordo ankoraŭ devas esti ŝaltita en la deponejaj agordoj.

**Kontroloj.** `bash tools/versio.sh --kontroli` kaj loka HTTP-servo liveras
la paĝon kaj la tradukajn dosierojn; la foliumila kontrola trairo restas farebla
sur la publikigita adreso.

---

## 5. Venontaj mejloŝtonoj

### M14 — IK-firmigo

**Celo.** Apartigi finpunktojn de fadenoj kaj firmigi la kapablomodelon.

**Taskoj (Fazoj).**

| Fazo | Versio | Enhavo |
| --- | --- | --- |
| 1 | 0.4.0 | ABI 2: `Tenilo` (indekso + generacio), `Permesoj::REVOKI/PLENAJ`, erar-kodoj `-9/-11/-28/-32/-90/-110`, vokoj 6–11 rezervitaj (`-2`), `ABI_VERSIO = 2`; A1-identeca pruvo; 13 Kani-pruvoj |
| 2 | 0.4.1 | `FinaPunktoObjekto`-tabelo `FINAJ_PUNKTOJ`, generacioj, referenckalkulo, minimuma kapablotabelo, `KREI_FINPUNKTON`(6)/`FERMI`(11); malnova vojo per defaulta finpunkto |
| 3 | 0.4.2 | Permes-kontroloj, `DONI`(7), `SENDI`/`RICEVI` transiras al `rdi` = tenilo, du-nivela generacia kontrolo, `KOMENCA_TENILO`; la fonta filtro venas kun `RICEVI_TEMPO` en Fazo 6 |
| 4 | 0.4.3 | `REVOKI`(8) tutfinpunkta, revokaj vekoj `-32`, purigo ĉe proceza detruo, `Result<Mesago, IkEraro>` |
| 5 | 0.4.4 | Plen-vica premrego (sendblokado), identeco, `-90`, `IkStatistiko` |
| 6 | 0.4.5 | `RICEVI_TEMPO`(9)/`SENDI_TEMPO`(10), `Atendo::{Senfina, Nebloka, Tempa}`, absoluta limdato por ĉiu voko, `-110`, `-11` |
| 7 | 0.4.6 | AC/CR3/EOI/ellasa revizio, pruva kompletigo, eldono |

**Decidoj (C1–C16, arbitracio de Fazo 1).**

- **C1.** `Mesago` restas 96 bajtoj; `ricevonto` estos finpunkta identigilo ekde
  Fazo 2 (hodiaŭ ĝi estas la celo-fadeno). Identigilon de la sendinta procezo
  oni aldonu per aparta voko, neniam per nova kampo.
- **C2.** La malnovaj erar-kodoj `-1/-2/-3/-4` restas porĉiam; `-3/-4` estas
  `malrekomenditaj` post M14, sed la konstantoj ne foriĝas. Novaj kodoj nur
  aldoniĝas.
- **C3.** La memorbuĝeto estas **aldona**: forigo de la malnova `MESAGUJOJ`
  kompensas la novajn tabelojn; la neta kresko estas ≤ 20 KiB.
- **C4.** `SENDI`/`RICEVI` ricevos tenilon en `rdi` en Fazo 3; la fonta filtro
  moviĝas al `RICEVI_TEMPO.rdx` (fadena id).
- **C5.** La kerno re-exportas `Permesoj`/`Kapablo` el `libmikerno`; la finpunkta
  objekto nomiĝas `FinaPunktoObjekto`, la tabelo `FINAJ_PUNKTOJ`. Oni ne
  enkonduku sinonimon `Rajtoj`.
- **C6.** La versio havas unu fonton, `[workspace.package] version`; la kerno
  legas ĝin per `env!("CARGO_PKG_VERSION")`, `limine.conf` ne portas version,
  kaj `tools/versio.sh --kontroli` (parto de `just kontroli`) kontrolas la
  dokumentojn.
- **C7.** `ŝanĝi` jam konservas `RFLAGS` (inkl. AC); la AC-aserto restas
  disciplina kaj la kialo estas la `STAC`-vivo, ne la kuntekstŝanĝo.
- **C8.** La kapablotabelo havas 8 liniojn (`[[KapabloEniro; 16]; 8]`); linio
  `0` (kerno) restas ĉiam malplena.
- **C9.** Ne ekzistas novaj plenigaĵoj; testoj pruvas nulan reston de la datumaro
  kaj la identecan transprenon.
- **C10.** Fazo 2 liveras verajn `Tenilo`-jn (minimuma kapablotabelo); Fazo 3
  ŝanĝas `rdi` al tenilo sen duobla modelo.
- **C11.** Montrila kontrolo okazas **antaŭ** tenila kontrolo; la malnovaj
  negativaj valoroj (`nurlega=-1`) restas.
- **C12.** Generacio komenciĝas ĉe `1`; indekso `< 16`; eluzita generacio
  emeritigas la fendon.
- **C13.** Buĝeto: ≈ 27 KiB da finpunktaj vicoj + 2 KiB da kapabloj + ≈ 1 KiB da
  IK-stato, minus ≈ 15 KiB da malnova `MESAGUJOJ` → neta ≈ 15 KiB.
- **C14.** `REVOKI` estas ĉiam tutfinpunkta; `rsi` devas esti `0`; `FERMI` nur
  fermas la lokan tenilon.
- **C15.** `r10 = u32::MAX` senfina; `0` nebloka (`-11`); alie
  `ceil(ms/10)` tikoj kaj `-110`.
- **C16.** `ricevi_mesagon` transiros al `Result<Mesago, IkEraro>` en Fazo 4;
  la shim de Fazo 2 validigos, ĉu la cela fadeno vivas, kaj rifuzos per `-3`.

**Progreso.**

| Dato | Fazo | Versio | Pruvo |
| --- | --- | --- | --- |
| 2026-11-13 | 1. ABI 2: `Tenilo`, permesoj, erar-kodoj, rezervitaj vokoj, A1-testo | 0.4.0 | `[ring3] procezo 4/5: idento (bone): sendinto=4/5 (uzanto asertis 999), ricevonto=4/5, resto_nula=jes`; `tenilo_kodigo_roundtrip`; 13 Kani-pruvoj; `size -A`: `.text` 76159, `.rodata` 50212, `.data` 4496, `.bss` 380928 B (bazo por la buĝeto C3) |
| 2026-11-13 | 2. `FinaPunktoObjekto`, generacioj, referencoj, kapablotabelo, `KREI_FINPUNKTON`(6)/`FERMI`(11), defaŭlta finpunkto | 0.4.1 | `[ik] tabeloj (bone): kreitaj=13, plena=true, fermitaj=13, duobla_fermo=true, nova_generacio=true, malnova_tenilo=true, kerno=true`; `[ring3] procezo 4: finpunktoj (bone): tenilo=1025, fermo=0, duobla=-9, malvalida=-9, nulaj_permesoj=-1, nekonataj_permesoj=-1, rekrea=1281, malnova=-9`; `[ring3] negativaj (F2): celo6=-3, celo0=-3`; `[fadeno 2] ricevis mesaĝon: tipo 1, ricevonto 2 (finpunkta id)`; A1 senŝanĝa; 16 Kani-pruvoj; `size -A`: `.text` 89871, `.rodata` 50724, `.data` 34808, `.bss` 364544 B — ĉefmemora kresko (`.data`+`.bss`) 13928 B ≤ 20 KiB (C3) |
| 2026-11-13 | 3. Permes-kontroloj, `DONI`(7), `SENDI`/`RICEVI` per tenilo, du-nivela kontrolo, `KOMENCA_TENILO` | 0.4.2 | `[ik] rajtoj (bone): sendi per RICEVI-nura → -1, ricevi per SENDI-nura → -1`; `[ik] izolado de kapabloj (bone): procezo 7 uzas tenilon de procezo 6 → -9`; `[ik] dono (bone): rajtoj 0b011 → 0b010, eskalado 0b100 → -1`; `[ring3] procezo 4: rajtoj (bone): sendi_per_ricevi=-1, ricevi_per_sendi=-1`; `[ring3] negativaj (F3): tenilo6=-9, tenilo0=-9`; `[sistemvoko] Ring 0-memtesto: skribo=0, nekonata=-2, nula=-1, troa=-1, cedo=0, krei=-28, doni=-9, fermi=-9`; `[ik] tabeloj`, A1 kaj `finpunktoj` senŝanĝaj; 20 Kani-pruvoj; `size -A`: `.text` 86431, `.rodata` 51288, `.data` 36056, `.bss` 364544 B — ĉefmemora kresko (`.data`+`.bss`) 1248 B (C3) |
| 2026-11-14 | 4. `REVOKI`(8) tutfinpunkta, revokaj vekoj `-32`, purigo ĉe proceza detruo, `Result<Mesago, IkEraro>` | 0.4.3 | `[ik] revoko (bone): sen_revoki=-1, revoko=0, donita=-9, fermo=-9, duobla=-9`; `[fadeno 1] revokis la defaŭltan finpunkton de fadeno 3`; `[fadeno 3] revoka veko (bone): kodo -32`; `[ik] purigo de procezo 4/5: 1 kapabloj liberigitaj, 1 finpunktoj revokitaj`; `[ring3] procezo 4: revoko (bone): argumento=-1, sen_revoki=-1, fermo_sen=0, revoko=0, sendo=-9, fermo=-9, duobla=-9`; `[sistemvoko] Ring 0-memtesto: …, revoki=-9, revok_argumento=-1`; 21 Kani-pruvoj; `size -A`: `.text` 92799, `.rodata` 51808, `.data` 36104, `.bss` 364544 B — ĉefmemora kresko (`.data`+`.bss`) 48 B (C3) |
| 2026-11-14 | 5. Sendblokado ĉe plena vico, masko de sendintoj, `-90`, `IkStatistiko` | 0.4.4 | `[fadeno 1] premrego (bone): 17 mesaĝoj senditaj; la lasta atendis liberan lokon`; `[fadeno 2] premrego (bone): 17 mesaĝoj, idento sendinto=1, ricevonto=2`; `[fadeno 2] sendblokada revoko (bone): enmetitaj=16, kodo=-32`; `[fadeno 3] revokis finpunkton 1; vekas la blokitan sendanton`; `[sistemvoko] Ring 0-memtesto: …, tro_longa=-90`; `[ik] statistiko (bone): senditaj=37, ricevitaj=21, premregoj=2, revokoj=7`; `[kerno] fina stato: 5 mortintaj, 0 pretaj, 0 blokitaj`; 23 Kani-pruvoj; `size -A`: `.text` 99551, `.rodata` 52336, `.data` 36424, `.bss` 364544 B — ĉefmemora kresko (`.data`+`.bss`) 320 B (C3) |
| 2026-11-14 | 6. `RICEVI_TEMPO`(9)/`SENDI_TEMPO`(10), `-110`, `-11`, absoluta limdato | 0.4.5 | `[fadeno 4] tempolimoj (bone): nebloka=-11 (0 tikoj), tempolimo=-110 (5 tikoj), mesaĝo=0/tipo 90, senfina=0/tipo 91`; `[ring3] procezo 5: tempolimoj (bone): nebloka=-11, tempolimo=-110, sendo=0, senfina=0, troa=-1`; `[ring3] procezo 6: tempolimoj (bone): nebloka=-11, tempolimo=-110, sendo=0, senfina=0, troa=-1`; `[memoro] izolado (bone): 64809 kadroj antaŭ, 64809 post`; `[ik] statistiko (bone): senditaj=41, ricevitaj=25, premregoj=2, revokoj=7, tempolimoj=3`; `[kerno] fina stato: 6 mortintaj, 0 pretaj, 0 blokitaj`; 26 Kani-pruvoj; `size -A`: `.text` 102719, `.rodata` 52908, `.data` 36488, `.bss` 380928 B — ĉefmemora kresko (`.data`+`.bss`) 16448 B ≤ 20 KiB (C3) |
| 2026-11-15 | 7. AC/CR3/EOI/ellasa revizio, pruva kompletigo, eldono | 0.4.6 | `[memoro] AC (bone): fermita=true, malfermita=true, interna=true, post_interna=true, fermita_post=true, ŝtelita=true, purigita=true`; `[kerno] gardoj (bone): malŝalto=true, interna=true, restarigo=true (IF antaŭe false)`; `[memoro] CR3 (bone): PCID=malŝaltita, aktiva=0x196000, kongrua=true, aliniita=true`; `[sistemvoko] Ring 0-memtesto: …, tro_longa=-90, sen_eoi=true`; `[apic] EOI (bone): 1000 konfirmoj, 1000 tikoj`; `[memoro] izolado (bone): 64799 kadroj antaŭ, 64799 post`; `[ik] statistiko (bone): senditaj=41, ricevitaj=25, premregoj=2, revokoj=7, tempolimoj=3`; `[kerno] fina stato: 6 mortintaj, 0 pretaj, 0 blokitaj; 1000 tikoj`; PVH kaj ISO startoj verdaj; 29 Kani-pruvoj (3 novaj: `eoi_nur_por_apic_vektoroj`, `cr3_radiko_estas_paĝe_aliniita`, `cr3_radiko_forprenas_nur_la_malaltajn_bitojn`); `size -A`: `.text` 103471, `.rodata` 53476, `.data` 36488, `.bss` 380928 B — ĉefmemora kresko (`.data`+`.bss`) 0 B (C3) |

**Rezultoj de la revizio (Fazo 7).**

- **AC.** `firmigo::testi_ac` pruvas la tutan vivciklon de `StacGardo`: fermita
  antaŭ la unua gardo, malfermita dum ĝi, nedaŭra nestita fenestro, restarigo
  ĉe `Drop`, kaj forigo de `AC`, kiun Ring 3 estus lasinta ŝaltita. La
  interrompa dispetĉo haltas, se `CLAC` ne fermus la fenestron.
- **CR3.** La ISO-starto montris, ke Limine lasas `CR4.PCIDE` (bito 17)
  ŝaltita. `pagado::pravalorizi` nun reŝargas `CR3` kun nula PCID kaj poste
  malŝaltas `PCIDE`; `ŝalti_radikon` haltas, se la bito restus ŝaltita, ĉar la
  modelo «unu radiko = unu TLB-spaco» dependas de ĝi. `testi_cr3` komparas la
  aktivan radikon kun la ĵus konstruita `PML4`.
- **EOI.** `apic::konfirmi` kalkulas la senditajn `EOI`-ojn; la fina raporto
  komparas ilin kun la temporizilaj tikoj. `estas_apic_vektoro` estas pruvita
  por la tuta `usize`-amplekso, do `int 0x80` neniam ricevas `EOI`-on.
- **Ellaso.** `testi_gardojn` pruvas la nestitan `InterrompaGardo`-vojon, kaj
  `Kunhavigita::akiri` dokumentas sian sekurecan invariancon.
- **Grando.** `skribi_formaton` estas `#[inline(never)]`: la novaj diagnozaj
  vokoj alie duobligis la formatigan vojon (`.text` 117007 → 103471 B).

**Akceptkriterioj.** Mesaĝo portas `sendinto`/`ricevonto`; revoko malvalidigas
kapablon kaj vekas la blokitan ricevanton per `-32`; proceza detruo liberigas
ĝiajn kapablojn kaj forprenas ĝiajn finpunktojn; plena vico premegas la
sendanton anstataŭ perdi mesaĝon (nur la ripoza kunteksto ricevas `-4`);
tro longa mesaĝo redonas `-90`; tempolimo vekas la ricevanton per `-110`, nebloka
voko redonas `-11` sen cedi, kaj `r10 = u32::MAX` blokas ĝis mesaĝo aŭ revoko.

---

### M16 — Temporiziloj kaj planado

**Celo.** Dormantaj fadenoj, prioritatoj kaj mezuro de plenumtempo.

**Taskoj.** Unuopa vekotemporizilo (min-kupo), `dormi(md)`, prioritatoj aŭ
loteria planado, statistiko, eventuala SMP-esploro kun IPI.

**Akceptkriterioj.** Fadeno dormanta 100 ms vekiĝas en tolero; la planisto ne
malsatigas fadenojn; la ripoza kunteksto restas senŝanĝa.

**Fazo 1 (0.6.0).** La dorma vojo funkcias per unuopa min-kupo:

- `Vekotemporizilo` (`kerno/vekotemporizilo.rs`) — kupo de `(limdato, FadenoId)`
  kun `pozicioj`-tabelo por O(log n) enigo, forigo kaj elpreno sen alokado; la
  kapacito estas `MAKSAJ_FADENOJ`. Reenmeto ĝisdatigas la limdaton anstataŭ
  duobligi la eron.
- `Planisto` — la tabelo `limdatoj` anstataŭiĝis per `vekoj`.
  `venkigi_tempolimojn` elprenas ĉiujn limdatojn `≤ nuna tiko` el la pinto,
  mezuras la prokraston kaj vekas la fadenojn. `veki` apartigis
  `veki_sen_limdato`, do veko pro operacio ankaŭ forigas la eron el la kupo.
- `planisto::dormi(millisekundoj)` — `ceil(ms/10)` tikoj en ciklo sub
  `InterrompaGardo`; la ripoza kunteksto estas rifuzata per `false`. Fremda
  veko aŭ tempolimo lasas markon, kiun la ciklo konsumas, do la sekva IK-voko
  ne konfuzas ĝin kun propra limdato.
- `SYS_DORMI = 12` (`rdi` = daŭro en ms), `libmikerno::syscall::dormi`,
  Ring 3-testo en `horizonto`; `ABI_VERSIO` restas `2`.
- `PlanistaStatistiko` (`ŝaltoj`, `limdataj_blokoj`, `vekigoj`,
  `maks_vekprokrasto`) kaj `veturoj` por la fina raporto.
- Kani-pruvoj (nur DI): `pinto_estas_minimumo`, `elpreni_ordonas`,
  `reenmeto_kaj_forigo`.

**`arm64`-stabileco.** La longdaŭra malstabileco de la `arm64`-startoj
montriĝis kiel frambuffera skribo unu vicon preter la fino: ĝi difektis la
`TTBR0`-radikon de procezo 5 kaj kaŭzis esceptan buklon. La kaŭzo estis la
`arm64`-vojo de `skribi_formaton`, kiu — malsimile al la `x86_64`-vojo — ne
malŝaltis interrompojn; la temporizila traktilo do povis skribi en la konzolon
meze de `novlinio`, kiam `y` estis transire egala al `vicoj`. La vojo nun
prenas `InterrompaGardo`, kaj la konzola stato nur ŝanĝiĝas atome.

**Pruvo.**

```text
[fadeno 4] dormo (bone): petitaj 30 ms, pasintaj 4 tikoj (kodo 0)
[fadeno 2] dormo (bone): petitaj 50 ms, pasintaj 5 tikoj (kodo 0)
[fadeno 3] dormo (bone): petitaj 70 ms, pasintaj 7 tikoj (kodo 0)
[fadeno 1] dormo (bone): petitaj 100 ms, pasintaj 10 tikoj (kodo 0)
[ring3] procezo 5: dormo (bone): kodo 0
[ring3] procezo 6: dormo (bone): kodo 0
[planisto] statistiko (bone): ŝaltoj=111, limdataj_blokoj=10, vekigoj=9, maks_vekprokrasto=0 tikoj
[planisto] veturoj: 21 21 19 18; minimumo 18
```

**Akceptkriterioj — pruvitaj.**

| Kriterio | Stato | Pruvo |
| --- | --- | --- |
| Fadeno dormanta 100 ms vekiĝas en tolero | **Pruvita** | `petitaj 100 ms, pasintaj 10 tikoj`; `maks_vekprokrasto=0` |
| La planisto ne malsatigas fadenojn | **Pruvita** | ĉiu testa fadeno veturis ≥ 10 fojojn (la raporto bezonas `minimumo ≥ 10`); observitaj minimumoj 17–18 |
| La ripoza kunteksto restas senŝanĝa | **Pruvita** | `planisto::dormi` redonas `false` por `RIPAZA_ID` (voko `-1`); la ripoza ciklo kaj ĝia stako ne ŝanĝiĝis |

Ripetitaj `arm64`-startoj de la fina bildo (QEMU `virt`, `ramfb`) atingas la
saman finan staton sen escepto; la `x86_64`-vojoj restas senŝanĝaj.

**Versio.** 0.6.0 (Fazo 1).

**Fazo 2 (0.6.1) — prioritata planado.** La preta vico restas FIFO; la
prioritato pezas la tempopecon anstataŭ doni absolutan antaŭecon:

- `Prioritato::{Malalta, Normala, Alta}` en `kerno/objekto.rs` donas respektive
  1, 2 kaj 4 tikojn (nominale 10, 20 kaj 40 ms). Novaj fadenoj defaŭlte havas
  `Normala`; la kerno fiksas la prioritaton antaŭ registrado. Neniu nova
  sistemvoko, loteria planado aŭ SMP-ŝanĝo; `ABI_VERSIO` restas `2`.
- `Planisto.restaj_tikoj` malpliiĝas ĉe temporizila planado. Je elĉerpiĝo la
  nuna fadeno iras al la vosto; `cedi` tuj fordonas la restaĵon. Blokado kaj
  morto tuj elektas alian pretan fadenon, kaj veko aldonas ĉe la vosto sen
  duobligo; ĉiu nova elekto rekomencas la tempopecon.
- Konstante preta fadeno atendas maksimume unu rondon: kun ok fadenoj,
  `MAKSA_PRETA_ATENDO = (8 − 1) × 4 = 28` tikoj. Tio estas limo en regulaj
  planistaj tikoj, ne senkondiĉa murhorloĝa garantio dum interrompoj estas
  malŝaltitaj. Alta prioritato ne povas senlime preterpasi malaltan.
- Limdata veko kaj efektiva kurado estas apartaj: `maks_vekprokrasto` plu
  devas esti ≤ 1 tiko. La dormaj kaj IK-tempolimaj revenprovoj toleras ĝis
  unu preta rondo plus unu tiko; la antaŭa unu-tika revenlimo de Fazo 1 ne
  validas por la novaj tempopecoj. La komenca ISO-provo montris 30-ms dormon
  revenantan post 5 tikoj malgraŭ senprokrasta veko; tio motivis la apartigon.
- La premrega ricevanto atendas la unuan registritan sendblokadon antaŭ ol
  malplenigi la vicon, do la testo ne dependas de la tempopecoj. Kvar kernaj
  fadenoj kun prioritatoj malalta/normala/alta/malalta trapasas barilon sen
  cedo kaj poste laboras dum komuna 32-tika intervalo; sen reala antaŭokupo
  la barilo ne povas pasi.

**Lokaj kontroloj (2026-11-16).** `just kontroli` sukcesis sen Kani: versio,
formato, Clippy por ambaŭ arkitekturoj kaj laborloka konstruo. `just iso` kaj
`just iso-arm` estis rekonstruitaj; QEMU-praŝarĝoj per `x86_64` PVH kaj BIOS-ISO,
`arm64` Image kaj UEFI-ISO ĉiuj atingis la finan staton sen `MALBONE`:

```text
[planisto] prioritata rondo 1:2:4 (bone):
[planisto] maksimuma preta atendo 28 tikoj (bone):
[planisto] egalprioritata ordo (bone):
[planisto] tuja cedo (bone):
[planisto] blokado, ripozo kaj veko (bone):
[fadeno 1] antaŭokupa laboro sen cedo (bone): 4 observitaj tikoj
[fadeno 2] antaŭokupa laboro sen cedo (bone): 8 observitaj tikoj
[fadeno 3] antaŭokupa laboro sen cedo (bone): 16 observitaj tikoj
[fadeno 4] antaŭokupa laboro sen cedo (bone): 4 observitaj tikoj
[fadeno 1] dormo (bone): petitaj 100 ms, pasintaj 10 tikoj (kodo 0)
[ik] statistiko (bone): senditaj=41, ricevitaj=25, premregoj=2, revokoj=7, tempolimoj=3
[kerno] fina stato: 6 mortintaj, 0 pretaj, 0 blokitaj; 1000 tikoj
```

La kvar vojoj raportis `maks_vekprokrasto=0`; la SMAP/PAN-fina provo atingis
la atenditan protektan escepton. `x86_64` uzas `-no-shutdown`, do la testilo
finis QEMU post la fina raporto; `arm64` finiĝis per la kerna halto.

**Formala pordego — sukcesinta.** La rulado
[DI #7](https://github.com/makoto-lernanto/mikerno/actions/runs/43835937076)
por komitero `aa4d933` finiĝis per `completed/success` la 2026-11-16:
39/39 taskoj sukcesis (`kontroli-kaj-konstrui`, `kani-pretigi` kaj 37 Kani-pruvoj),
sen malsukceso aŭ nuligo. La tuta rulado daŭris 10 min 51 s
(09:00:39Z–09:11:30Z).

Ĉiuj kvin novaj determinismaj scenaroj en `planisto::kani_prioritatoj`
sukcesis: `pezaj_tempopecoj_sen_malsatigo`,
`malalta_prioritato_atendas_maksimume_unu_rondon`, `egalprioritataj_rotacias`,
`cedo_liberigas_tempopecon` kaj `blokitaj_ne_kuras_kaj_veko_ne_duobligas`.
Ili kontrolas la montritajn fiksajn statojn (inkluzive ok fadenojn ĉe la plej
longa ronda atendo), ne ĉiujn arbitrajn blokadajn historiojn. La DI-matrico
eltiris ilin aŭtomate; Kani ne estis rulita loke. La formala pordego de
M16 Fazo 2 estas verda.

**Versio.** 0.6.1 (Fazo 2).

**Fazo 3 (0.6.2) — specimena plenumtempokontado.** La restanta M16-celo
«mezuro de plenumtempo» ricevas po-fadenan tik-specimenadon, ne precizan
murhorloĝan mezuron. La prioritata politiko, ABI 2 kaj unuprocesora modelo
restas senŝanĝaj; SMP/IPI ne apartenas al ĉi tiu fazo.

- `Planisto.plenumtikoj` enhavas saturiĝantan `u64`-nombrilon por ĉiu fadeno.
  Ĉiu temporizila voko de `elekti_por_tiko(false)` aldonas unu specimenon al
  la interrompita fadeno antaŭ la elekto de ĝia posteulo. Cedo, registrado,
  blokado, veko kaj morto mem ne aldonas specimenon; `veturoj` plu nombras
  elektojn, ne tempon.
- Indekso 0 inkluzivas la ripozan kaj praŝarĝan kuntekston antaŭ la unua
  planista lanĉo. Blokitaj kaj mortintaj fadenoj konservas siajn nombrojn,
  sed ne ricevas novajn specimenojn dum aliaj kuntekstoj kuras.
- `planisto::plenumtikoj()` kopias la tabelon sub `InterrompaGardo`. La fina
  raporto legas la horloĝon kaj la specimenojn kun interrompoj malŝaltitaj,
  montras ĉiun fadenon kaj kontrolas `sumo == horloĝo`, pozitivan ripozan
  nombron kaj pozitivan nombron por ĉiu el la kvar laborantaj testfadenoj.
- Ambaŭ temporiziloj liveras nominale 100 Hz: unu specimeno reprezentas
  proksimuman 10-ms plenumtempon. Mallonga laboro inter interrompoj povas
  ricevi nul specimenojn; ne estas sub-tika precizeco aŭ apartigo inter
  uzantreĝima kaj kernreĝima laboro de la sama fadeno. Periodoj perditaj dum
  malŝaltitaj interrompoj ne estas rekonstruitaj. La sumkontrolo validas
  antaŭ saturiĝo aŭ horloĝa superfluo, ne kiel senfina tempa garantio.

**Lokaj kontroloj (2026-11-16).** `just kontroli`, `just iso` kaj `just iso-arm`
sukcesis. La kvar QEMU-vojoj atingis la finan staton kun 6 mortintaj,
0 pretaj kaj 0 blokitaj fadenoj, sen `MALBONE`; la SMAP/PAN-provoj atingis
la atenditajn protektajn esceptojn. La 100-ms dormo revenis post 10 tikoj,
`maks_vekprokrasto=0`, kaj la antaŭokupa provo plu observis 4/8/16/4 tikojn.

| Vojo | Sumo / horloĝo | Laboraj specimenoj | Ripozaj specimenoj |
| --- | --- | --- | --- |
| `x86_64` PVH | 1000 / 1000 | 64 | 936 |
| `x86_64` BIOS-ISO | 1000 / 1000 | 68 | 932 |
| `arm64` Image | 1000 / 1000 | 73 | 927 |
| `arm64` UEFI-ISO | 1000 / 1000 | 75 | 925 |

La mezuroj venas de la ripetita loka kontrolo post la pruva riparo (2026-11-16);
ili anstataŭas la unuajn valorojn de la sama tago. La malsamaj distribuoj estas
realaj observaĵoj de apartaj praŝarĝoj, ne postulo de bit-identa tempo inter
arkitekturoj. La tri lokaj kontadaj provoj sukcesis: tempopecoj donas 2/4/8
specimenojn dum 14 tikoj; ripetaj cedoj ŝanĝas elekton sed ne la specimenaron;
ripozo, blokita stato, veko kaj morto konservas la ĝustan atribuon en fiksa
scenaro.

**Malsukcesa DI #8 kaj riparo (2026-11-16).** La unua Fazo 3-puŝo
(`f0781f5`) trafis ruĝan `kani-kontroli`: `tempopecoj_konservas_plenumtikojn`,
`cedo_ne_kreas_tempon` kaj `ripozo_kaj_blokado_apartigas_tempon` malsukcesis
ĉe `memcmp.unwind.0`. La tuta-tabela egaleco de naŭ `u64`-oj malvolviĝis al
72-bajta `memcmp`, kiu postulis pli ol la fiksitaj 12 aŭ 16 malvolvaj paŝoj;
la kvara, simbola pruvo sukcesis. La antaŭa konkludo, ke la videblaj
Rust-bukloj difinas la malvolvan bezonon, estis troa: la kompilo aldonas sian
propran bajtan buklon.

Aldone `vekotemporizilo::kani_proofs::reenmeto_kaj_forigo` estis nuligita de la
600-sekunda `timeout`: la unua voko de la solvilo daŭris 532 sekundojn kaj la dua
ne finiĝis antaŭ la limo. En la antaŭa verda DI la sama pruvo bezonis proksimume
9 minutojn kaj 31 sekundojn, do temas pri randa plenumtempo, ne pri nova
kontraŭekzemplo; en tiu antaŭa DI ĉiuj asertoj estis verdaj.

Por ne lasi ruĝan `main`, `f0781f5` estis malfarita per la nemaliga reverta
komit `1407460`, kiu revenigis la validigitan Fazon 2 (`0.6.1`) al la deponejo.
Poste la Fazo 3-ŝanĝoj estis reaplikitaj loke kaj la tri egalecoj anstataŭitaj
per la helpa `samaj_plenumtikoj`, kiu komparas naŭ erojn per skalaraj komparoj.
Ĉiuj asertoj, la naŭ kontadaj eroj (inkluzive nuloj) kaj la malvolvaj kontroloj
restas; neniu kontrolo estis malŝaltita aŭ malstreĉita.

**Formala pordego — verda (DI #10).** La riparita Fazo 3 estis puŝita kiel
`70e1e3e`; DI #10 (rulo 43843897740) finiĝis sukcese: 43 el 43 laboroj kaj 41
Kani-pruvoj, de 2026-11-16 09:53:51Z ĝis 10:02:52Z (9 min 1 s). La kvar novaj
pruvoj en `kani_plenumtempo` sukcesis:
`tempopecoj_konservas_plenumtikojn`, `cedo_ne_kreas_tempon`,
`ripozo_kaj_blokado_apartigas_tempon` kaj
`nur_la_nuna_nombrigilo_ŝanĝiĝas_kaj_saturiĝas`. La unuaj tri reuzas la
fiksajn lokajn scenarojn; la lasta kontrolas arbitran nunan identigilon kaj
arbitrajn `u64`-nombrilojn por atribuo kaj saturiĝo de unu specimeno. La
bloka/morta scenaro rekte modelas statŝanĝojn, ne la tutajn kuntekstŝaltajn
vojojn. La kupa pruvo `vekotemporizilo::kani_proofs::reenmeto_kaj_forigo`,
kiu en DI #8 trafis la templimon, sukcesis ĉi-foje en 7 min 48 s. Kani ne estis
rulita loke; la DI-matrico eltrovis ĉiujn pruvojn aŭtomate. La formala pordego
de M16 Fazo 3 estas verda.

**Versio.** 0.6.2 (Fazo 3).

---

### M17 — Konservado, dosiersistemo kaj ŝelo

**Celo.** Transformi MiKerno-n el nura demonstracio al uzebla sistemo per aldono de daŭra konservado, dosierujaj operacioj kaj uzant-interaga ŝelo.

**Taskoj.** `virtio-blk`-pelilo, simpla nurlega dosiersistemo (SNDS), uzantspaca kupa alokilo, ELF-ŝargo el dosiero (`exec`), kaj ŝelo kun la devigaj Esperantaj komandnomoj (`eldiri`, `listi`, `legi`, `procezoj`, `mortigi`, `ruli`).

#### 1. Fazoj de Efektivigo

##### Fazo 1: `virtio-blk` pelilo kaj PCI-bazo (Versio 0.7.0)
*   **Enhavo**:
    *   Minimuma PCI-konfigura spaca legilo (nur por QEMU).
    *   `virtio` moderna interfaco (MMIO por `arm64`, PCI por `x86_64`).
    *   Virtio-vicoj (Virtqueues) kun DMA-bazitaj priskribiloj.
    *   Bloka legi/skribi-interfaco en la kerno (`kerno/konservado/virtio_blk.rs`).
*   **Decidoj**:
    *   C17.1: Ni subtenas nur `virtio-blk` por QEMU; reala SATA/NVMe restas por post 1.0.
    *   C17.2: DMA-bufruj estas alokitaj en la kerna spaco kaj mapitaj al la aparato; ni uzas `UnsafeCell` kun striktaj `core::sync::atomic` bariloj.

##### Fazo 2: Simpl NurLega Dosiersistemo - SNDS (Versio 0.7.1)
*   **Enhavo**:
    *   Propra, ekstreme simpla nurlega dosiersistemo (`SimplaDosiersistemo`).
    *   Strukturo: Superbloko, Inoda tabelo, Dosierujaj eroj (nomo + inoda indekso), Datumblokoj.
    *   Ilo `tools/krei_snds_img.sh` por paki dosierujon en `mikerno.snds` dum konstruo.
    *   Kerna interfaco: `malfermi`, `legi`, `listi_dosierujon`.
*   **Decidoj**:
    *   C17.3: Ni evitas FAT12/16 pro ĝia komplekseco kaj bajtordaj problemoj. SNDS estas grandendiana aŭ etendiana laŭ la gastiganto (defaŭlte etendiana).
    *   C17.4: Maksimuma dosierlongo estas 64 KiB por eviti komplikajn nekontiguajn blokojn en ĉi tiu fazo.

##### Fazo 3: Uzantspaca kupo kaj `exec` (Versio 0.7.2)
*   **Enhavo**:
    *   Nova sistemvoko `SYS_BRK` (13) por etendi la uzantan datumsegmenton.
    *   Simpla `bump` aŭ `free-list` kupa alokilo en `horizonto/src/biblioteko.rs`.
    *   Nova sistemvoko `SYS_EXEC` (14): legas ELF-dosieron el SNDS, validigas ĝin, kreas novan adresan spacon, kaj lanĉas ĝin en Ring 3.
*   **Decidoj**:
    *   C17.5: `SYS_EXEC` anstataŭigas la nunan procezon (kiel `execve`), ne kreas novan (kiel `fork`). `fork` restas kiel teknika ŝuldo.
    *   C17.6: La ŝelo uzas `SYS_BRK` por allokigi bufrujn por enigaj linioj.

##### Fazo 4: Ŝelo kaj bazaj komandoj (Versio 0.7.3)
*   **Enhavo**:
    *   La `horizonto` bildo fariĝas plena ŝelo (`horizonto/src/ligo.rs`).
    *   La prompto de la objekta ŝelo estas `ligo:~>`; ĝi sekvas la filozofion de PowerShell 7: la fluo portas objektojn/rekordojn, ne tekston, kaj `listi` montras koloran tabelon laŭ la ĵetonoj de FluoU.
    *   Lega bufrujo el COM1 (seria konzolo) aŭ frambuffera klavaro (se havebla).
    *   Komandoj:
        *   `eldiri <teksto>`: presas al la konzolo.
        *   `listi [dosierujo]`: listigas dosierojn en SNDS.
        *   `legi <dosiero>`: legas kaj presas dosieran enhavon.
        *   `procezoj`: montras la procezan tabelon (`SYS_PROCEZOJ`).
        *   `mortigi <id>`: mortigas procezon per `SYS_MORTIGI`; `pid ≤ 1` estas protektita.
        *   `ruli <dosiero>`: kreas novan procezon el SNDS (`SYS_LANĜI`).
*   **Decidoj**:
    *   C17.7: La maksimuma SNDS-dosiero altiĝas de 64 KiB al 128 KiB. La ŝelo
        kaj ĝia helpteksto ne plu eniras la malnovan limon; la bildo aŭtomate
        kreskas al 256 sektoroj (128 KiB) kaj la memprovo restas valida.
    *   C17.8: `ruli` kreas novan procezon per aparta `SYS_LANĜI` (20), ne
        anstataŭigas la ŝelon per `SYS_EXEC`; tiel la ŝelo pluvivas la lanĉon
        kaj `mortigi` povas fermi la novan procezon. `SYS_EXEC` restas por la
        bildoj, kiuj mem transprenas la procezon.

##### Fazo 5: Arkitektura adaptado kaj purigo (Versio 0.7.4)
*   **Enhavo**:
    *   Certigi, ke `virtio-blk` MMIO-vojo funkcias sur `arm64` (QEMU `virt`).
    *   Ĝisdatigi `just kontroli` kaj DI por inkluzivi la novajn komandojn.
    *   Forigi provizorajn `#[allow]`-ojn aldonitajn dum M17.

#### 1a. Fazo 1 — efektivigita (0.7.0)

**Transporta tavolo.** `arkitekturo/komuna.rs` difinas `VirtioTransporto`,
`VirtioSpeco` kaj la modernajn (virtio 1.0) registrajn alirojn: trajtoj, vico,
priskribilaj adresoj, sciigo kaj kapacito. Sur `x86_64` la aparato troviĝas per
la PCI-konfigura spaco (`pci.rs`); la modernaj kapabloj (`common`, `notify`,
`device`) estas legataj el la kapablolisto, la BAR-oj mapiĝas nekaŝeble, kaj la
buso/memoro-mastroj ŝaltiĝas. Sur `arm64` la MMIO-fenestro
`0x0a000000..0x0a004000` estas mapita nekaŝeble kaj la unua moderna
`virtio-blk` (`magio=0x74726976`, versio 2, aparato 2, vendor QEMU) elektiĝas.
QEMU bezonas `-global virtio-mmio.force-legacy=false`, alie la MMIO-aparato
restas en la malnova (versio 1) formato.

**Pelilo.** `mikerno/src/konservado/virtio_blk.rs` interkonsentas nur
`VIRTIO_F_VERSION_1`; aliaj trajtoj estas rifuzataj per malplena
`driver_features`. Unu vico de ok priskribiloj uziĝas por la petoj. La vico,
la disponebla ringo, la uzita ringo kaj la petbufro (16-bajta `BlkKapo`,
1-bajta stato, 512-bajta datumoj) loĝas en kvar sinsekvaj kadroj el
`KadraAlokilo`; la aparato atingas ilin per la identeca mapo. Antaŭ la sciigo
kaj antaŭ ĉiu legado de la uzita ringo staras `fence(Ordering::SeqCst)`.
`Vico` mem tenas la sekvan disponeblan indekson kaj la lastan uzitan indekson;
ĉiu peto atendas la uzitan ringon per polusa buklo (sen interrompoj).

**Kerna interfaco kaj memprovo.** `legi_blokon`, `skribi_blokon` kaj `kapacito` estas
la publika surfaco; la pelilo komenciĝas post la temporizilo. La memprovo legas
la unuajn 64 sektorojn, kalkulas FNV-1a-haketon, skribas determinisman ŝablonon
en la lastan sektoron, relegas ĝin, kaj restarigas la originalon. En Fazo 1 la
diskon (64 sektoroj po 512 bajtoj) kreis `tools/krei-testan-diskon.sh`; ekde
Fazo 2 la sama memprovo ruliĝas sur la SNDS-bildo (128 sektoroj). Ambaŭ
QEMU-scripts aldonas `-drive`/`-device` kaj `-snapshot`, do la memprovo ne
ŝanĝas la dosieron.

**Lokaj rezultoj (2026-11-16, versio 0.7.0).**

| Vojo | Transporto | Linio |
| --- | --- | --- |
| x86_64 PVH | PCI | `[konservado] virtio-blk: 64 sektoroj legitaj, haketo 0xc139f0538c943325` |
| x86_64 BIOS-ISO | PCI | sama haketo; `skribo kaj restarigo (bone)` |
| arm64 Image | MMIO | sama haketo; `skribo kaj restarigo (bone)` |
| arm64 UEFI-ISO | MMIO | sama haketo; `skribo kaj restarigo (bone)` |

**Riparitaj kaŭzoj dum la unua efektivigo.** La PCI-kapablolisto ne estas
sinsekva (0x98 → 0x84 → 0x70 …), do la serĉo ne rajtas postuli kreskantan
montrilon. La disponebla indekso devas progresi post ĉiu peto; alie la aparato
vidas nur la unuan peton. La MMIO-statuso estas 32-bita registro: bajta
legado/skribado ne atingas la aparaton, do la trajta interkonsento ŝajne
malsukcesas. Ĉiuj tri estis riparitaj kaj la kvar vojoj estas verdaj.

**Nuba DI (2026-11-16).** Runo
[#43886335925](https://github.com/makoto-lernanto/mikerno/actions/runs/43886335925)
ĉe `0e5b627`: ĉiuj 43 taskoj verdaj — `kontroli-kaj-konstrui` (19 s, inkluzive
`just konstrui-arm` por la plata arm64-`Image`), `kani-pretigi` (53 s) kaj 41
pruvoj de la matrico `kani-kontroli`; la tuta runo daŭris 11 min 45 s. La
arm64-bildo do estas ankaŭ ligata kaj konvertata en DI, ne nur lintata.

**Eldono.** [`v0.7.0`](https://github.com/makoto-lernanto/mikerno/releases/tag/v0.7.0)
enhavas `mikerno.iso` (SHA-256
`4afacbc00239d7b9127cdb2b8086ba66ce1d961722f7493281e96f8fc00e0897`) kaj
`mikerno-arm.iso` (SHA-256
`af405daa3aa573315902382c2fe30d744cd3fbc541a26dc9b858cc8c1c364de4`).

**Limigoj.** Nur modernaj transportoj, nur QEMU, polusa atendo, unu vico, po
unu sektoro per peto, sen IOMMU (M19) kaj sen interrompoj por la fino de DMA.

**Versio.** 0.7.0 (Fazo 1).

#### 1b. Fazo 2 — efektivigita (0.7.1)

**Bildo.** SNDS uzas 512-bajtajn sektorojn kaj etendianajn kampojn. La
superbloko en sektoro 0 portas la magion `SNDS`, version 1, la sektoran kaj
inodan grandojn, la nombron da inodoj, la inodan tabelon, la unuan datuman
sektoron, la kapaciton kaj la radikan inodon. Inodo (16 bajtoj) enhavas la
tipon (dosiero aŭ dosierujo), la grandon, la unuan sektoron kaj la nombron da
sektoroj. Dosieruja ero (64 bajtoj) konsistas el 4-bajta inoda indekso kaj
60-bajta nomo kun fina NUL; nomoj do havas maksimume 59 bajtojn. Dosiero estas
kontigua ene de sia inodo, kaj la maksimuma grando estas 64 KiB (C17.4). La
bildo havas almenaŭ 128 sektorojn.

**Pakilo.** `tools/krei_snds_img.sh` legas dosierujon (defaŭlte
`mikerno/snds/`) kaj skribas `target/mikerno.snds`; `just snds` faras la samon
sen QEMU. La skripto estas unu `python3`-programo en la ŝelo: ĝi ordigas la
erojn laŭ UTF-8-bajtoj, rifuzas nomojn pli longajn ol 59 bajtoj, dosierojn pli
grandajn ol 64 KiB kaj ne-regulajn dosierojn, kaj plenigas la reston per nuloj.
Du ruloj produktas identan bildon (SHA-256
`36a7ebd9cc08d2a729025fb285810ea85ec9ec2feac366f041be824f638d4e52`).

**Kerna interfaco.** `konservado/snds.rs` validigas la superblokon kaj ĉiun inodon
(limoj, sektornombro, tipo) antaŭ ol uzi ĝin. `SimplaDosiersistemo` tenas la
superblokon post la muntado; `malfermi(vojo)` solvas relativan vojon ekde la
radika inodo, `legi(dosiero, bufro)` postulas sufiĉe grandan bufron kaj legas
la tutan dosieron, kaj `listi_dosierujon(vojo, vizito)` donas la erojn al
kromprogramo. La legadoj uzas la saman `virtio-blk`-pelilon; ne ekzistas
sektora kaŝmemoro, ĉiu aliro legas la aparaton. La dosiersistemo estas nurlega
kaj ne havas apartan VFS-tavolon; `exec` uzos ĝin en Fazo 3.

**Memprovo.** Post la muntado la kerno kalkulas FNV-1a-haketon super la nomoj,
la grandoj kaj la enhavoj de ĉiuj dosieroj, presas la nombrojn kaj la haketon,
legas `bonveno.txt` kaj komparas ĝin kun la atendata teksto, kaj listigas
`notoj`. La muntado okazas tuj post la `virtio-blk`-memprovo; se ĝi
malsukcesas, la kerno presas `[dosiersistemo] SNDS (MALBONE): …` kaj daŭrigas
la ceterajn scenarojn.

**Lokaj rezultoj (2026-11-18, versio 0.7.1).**

| Vojo | Linio |
| --- | --- |
| x86_64 PVH | `[konservado] virtio-blk: 64 sektoroj legitaj, haketo 0x840ee03e1d84f800`; `[dosiersistemo] SNDS: 4 dosieroj, 1 dosierujo. Haketo: 0x3ba743bb3a5561c3` |
| x86_64 BIOS-ISO | sama duopo; `[dosiersistemo] legado (bone): bonveno.txt, 18 bajtoj` |
| arm64 Image | sama duopo; `[dosiersistemo] listado de notoj: dua.txt, unua.txt` |
| arm64 UEFI-ISO | sama duopo; la kvar vojoj estas verdaj |

**Nuba DI (2026-11-18).** Runo
[#44246263834](https://github.com/makoto-lernanto/mikerno/actions/runs/44246263834)
ĉe `cbfca8b`: ĉiuj 43 taskoj verdaj — `kontroli-kaj-konstrui` (23 s, inkluzive
`just konstrui-arm` por la plata arm64-`Image`), `kani-pretigi` (49 s) kaj 41
pruvoj de la matrico `kani-kontroli`; la tuta runo daŭris 11 min 9 s. La solaj
anoncoj estis ĝeneralaj avertoj pri Node.js 20, kiuj ne rilatas la kodon.
La SNDS-memprovo ne havas propran Kani-pruvon; ĝi ruliĝas en la startoj, dum
la formala pordego restas la ekzistanta matrico.

**Eldono.** [`v0.7.1`](https://github.com/makoto-lernanto/mikerno/releases/tag/v0.7.1)
(etikedo ĉe `cbfca8b`) enhavas `mikerno.iso` (SHA-256
`8ce4d37bdbb8ed9c1a7289b5e3c61aa992e58ddfc4f99783162a50a5b0e915ad`) kaj
`mikerno-arm.iso` (SHA-256
`32f87672070866eb684e5ffe0c7987fec62c4d88f7095ddfa60d4e835ccf6bbd`). La du
ISO-oj estis rekonstruitaj el la puŝita kodo `7dc09b0` (esperantigo de la
terminaro) post la verda DI-runo
[#44319290324](https://github.com/makoto-lernanto/mikerno/actions/runs/44319290324),
kaj praŝarĝitaj; la dekstraj elŝutaj ligiloj de la retejo montras al ĉi tiu
etikedo.

**Limigoj.** Nurlega; unu ebeno da dosierujoj sen aparta VFS; dosiero estas
kontigua ene de sia inodo; neniu kaŝmemoro; la bildo estas fiksita dum
konstruo (ĝisdatigo postulas novan bildon); maksimuma dosiero 64 KiB (C17.4).

**Versio.** 0.7.1 (Fazo 2).

#### 1c. Fazo 3 — efektivigita (0.7.2)

**Adresaro.** La uzanta bildo restas ĉe `0x1_0000_0000`, la stako okupas la
tutan `0x1000_0000`–`0x1010_0000` (16 MiB), kaj la kupo komenciĝas tuj poste:
`UZANTA_KUPO_BAZO = 0x1_0200_0000`, kun limo `UZANTA_KUPO_FINO =
0x2_0000_0000`. La sekretaj paĝoj ĉe `0x2_0000_0000` do restas ekster la kupo.

**`SYS_BRK` (13).** `Procezo` ricevas la kampon `kupo_fino` (komenca valoro
`UZANTA_KUPO_BAZO`) kaj la metodojn `kupaj_limoj()` kaj `etendi_kupon(nova,
alokilo)`. La semantiko sekvas `brk`: `nova == 0` redonas la nunan limon; peto
en `[UZANTA_KUPO_BAZO, UZANTA_KUPO_FINO]` mapas la mankantajn paĝojn kiel
nuligitajn `UZANTA_DATUMOJ`-foliojn kaj redonas la novan limon; peto ekster la
intervalo aŭ mapada eraro redonas la malnovan limon. La limo neniam
malpliiĝas: `nova` malpli alta ol la nuna valoro nur redonas la nunan limon,
do `brk` estas monotona kaj nenia malmapo okazas. La voko validas nur el Ring 3
kaj ruliĝas sub `InterrompaGardo`.

**Kupa alokilo.** `horizonto/src/biblioteko.rs` uzas simplan `bump`-alokilon:
`KUPO_NUNA` kaj `KUPO_FINO` estas atomaj `usize`-oj; la unua `aloki` vokas
`brk(0)` por lerni la bazon, poste `brk`-as la aliniitan finon (`KUPO_ALINIO =
16`) antaŭ ol redoni la montrilon. `liberigi` estas malplena — la kupo kreskas
nur. La buklo en `horizonto` alokas 4096 bajtojn, skribas kaj relegas ŝablonon
(mod 251) kaj presas la akceptan linion.

**`SYS_EXEC` (14).** `kerno/exec.rs` legas la vojon (1–255 bajtoj) el la uzanta
spaco sub `StacGardo`, postulas validan UTF-8-on sen NUL, malfermas la dosieron
per SNDS, postulas almenaŭ 64-bajtan ELF-kapon, alokas kontiguajn kadrojn per
`aloki_kontiguajn`, legas la dosieron rekte en ilin kaj muntas ĝin per
`munti_el_bufro` (la sama `ŝargi_bildon` kiel la praŝarĝa vojo) en novan
`Spaco`-n kun freŝa stako. `Procezo::anstataŭigi_spacon` ŝaltas la kernan
radikon se la malnova spaco estas aktiva, anstataŭigas la spacon, restarigas
`kupo_fino` kaj detruas la malnovan spacon; poste `procezo::aktivigi(id)` kaj
`eniri_uzanton(..., REĜIMO_EXEC, id)`. La eniro neniam revenas; malsukceso
presas `[kerno] exec: procezo N malsukcesis: …` kaj la voko redonas
`MALĜUSTA_ARG`. `libmikerno::starto::REĜIMO_EXEC` (4) igas la novan bildon
raporti `[ring3] exec: nova bildo funkcias (procezo N)` anstataŭe ripeti la
testojn (C17.5: `exec` anstataŭigas la nunan procezon; `fork` restas ŝuldo).

**arm64-aliniiĝo.** La arm64-ELF estis 90 520 bajtoj pro la defaŭlta 64 KiB
`max-page-size` de `rust-lld`; `horizonto/build.rs` nun pasigas `-z
max-page-size=4096`, do la segmentoj ne plu havas 64 KiB-truojn kaj la bildo
(31 056 bajtoj) taŭgas por la 64 KiB-limigo de SNDS (C17.4).

**SNDS-aldonaĵoj.** `tools/krei_snds_img.sh` akceptas pliajn argumentojn
`nomo=vojo`, kiujn ĝi metas en la radikon de bildo kopiita al provizora
dosierujo. `tools/run-qemu.sh` kaj `tools/run-qemu-arm.sh` aldonas
`horizonto.elf=<konstruita horizonto>`; tiel la sama bildo servas kiel celo de
`SYS_EXEC` en ambaŭ arkitekturoj.

**Lokaj rezultoj (2026-11-19, versio 0.7.2).**

| Vojo | Linio |
| --- | --- |
| x86_64 PVH | `[ring3] kupo: alokita 4096 bajtoj ĉe 0x102000000`; `[kerno] exec: procezo 5 anstataŭigita per horizonto.elf (39648 bajtoj, enirpunkto 0x100002140)`; `[ring3] exec: nova bildo funkcias (procezo 5)`; `[memoro] izolado (bone): 64771 kadroj antaŭ, 64771 post` |
| arm64 Image | `[ring3] kupo: alokita 4096 bajtoj ĉe 0x102000000`; `[kerno] exec: procezo 5 anstataŭigita per horizonto.elf (31056 bajtoj, enirpunkto 0x100000728)`; `[ring3] exec: nova bildo funkcias (procezo 5)`; `[memoro] izolado (bone): 63979 kadroj antaŭ, 63979 post` |

Ambaŭ vojoj finiĝas per `[kerno] fina stato: 6 mortintaj, 0 pretaj, 0
blokitaj` kaj nul `MALBONE`-linioj; la pluraj `[eraro] procezo …`-linioj estas
la intencaj negativaj provoj (montrilo, sekreta paĝo, kapablo, `SMAP`).

**Nuba DI (2026-11-19).** Runo
[#44469170723](https://github.com/makoto-lernanto/mikerno/actions/runs/44469170723)
ĉe `b6b7549`: ĉiuj 43 taskoj verdaj — `kontroli-kaj-konstrui`, `kani-pretigi`
kaj la 41 pruvoj de la matrico `kani-kontroli`; la runo daŭris 11 min 15 s.

**Eldono.** [`v0.7.2`](https://github.com/makoto-lernanto/mikerno/releases/tag/v0.7.2)
(etikedo ĉe `b6b7549`) enhavas `mikerno.iso` (SHA-256
`4dd10e84b3b160d2e01344a3fabfab2047da7d1aea3ab263298c64b83edf8ac4`) kaj
`mikerno-arm.iso` (SHA-256
`b739892d259d9d035b22af19609b02b0b7ed299d9ee94e467e098eabdf435ea2`). La du
ISO-oj estis rekonstruitaj el la puŝita kodo post la verda DI-runo kaj
praŝarĝitaj antaŭ la alŝuto; la retejo montras la version 0.7.2 kaj la
elŝutajn ligilojn al ĉi tiu etikedo.

**Limigoj.** `brk` nur pligrandigas kaj neniam malmapas; la alokilo ne
liberigas; `exec` anstataŭigas la procezon (neniu `fork`); la celata dosiero
devas esti en la radiko de SNDS kaj havi maksimume 128 KiB (64 KiB en 0.7.2;
la limo altiĝis en la Fazo 4, vidu C17.7); la bildo de
`horizonto` devas esti pakita en la SNDS-bildon dum la konstruo.

**Versio.** 0.7.2 (Fazo 3).

#### 1d. Fazo 4 — efektivigita (0.7.3)

**Objekta ŝelo Ligo.** `horizonto/src/ligo.rs` anstataŭas la memprovajn
reĝimojn de la bildo per vera ŝelo. `libmikerno::starto::REĜIMO_ŜELO` (6) kaj
`REĜIMO_FONO` (5) distingas la du procezojn, kiujn la kerno kreas post la
izolada scenaro: procezo 1 «ŝelo» eniras `ligo::starti()`, procezo 2 «fono»
restas en `dormi(1000)`-buklo. La ŝelo legas bajtojn per `SYS_ENIGI` (15), ne
eĥas la enigon loke kaj reskribas la linion per `\r\x1b[K` plus la
prompto; la ekrana prezento de la frambufo konsumas la samajn `ANSI`-sekvencojn
kiel la seria konzolo.

**Objektoj, ne teksto.** La dukto transdonas `Vico`-n da `Rekordo`-j
(`Dosierero` el `libmikerno::registro`, 72 bajtoj, kaj `ProcezoRekordo`, 104
bajtoj). La unua stadio produktas rekordojn (`listi`, `legi`, `procezoj`,
`eldiri`, `pri`), kaj la sekvaj stadioj konservas la kampan strukturon:
`filtri` komparas nombrojn laŭ la montrata unuo, `ordigi` ordigas laŭ kampo,
`unua` tranĉas, `sumi` kaj `nombri` produktas skalaron, `montri` projekcias
kolumnojn, `teksto` estas la eksplicita teksta limo. `Speco` difinas la
kampojn de ĉiu familio: dosiero (`nomo`, `tipo`, `grandeco`, `tenilo`),
procezo (`pid`, `nomo`, `stato`, `memoro`, `permesoj`), mesaĝo (`noto`) kaj
kampo (`kampo`, `valoro`). La ŝelo ne konvertas la fluon al teksto antaŭ la
fina stadio.

**Koloroj laŭ FluoU.** La tabelo uzas la ĵetonojn de `docs/FLUOU.md` §2:
kapo en `FG_TEKSTO3`, nomoj en `FG_NOMO`, nombroj dekstre alĝustigitaj,
statoj en `FG_SUKCESO`/`FG_INFORMO`/`FG_DANĜERO`, kaj la tenila valoro en
`FG_ORO` sur `BG_TENILO` — ĉiam kune kun la teksto `TENILO`, do la koloro
neniam estas la sola portanto de informo. `temo` kaj `akcento` elektas la
koloraron; `akcento` akceptas la ses akcentojn de FluoU.

**Komandoj.** `helpu`, `listi`, `legi`, `procezoj`, `mortigi`, `ruli`,
`eldiri`, `pri`, `temo`, `akcento`, `eniro`, `filtri`, `ordigi`, `unua`,
`sumi`, `nombri`, `montri`, `teksto`, `purigi` kaj `fini`. `mortigi`
protektas nur `pid ≤ 1`; la kerno mem rifuzas `0` kaj `1` per `NEPERMITITA`
(-13). `ruli` vokas `SYS_LANĜI` (20), kiu kreas novan procezon el SNDS
(la nova krado `saluto` presas `[saluto] saluton el la procezo N; ĝis
revido`). `fini` (aŭ `Ctrl+D` en malplena linio) finas la ŝelon per
`SYS_FINI`.

**Eniga redaktado.** La ŝelo havas historian ringon (8 eroj, supre/malsupre
per la sagoklavoj), `Tab`-kompletigon de la unua vorto kaj de la komandaro
(komuna prefikso aŭ lista propono), `Ctrl+C` por malplenigi la linion,
`Ctrl+L`/`purigi` por malplenigi la ekranon. La kerna eniga bufro estas la
UART-FIFO; la ŝelo do legas unu bajton post alia kaj mem konservas la linion.

**SNDS-limo kaj bilda grando (C17.7).** La ŝelo kun sia helpo superis la
malnovan 64 KiB-limon. `mikerno/src/konservado/snds.rs` kaj
`tools/krei_snds_img.sh` nun akceptas 128 KiB; la bildo kreskas al 256
sektoroj kaj la virtio-blk-memprovo restas valida. Por teni la bildojn
malgrandaj, `Cargo.toml` donas al `horizonto` kaj `saluto` `opt-level = "z"`,
`codegen-units = 1` kaj tutan `strip`; la rezulto: `horizonto` 97 048 bajtoj
(x86_64) kaj 93 816 bajtoj (arm64), `saluto` 16 024 kaj 16 664 bajtoj.

**Perdita vekigo en `hlt`/`wfi`.** La atenda buklo de la kerno post la ŝelo
ŝajne ne rimarkis la malaperon de procezo 1. La kaŭzo estis `options(nomem)`
ĉe la `hlt`/`wfi` inline-asm: ĝi diris al la kompililo, ke la instrukcio ne
legas memoraĵon, do la kompililo rajtis eltiri la legadon de la proceza
tabelo el la buklo. Sen `nomem` la legado restas en la buklo kaj la
interrompa traktilo videblas al la kompililo. Ĉi tiu latenta cimo koncernis
ankaŭ la antaŭajn atendojn de tikoj.

**Riparo post la unua eldono.** La unue alŝutita 0.7.3-bildo ankoraŭ
enhavis malnovan diagnozan eligon en la temporizila traktilo: ambaŭ arkoj
skribis `[horloĝo] tiko N` dum la unuaj kvin tikoj kaj poste ĉiun 100-an
tikon (≈1 s) al la seria konzolo dum la tuta funkciado. Ĝi intermiksiĝis kun
la `ligo:~>`-prompto kaj eĉ kun la vicoj de `listi`. La eldono estis
retirita: la du ISO-oj kaj la etikedo `v0.7.3` estis forigitaj, `main` estis
restarigita al la antaŭa stato, kaj la eligaj linioj estis forigitaj el
`x86_64/temporizilo.rs` kaj `arm64/temporizilo.rs`; la tika nombrilo kaj la
planista voko restas senŝanĝaj. Novaj lokalaj provoj (x86_64 PVH kaj arm64
Image) montris nulan `[horloĝo]`-eligon dum plursekundaj paŭzoj ĉe la
prompto, kaj la ŝelo plu funkcias. La du riparoj estas la kommitoj
`4abd527` (temporizila eligo) kaj `d66e694` (puraj ASCII-nomoj); la nova
eldono estas kreita el la dua komito.

**Puraj ASCII-komandnomoj.** Laŭ la lingva regulo, ĉiuj Ligo-komandnomoj
kaj argumentoj estas Esperantaj vortoj, kiuj jam estas puraj ASCII: la
komando `lanĉi` fariĝis `ruli`, la mesaĝa kampo `mesaĝo` fariĝis `noto`, kaj
la akcento `ĉielo` fariĝis `lazuro` (helblua). La helpo, la eraroj, la
tabelaj kapoj kaj la normaj dokumentoj sekvas la samajn nomojn; la x-sistemo
restas malpermesita.

**Lokaj rezultoj (2026-11-19, versio 0.7.3).**

| Vojo | Linio |
| --- | --- |
| x86_64 PVH | `[kerno] ŝelo startas kiel procezo 1; por fini: fini`; `listi` -> `bonveno.txt 18 B TENILO 0x0001`, `horizonto.elf 94.7 KiB`, `legu.txt 158 B`, `notoj dosierujo 128 B`, `saluto.elf 15.6 KiB`; `listi \| sumi grandeco` -> `SUMO DE GRANDECO 110.7 KiB`; `procezoj` -> `1 ŝelo Kuranta`, `2 fono Preta`; `mortigi 2` -> `[procezo] 2 detruita: 1 privataj PDPT-oj, PML4 0x261000, 45 kadroj liberigitaj` kaj `Procezo 2 (fono) finiĝis.`; `ruli saluto.elf` -> `[kerno] nova procezo 3 lanĉita el SNDS: saluto.elf (enirpunkto 0x100000c31)`, `Procezo 3 lanĉita: saluto.elf` kaj `[saluto] saluton el la procezo 3; ĝis revido`; `fini` -> `[kerno] ŝelo finiĝis; la kerno daŭras`; `[kerno] fina stato: 7 mortintaj, 0 pretaj, 0 blokitaj; 2710 tikoj` |
| arm64 Image | la sama interagado per PL011; `listi` montras `18 B`, `91.7 KiB` kaj `128 B`, `sumi grandeco` -> `SUMO DE GRANDECO 108.2 KiB`; `mortigi 2` -> `[procezo] 2 detruita: 9 privataj subtabeloj, TTBR0 0x4053c000, 43 kadroj liberigitaj` kaj `Procezo 2 (fono) finiĝis.`; `ruli saluto.elf` -> `[kerno] nova procezo 3 lanĉita el SNDS: saluto.elf (enirpunkto 0x100000028)`; `[kerno] fina stato: 7 mortintaj, 0 pretaj, 0 blokitaj; 2840 tikoj` |

Ambaŭ vojoj havas nul `MALBONE`-liniojn; la ŝelo kaj la fono malaperas antaŭ
la fina stato, do la kadra bilanco restas pura. La nesukcesaj komandoj
ellasas la Esperantajn erarojn kun la kodo kaj ne haltigas la ŝelon: `ls` ->
`Nekonata komando: "ls" (-2) — tajpu helpu`, `legi neniu.txt` ->
`✕ legi: dosiero ne ekzistas: neniu.txt (-3)`, `mortigi 1` ->
`procezo 1 estas protekta sistema procezo (-1)`.

**Nuba DI (2026-11-19).** Runo
[#44559431844](https://github.com/makoto-lernanto/mikerno/actions/runs/44559431844)
ĉe `d66e694`: ĉiuj 43 taskoj verdaj — `kontroli-kaj-konstrui`, `kani-pretigi`
kaj la 41 pruvoj de la matrico `kani-kontroli`; la runo daŭris 11 min 10 s.

**Eldono.** [`v0.7.3`](https://github.com/makoto-lernanto/mikerno/releases/tag/v0.7.3)
(etikedo ĉe `d66e694`) enhavas `mikerno.iso` (SHA-256
`f1bde04e71a260b87edb998f2901e05800d95b37a6cc93b0bd1b8bba1725daa8`) kaj
`mikerno-arm.iso` (SHA-256
`e3af18f500e2db7684ffe993c4de85d190b0773b7681bd68129ec0358140a53b`). La du
ISO-oj estis konstruitaj el la puŝita kodo post la verda DI-runo kaj
praŝarĝitaj antaŭ la alŝuto; la retejo montras la version 0.7.3 kaj la
elŝutajn ligilojn al ĉi tiu etikedo.

**Limigoj.** SNDS restas nurlega; la ŝelo legas COM1 per polusa `SYS_ENIGI`
(10-tika dormo inter provoj); mankas `fork`, do `ruli` kreas novan procezon
kaj `exec` anstataŭigas la nunan; la prompto estas `ligo:~>` kaj la
komandnomoj estas nur Esperantaj.

**Versio.** 0.7.3 (Fazo 4).

#### 1e. Fazo 5 — efektivigita (0.7.4)

**Purigado de `#[allow]`-oj.** M17 Fazo 1 aldonis du provizorajn
`#[cfg_attr(target_arch = "x86_64", allow(dead_code))]`-ojn en
`arkitekturo/komuna.rs`: `VirtioSpeco::Mmio` kaj `VirtioTransporto::nova_mmio`
ekzistas nur por `arm64`. Anstataŭ la permesiloj la MMIO-eroj nun estas baritaj
per `#[cfg(target_arch = "aarch64")]` — la varianto, la konstruilo, la
`mmio_reg`-modulo kaj la koncernaj `match`-branĉoj en `komuna.rs` kaj
`virtio_blk.rs`. Ankaŭ la `#[allow(clippy::mut_from_ref)]` de
`Kunhavigita::akiri` forfalis: `akiri` nun redonas krudan montrilon `*mut T`
(kiel `UnsafeCell::get`), kaj ĉiu alirejo derivas la referencon per eksplicita
`unsafe { &mut *… }`. Restas nur la antaŭkonstitucia modul-nivela
`cfg_attr(target_arch = "aarch64", allow(dead_code))` de `komuna.rs` (M15),
registrita en §9.3.

**Arm64 MMIO-vojo.** `QEMU_DISPLAY=none bash tools/run-qemu-arm.sh release`
elektas la MMIO-transporton de `virt`:

```
[konservado] virtio-blk (MMIO): 256 sektoroj, 512 bajtoj po sektoro
[konservado] virtio-blk: 64 sektoroj legitaj, haketo 0x0958f2eeba293047
[konservado] virtio-blk: skribo kaj restarigo (bone)
[dosiersistemo] SNDS: 6 dosieroj, 1 dosierujo. Haketo: 0x5aa97779fbdd1bcb
```

**Pordego kaj DI.** `just kontroli` nun lintas ankaŭ `horizonto` (la
Ligo-komandojn) kaj `saluto` por `aarch64-unknown-none`; DI plenumas la saman
`just kontroli` kaj poste `just konstrui-arm` por la liniigita `arm64-Image`.
La loka pordego finiĝis sen eraro per:

- `bash tools/versio.sh --kontroli` → `versio: ĉiuj dokumentaj lokoj kongruas kun 0.7.4`;
- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --release -- -D warnings`;
- `cargo clippy -p mikerno --release --target aarch64-unknown-none -- -D warnings`;
- `cargo clippy -p horizonto -p saluto --release --target aarch64-unknown-none -- -D warnings`;
- `cargo build --workspace --release`.

**Lokaj rezultoj (2026-11-20, versio 0.7.4).**

| Vojo | Linio |
| --- | --- |
| x86_64 PVH | `[konservado] virtio-blk (PCI): 256 sektoroj, 512 bajtoj po sektoro`; `[dosiersistemo] SNDS: 6 dosieroj, 1 dosierujo. Haketo: 0xcd95864fee35a274`; `listi` -> `bonveno.txt 18 B TENILO 0x0001`, `horizonto.elf 94.7 KiB`, `saluto.elf 15.6 KiB`; `procezoj` -> `1 ŝelo Kuranta`, `2 fono Preta`; `ruli saluto.elf` -> `[kerno] nova procezo 3 lanĉita el SNDS: saluto.elf (enirpunkto 0x100000c31)`, `Procezo 3 lanĉita: saluto.elf`, `[saluto] saluton el la procezo 3; ĝis revido`; `fini` -> `[kerno] fina stato: 7 mortintaj, 0 pretaj, 0 blokitaj; 1447 tikoj` |
| arm64 Image | `[konservado] virtio-blk (MMIO): 256 sektoroj, 512 bajtoj po sektoro`; `[dosiersistemo] SNDS: 6 dosieroj, 1 dosierujo. Haketo: 0x5aa97779fbdd1bcb`; `listi` -> `horizonto.elf 91.6 KiB`, `saluto.elf 16.2 KiB`; `procezoj` -> `1 ŝelo Kuranta`, `2 fono Preta`; `ruli saluto.elf` -> `[kerno] nova procezo 3 lanĉita el SNDS: saluto.elf (enirpunkto 0x100000028)`; `fini` -> `[kerno] fina stato: 7 mortintaj, 0 pretaj, 0 blokitaj; 2128 tikoj` |

Ambaŭ vojoj havas nul `[horloĝo]`-liniojn dum la interagado, kaj la ŝelo kaj
la lanĉitaj procezoj malaperas antaŭ la fina kadra bilanco.

**Versio.** 0.7.4 (Fazo 5).

#### 2. Evitindaj Eraroj (Konsiloj por la Arkitekto)

1.  **Virtio kaj Memoro-Bariloj**: Virtio-vicoj postulas striktan ordigon de skriboj. En Rust, uzu `core::sync::atomic::compiler_fence(Ordering::SeqCst)` antaŭ ol sciigi la aparaton pri novaj priskribiloj. Malsukceso kaŭzos silentan datuman korupton.
2.  **SMAP/SMEP en Ring 3**: Kiam la ŝelo (Ring 3) petas al la kerno legi dosieron, la kerno ne povas rekte skribi al la uzanta bufrujo sen `STAC`-fenestro. Uzu provizoran kernan bufrujon, kopiu la datumojn, kaj transdonu ilin per `SYS_RICEVI` (IK) aŭ kopiu ilin al la uzanta spaco per sekura `kopii_al_uzanto` funkcio.
3.  **SNDS Bilda Generado**: La `tools/krei_snds_img.sh` skripto devas esti sendependa de platformo (uzu `python3` anstataŭ `bash`-specifaj iloj se eble), por ke DI en GitHub Actions (Ubuntu) kaj loka WSL2 produktu identajn haketojn.
4.  **Mortigo de la Ŝelo**: Se la ŝelo mem mortas (`SYS_FINI`), la sistemo ne havas uzant-interfacon. La kerno devas havi specialan kazon: se procezo 1 (la ŝelo) mortas, la kerno aŭtomate lanĉas ĝin denove el SNDS.

#### 3. Akceptkriterioj

| Kriterio | Stato | Pruvo (Elĉerpaĵo de QEMU) |
| :--- | :--- | :--- |
| `virtio-blk` legas blokojn | Plenumita (Fazo 1) | `[konservado] virtio-blk: 64 sektoroj legitaj, haketo 0xc139f0538c943325`; skribo kaj restarigo verdaj (PCI kaj MMIO) |
| SNDS estas muntita kaj legebla | Plenumita (Fazo 2) | `[dosiersistemo] SNDS: 4 dosieroj, 1 dosierujo. Haketo: 0x3ba743bb3a5561c3`; `[dosiersistemo] legado (bone): bonveno.txt, 18 bajtoj` |
| `SYS_BRK` funkcias en Ring 3 | Plenumita (Fazo 3) | `[ring3] kupo: alokita 4096 bajtoj ĉe 0x102000000` (x86_64 kaj arm64) |
| `SYS_EXEC` anstataŭigas la procezon | Plenumita (Fazo 3) | `[kerno] exec: procezo 5 anstataŭigita per horizonto.elf (39648 bajtoj, enirpunkto 0x100002140)`; `[ring3] exec: nova bildo funkcias (procezo 5)` |
| Ŝelo startas kaj akceptas enigon | Plenumita (Fazo 4) | `[kerno] ŝelo startas kiel procezo 1; por fini: fini`; prompto `ligo:~> ` kun linia redaktado (historio, `Tab`, `Ctrl+C`, `Ctrl+L`) |
| `listi` kaj `legi` funkcias | Plenumita (Fazo 4) | `ligo:~> listi` -> `bonveno.txt 18 B TENILO 0x0001`, `horizonto.elf 94.7 KiB`, `notoj dosierujo 128 B` <br> `ligo:~> legi bonveno.txt` -> `│ Saluton, MiKerno!` kun `TENILO 0x0001` |
| `procezoj` kaj `mortigi` funkcias | Plenumita (Fazo 4) | `ligo:~> procezoj` -> `1 ŝelo Kuranta`, `2 fono Preta` <br> `ligo:~> mortigi 2` -> `[procezo] 2 detruita: 1 privataj PDPT-oj, PML4 0x261000, 45 kadroj liberigitaj`, `Procezo 2 (fono) finiĝis.`; `mortigi 1` rifuzata per `-1` |
| `ruli` startas novan ELF | Plenumita (Fazo 4) | `ligo:~> ruli saluto.elf` -> `[kerno] nova procezo 3 lanĉita el SNDS: saluto.elf (enirpunkto 0x100000c31)`; `[saluto] saluton el la procezo 3; ĝis revido` |
| `arm64` vojo restas verda | Plenumita (Fazo 4) | `QEMU_DISPLAY=none bash tools/run-qemu-arm.sh release` montras la saman interagadon per PL011; `[kerno] fina stato: 7 mortintaj, 0 pretaj, 0 blokitaj; 2840 tikoj` |
| `virtio-blk` MMIO funkcias sur `arm64` | Plenumita (Fazo 5) | `[konservado] virtio-blk (MMIO): 256 sektoroj, 512 bajtoj po sektoro`; `[konservado] virtio-blk: skribo kaj restarigo (bone)`; `[dosiersistemo] SNDS: 6 dosieroj, 1 dosierujo` |
| M17-aj `#[allow]`-oj forigitaj | Plenumita (Fazo 5) | `cargo clippy` sen `-A`/`#[allow]` por `mut_from_ref` nek por la MMIO-eroj de `komuna.rs` sur ambaŭ arkoj |

#### 4. Teknika Ŝuldo Post M17

| Prioritate | Ŝuldo | Rimedo |
| :--- | :--- | :--- |
| Alta | Neniu `fork`; nur `exec` | Post 1.0 (postulas kopiadon de adresaj spacoj) |
| Meza | SNDS estas nurlega | M17-poste: aldoni `skribi` kaj `krei` |
| Meza | Neniu IOMMU por `virtio` DMA | M19 (Aparata izolado) |
| Malalta | Ŝelo uzas polusan legadon por COM1 | M18: uzi interromp-bazitan I/O |

---

### M18 — Inĝeniera matureco

**Celo.** Ripetebla, pruvita kaj dokumentita projekto.

**Taskoj.** DI (Linux), Kani en DI, testoj por arkitekture sendependaj
moduloj, QEMU-matrico (BIOS/UEFI, diversaj memorkvantoj), `cargo-deny` /
permesila kontrolo, regenerado de la ISO en DI.

**Akceptkriterioj.** Ĉiu tirpeto plenumas `fmt`, `clippy`, `konstruo`, `kani` kaj la
QEMU-smokan teston; la ISO estas artefakto de DI, ne permana dosiero.

---

### M19 — Aparata izolado (post 1.0)

**Celo.** Malhelpi, ke aparato atingu arbitran fizikan memoron per DMA.

**Taskoj.** ACPI `DMAR`/`IVRS`-analizo, PCI-enumerado, radikaj kaj kuntekstaj
tabeloj por la IOMMU, DMA-mapoj por ĉiu aparato, interrompa readresado;
QEMU-matrico kun `intel-iommu`.

**Akceptkriterioj.** Aparato povas atingi nur siajn proprajn DMA-mapojn; nevalida
mapo produktas raportitan `FAULT` anstataŭ silentan memoran korupton. Ĉi tiu
mejloŝtono venas post 1.0, ĉar ĝi dependas de la PCI/aparata tavolo el M17.

---

## 6. Inĝenieraj decidoj

| Decido | Kialo | Kosto |
| --- | --- | --- |
| Neniu ekstera Rust-dependaĵo | Aŭdebleco, riparebleco, plena kontrolo | Pli da propra kodo |
| Du praŝarĝaj protokoloj | PVH por rapida `cargo run`; MB2 por reala ISO | Du analiziloj kaj du magioj |
| Esperanto en kodo kaj dokumentaro | Konsekvenco kaj distingiĝo | Malpli da kontribuantoj |
| Identeca mapo | Simpla praŝarĝo kaj aliro al MMIO | Ne plene izolita; W^X nur laŭ sekcioj |
| Statikaj kernaj strukturoj | Sen kerna kupo; antaŭvidebla memoro | Fiksaj limoj (8 fadenoj, ...) |
| Interromp-malŝaltaj kritikaj sekcioj | Unuprocesora; simpla kaj sendependa de ŝlosiloj | Ne skaliĝas al SMP |
| `EOI` antaŭ la traktilo | La traktilo rajtas ŝanĝi kuntekston; la sekva tiko devas alveni | Pli da nestitaj interrompoj |
| PIT-kalibrado de la APIC | Busfrekvenco ne estas arkitekture legebla | 50 ms da starta prokrasto |
| 4 GiB fizika limo | Bitara indekso kaj identeca mapo | Ne subtenas pli da ĉefmemoro |
| Kopiataj mesaĝoj | Sendanto povas tuj reuzi sian bufron | 64-bajta limo |
| Sama virtuala adreso por la sekreta paĝo | La pruvo montras fizikan izoladon, ne nur malsamajn adresojn | Nur unu paĝo por procezo; la dua paĝo restas viktima provo |
| Unu `planisto::lanĉi()` por ambaŭ procezoj | La du procezoj vere interplektiĝas; neniu dua starta vojo | La praŝarĝo devas decidi la tutan aron antaŭe |
| Kani-speguloj | Traktebla stato-spaco | La produkta kodo ne estas pruvita |

---

## 7. Teknika ŝuldo (prioritatigita)

| Prioritate | Ŝuldo | Efiko | Rimedo |
| --- | --- | --- | --- |
| Alta | Neniu `fork`/`exec`; procezoj estas preparitaj de la praŝarĝo | Ne eblas dinamika lanĉo el Ring 3 | M17 (`fork`/`exec`) |
| Alta | Ring 3-esceptoj krom `#PF`/`#GP` haltigas la sistemon | Ne rekaptebla eraro (ekz. `#UD`) | M12/M18 |
| Alta | Neniu DI nek testoj | Regresoj nerimarkeblaj | M18 |
| Alta | Malaktuala ISO | Konfuzaj testrezultoj | DI + versio |
| Malalta | Dokumentoj kaj `Cargo.toml` povas malkongrui | Konfuzaj eldonoj | `tools/versio.sh --kontroli` en `just kontroli` |
| Meza | IK-finfunkto = fadeno | Ne eblas sendependaj objektoj | M14 |
| Meza | Sekreta paĝo estas unu fiksa mapado por procezo | Neniu ĝenerala `mmap`/`brk` por uzantaj datumoj | M17 |
| Meza | Neniu IOMMU | Aparata DMA ne estas izolita | M19 |
| Meza | Frambufo sen rulumo | Malbona uzsperto | M6-poste |
| Malalta | `Kunhavigita::akiri` liveras krudan montrilon el `&self` | La derivo de referenco devas esti eksplicita `unsafe` | reviziita (M17 F5); la disciplino restas dokumentita |
| Malalta | Kani-modelo ≠ produkta kodo | Pruvoj ne rekte aplikeblas | M9 |
| Malalta | Antaŭkonstituciaj `#[allow]`-oj sen `PERMESO-JUSTIFIKO` | Povas kaŝi neuzatan aŭ neŝatatan kodon | revizii dum M18 (§9.3) |

---

## 8. Difino de «1.0»

MiKerno atingos 1.0 kiam:

1. `horizonto` (aŭ alia uzanta procezo) ruliĝas en Ring 3 kun propra adresa
   spaco, sen povoj legi la kernon aŭ alian procezon.
2. Sistemvokoj `SKRIBI`, `FINI`, `CEDI`, `SENDI`, `RICEVI` funkcias kaj estas
   dokumentitaj en `libmikerno`.
3. IK havas apartajn finpunktajn objektojn, rajtojn kaj tempolimojn.
4. Kadroj kaj paĝtabeloj estas liberigeblaj; longa testado ne elĉerpas memoron.
5. La ISO estas konstruata de DI, kaj `fmt`, `clippy`, `konstruo`, `kani` kaj
   QEMU-smoka testo estas devigaj.
6. La dokumentaro (README, ĉi tiu dokumento, sistemvoka referenco) estas
   kompleta kaj kongrua kun la kodo.

---

## 9. Kvalita Konstitucio (neintertraktebla)

> **Valideco.** Ekde la 2026-11-15 ĉiu Fazo devas konformi al ĉi tiu
> konstitucio.
>
> **Ununura difino de «farita».** `just kontroli` (loke, sen Kani) **kaj** la
> DI-matrico `kani-kontroli` finiĝas sen eraro **kaj** la akcepto de la nuna
> Fazo montras la atenditajn liniojn (`arm64`: `just ruli-arm` kaj
> `just ruli-iso-arm`; `x86_64`: la PVH- kaj ISO-startoj). **Ĉiu ruĝa
> kontrolo signifas, ke la tasko ne estas farita**; oni ne rajtas diri, ke ĝi
> «jam efektiviĝis».

### 9.1. Tri neintertrakteblaj reguloj

1. **Malpermesite ŝanĝi la kontrolon por ke ĝi pasu.** Oni ne aldonu
   `#[allow(...)]`, ne malaltigu nivelojn de la statika analizo (`-D warnings` restas), ne
   forigu aŭ komentu malsukcesajn testojn aŭ pruvojn, kaj ne uzu
   `kani::assume` por mallarĝigi la enigan spacon kaj kaŝi kontraŭekzemplon.
2. **Se `#[allow]` estas vere necesa**, skribu la kialon kiel komenton en
   Esperanto kaj listigu ĝin aparte en la respondo, atendante la aprobon de la
   uzanto:
   ```rust
   // PERMESO-JUSTIFIKO: la eroj de la fw_cfg-dosierujo havas fiksan longon
   // de 64 bajtoj, do la indeksoj estas garantiitaj de la protokolo.
   #[allow(clippy::indexing_slicing)]
   ```
3. **Ĉiu liveraĵo inkluzivu veran komandan eligon** — la finan eligon de
   `just kontroli` kaj la konkludon de la DI-matrico `kani-kontroli` (kun la
   protokoloj de la malsukcesintaj pruvoj). Sen eligo = sen plenumo.

### 9.2. La pordego de MiKerno

La originala `just di` ne ekzistas en MiKerno; la ekvivalento estas
`just kontroli`, kiu vokas:

- `bash tools/versio.sh --kontroli`;
- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --release -- -D warnings`;
- `cargo clippy -p mikerno --release --target aarch64-unknown-none -- -D warnings`;
- `cargo clippy -p horizonto -p saluto --release --target aarch64-unknown-none -- -D warnings`;
- `cargo build --workspace --release`.

Kani ne estas parto de `just kontroli`: ĝi ruliĝas nur en DI, po unu tasko por
ĉiu pruvo (§4.7–§4.8).

Aldone, la liveraĵo de Fazo inkluzivu la akceptajn startojn difinitajn en
[ARM.md](ARM.md) §11 aŭ en la koncerna mejloŝtono: `just ruli-arm`,
`just ruli-iso-arm`, `tools/run-qemu.sh release` (PVH) kaj
`tools/run-qemu.sh iso`. Do anstataŭ unu `just di`, MiKerno havas la lokan
pordegon `just kontroli`, la DI-matricon `kani-kontroli` kaj la platformajn
akceptojn. Ekde 2026-11-16 `just kontroli` ne plu enhavas Kani-on; `cargo kani`
ruliĝas nur en GitHub Actions, po unu pruvo (§4.7–§4.8).

### 9.3. Jam ekzistantaj `#[allow]`-oj

La esceptoj starigitaj antaŭ ĉi tiu konstitucio — la `cfg_attr`-aj
`allow(dead_code)`/`allow(unused_imports)` por `aarch64` (M15) kaj la
`allow(dead_code)` ĉe strukturaj bufroj — restas registritaj kiel teknika
ŝuldo (§7). La esceptoj de M17 — la du
`cfg_attr(target_arch = "x86_64", allow(dead_code))` de `komuna.rs` kaj la
`allow(clippy::mut_from_ref)` de `Kunhavigita` — estis forigitaj en Fazo 5
(0.7.4): la MMIO-eroj nun estas baritaj per `#[cfg(target_arch = "aarch64")]`,
kaj `akiri` redonas krudan montrilon. Oni ne aldonu novajn sen la aprobo de la
uzanto laŭ §9.1; kiam Fazo tuŝas ekzistantan escepton, ĝi ricevu
`PERMESO-JUSTIFIKO`-komenton.

---

*Ĉi tiu dokumento estas vivanta; ĝisdatigu ĝin kune kun ĉiu mejloŝtono.*
