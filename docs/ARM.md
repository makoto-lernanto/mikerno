# MiKerno-ARM — plano por la `arm64`-ŝpuro

> Stato: 2026-11-15 · A0–A4 plenumitaj kaj kunfanditaj en M15 (`0.5.0`)
>
> Rilataj: [FLUOU.md](FLUOU.md) §9 (A06, A07), [MEJLOSTONOJ.md](MEJLOSTONOJ.md),
> [VORTARO.md](VORTARO.md).

---

## 1. Celo

MiKerno akiru duan platforman malantaŭon, `arm64` (AArch64), kiu plenumas la
saman kernan nukleon kaj la saman ABI 2-semantikon kiel la `x86_64`-malantaŭo:
`Tenilo`, `Mesago`, la voknumeroj 1–11 kaj la erar-kodoj restas senŝanĝaj; nur
la enirmekanismo kaj la registra aranĝo estas platformaj (`int 0x80` kontraŭ
`svc #0`). Tio kongruas kun FLUOU A06/A07: sama kompona arbo, sama ABI, piksela
paritateco.

Ne-celoj por la unuaj fazoj:

- SMP, IOMMU, virtio kaj reto;
- KASLR sur `arm64` (la bildo restas ĉe fiksa bazo);
- `arm64`-BIOS-ISO aŭ `multiboot2`-pakaĵo (la ISO estas UEFI-bootebla);
- Kani-pruvoj por la `arm64`-malantaŭo (Kani subtenas nur `x86_64`).

## 2. Kial aparta ŝpuro

La nuna nukleo ne estas arkitekture sendependa, malgraŭ la fasado
`arkitekturo`: ĝi portas `x86_64`-konceptojn en komunaj tipoj kaj en la
praŝarĝa vojo. Tial la ŝpuro iris paralele al M16–M18:

- disvolviĝo en branĉo `arm64`;
- fazoj A0–A4 kun propraj akceptkriterioj;
- versioj `0.4.3-arm.N` (antaŭeldono) dum la ŝpuro; ĝi ne antaŭenigas la
  ĉefan `0.x.y`-linion;
- kunfando nur post kiam A4 estas verda; la numero estis elektita laŭ la
  tiama stato: **M15 → `0.5.0`** (2026-11-15). La antaŭe planitaj M15–M18
  translokiĝis al M16–M19.

`tools/versio.sh` akceptas ankaŭ la antaŭeldonon `0.x.y-arm.N` (farita en A1),
kaj `AGENTS.md` priskribas tiun versiadon.

## 3. Nunaj kouplingoj al `x86_64`

| Dosiero | Nuna stato | Bezono |
| --- | --- | --- |
| `mikerno/src/arkitekturo/mod.rs` | fasado ekzistas; A0 re-eksportas `Kunteksto`, `InterrompaKadro`, `ELF_MAŜINO` | ĝeneraligi la restajn nomojn; la semantiko restu en la malantaŭo |
| `mikerno/src/arkitekturo/x86_64/eniro.rs` | 32-bita PVH/Multiboot2-ŝelko, KASLR | restas; `arm64` ricevas propran `eniro.rs` |
| `mikerno/src/kerno/objekto.rs` | `Fadeno` uzas `arkitekturo::Kunteksto` (A0) | — |
| `mikerno/src/interrompo/mod.rs` | komuna registrejo/gardo; la kadro kaj la escepta klasifiko loĝas en `x86_64/interrompo.rs` (A0) | restas |
| `mikerno/src/kerno/sistemvoko.rs` | nur la kerna efektivigo; `arkitekturo::instali_sistemvokon()` (A0) | restas |
| `mikerno/src/memoro/virtuala.rs` | `AdresaSpaco`/`PaĝaTablo` preskaŭ neŭtralaj; trajta metodo `privataj_subtabeloj` (A0) | `CR3`-komentoj restas |
| `mikerno/src/kerno/uzantspaco.rs` | `arkitekturo::ELF_MAŜINO` (A0) | — |
| `mikerno/src/kerno/mod.rs` | `arkitekturo::firmiga_raporto()` (A0) | — |
| `libmikerno/src/syscall.rs` | `invoki` jam havas `cfg`-branĉojn; la ne-`x86_64` branĉo redonas `-1` | aldoni `svc #0`-branĉon por `aarch64` |
| `horizonto/` | `iretq`, `rdi`/`rsi`, ELF ĉe 4 GiB | `eret`, `x0`/`x1`, sama bazo; la ligila skripto reuzeblas |
| `.cargo/config.toml`, `mikerno/build.rs`, `justfile`, `tools/run-qemu.sh` | nur `x86_64` | elekti ligilon kaj lanĉilon laŭ `CARGO_CFG_TARGET_ARCH` |

## 4. Cela dosieruja strukturo

```text
mikerno/src/
├── main.rs                     # enirpunktoj; elekto de la malantaŭo per cfg
├── arkitekturo/
│   ├── mod.rs                  # fasado: Kunteksto, Spaco, kadro, trajtoj
│   ├── komuna.rs               # EsceptoTipo, SistemvokaKadro, FirmigaRaporto
│   ├── x86_64/                 # nunaj dosieroj, sen esencaj ŝanĝoj
│   └── arm64/
│       ├── mod.rs              # pravalorizi: seria → MMU → VBAR → GIC → tempilo
│       ├── eniro.rs            # Image-kapo, EL2→EL1, stako, .bss
│       ├── fdt.rs              # minimuma FDT/DTB-analizilo
│       ├── pagado.rs          # TTBR0_EL1, TCR_EL1, MAIR_EL1, SCTLR_EL1
│       ├── interrompo.rs       # VBAR_EL1, ESR_EL1/FAR_EL1, escepta klasifiko
│       ├── gic.rs              # GICv2: distribuilo + procesora interfaco
│       ├── temporizilo.rs      # ĝenerala temporizilo (CNTP)
│       ├── procesoro.rs        # DAIF, WFI, CurrentEL, PSCI
│       ├── planado.rs          # kuntekstŝanĝo, eret-trampolino
│       ├── sistemvoko.rs       # svc #0-kadro → komuna dispetĉo
│       ├── uzanta.rs           # EL1→EL0 per eret
│       └── seria.rs            # PL011
├── interrompo/mod.rs           # registrejo + InterrompaGardo sen platformaj detaloj
├── kerno/                      # sen x86-specifaj tipoj
├── memoro/                     # AdresaSpaco restas; nur nomoj ĝeneraliĝas
└── ekrano/                     # senŝanĝa; frambufo venas per EniraInformo
```

Novaj dosieroj ekster la kerna kesto:

```text
mikerno/boot/linker-arm64.ld   # Image ĉe 0x4008_0000, text_offset 0x80000
tools/run-qemu-arm.sh          # qemu-system-aarch64 -M virt
```

## 5. Komunaj tipoj kaj trajtoj

- **`arkitekturo::Kunteksto`** — platforma, `#[repr(C)]`. `x86_64` konservas la
  nunajn kampojn; `arm64` konservas `x19`…`x28`, `x29`, `x30`, `sp` kaj
  `daif`. `kerno::objekto::Fadeno` uzas la re-eksportitan tipon.
- **`arkitekturo::InterrompaKadro`** — platforma. Komuna kodo aliras ĝin nur
  per la trajto `SistemvokaKadro`:
  `numero()`, `argumento(i)`, `agordi_rezulton()`, `el_uzanto()`.
- **`arkitekturo::EsceptoTipo`** — `PaĝaEraro`, `ProtektaEraro`, `Nekonata(u64)`.
  La komuna politiko (mortigi la uzantan procezon, raporti, `halti`) restas en
  `interrompo`; la malantaŭo nur klasifikas (`#PF`/`#GP` aŭ data abort/undef).
- **`arkitekturo::FirmigaRaporto`** — nomoj kaj statoj de la firmigaj trajtoj
  (`SMEP`/`SMAP` aŭ `PXN`/`UXN`/`PAN`). `kerno::ĉefa` nur presas ĝin.
- **`arkitekturo::instali_sistemvokon()`** — `x86_64` instalas la DPL3-pordegon
  `0x80`; `arm64` markas la `svc`-vektoron.
- **`arkitekturo::ELF_MAŜINO`** — `62` aŭ `183`, uzata de la ELF-ŝargilo.
- **`PaĝaTablo::privataj_subtabeloj()`** — anstataŭ `privataj_pdptoj()`; la
  signifo estas platforma (PDPT aŭ L1/L2-subtabeloj).

## 6. ABI 2 sur `arm64`

| Elemento | `x86_64` | `arm64` |
| --- | --- | --- |
| Voknumero | `rax` | `x8` |
| Argumentoj 0–3 | `rdi`, `rsi`, `rdx`, `r10` | `x0`–`x3` |
| Rezulto | `rax` | `x0` |
| Enirmekanismo | `int 0x80` | `svc #0` |
| Unua uzanta voko | `rdi` = reĝimo, `rsi` = identigilo | `x0` = reĝimo, `x1` = identigilo |

`Tenilo` (32 bitoj), `Mesago` (96 bajtoj, `#[repr(C)]`) kaj la erar-kodoj
restas bajte identaj; nur `libmikerno::syscall::invoki` kaj la kerna kadro
estas platformaj. FLUOU A07 nun tekstas «la registra aranĝo estas platforma,
la semantiko komuna».

## 7. Memormodelo

- 4 KiB-granulo, 48-bitaj virtualaj adresoj.
- Unu `TTBR0_EL1` por procezo, same kiel unu `CR3`: la kernaj identecaj eroj
  estas kunhavataj, la uzanta fenestro estas privata. `TTBR1_EL1` restas
  malŝaltita (`TCR_EL1.EPD1 = 1`).
- Identeca mapo de `0..4 GiB` per `L1`-blokoj de 1 GiB (ĉefmemoro ĉe `0x4000_0000`,
  MMIO de `virt` sube); la uzanta fenestro `[0x1_0000_0000, 0x80_0000_0000)`
  per `L2`/`L3`.
- `MAIR_EL1`: normala WB por ĉefmemoro, `Device-nGnRnE` por MMIO; `PaĝaPermesoj`
  mapiĝas al `AP[2:1]`, `UXN`, `PXN` kaj al la kaŝebla atributo.
- `KadraAlokilo` restas valida: ĝi alogas kadrojn sub 4 GiB, kiuj estas
  identece mapitaj.
- TLB: `TLBI VMALLE1` + `DSB ISH` + `ISB` post mapŝanĝoj; sen ASID unue.
- Firmigo: `PXN`/`UXN` anstataŭas `SMEP`; `PAN` anstataŭas `SMAP` nur se la
  procesoro ĝin havas (`cortex-a53` ne havas, `-cpu max` havas).

## 8. Praŝarĝo

- 64-bajta `arm64`-`Image`-kapo: `code0`/`code1`, `text_offset = 0x80000`,
  `image_size`, `flags`, `magic = 0x644d5241` (`ARM\x64`).
- QEMU `virt` ŝargas la bildon ĉe `0x4008_0000` kaj transdonas la DTB en `x0`.
- UEFI-ISO: `mikerno-arm.iso` enhavas la plataĵon `mikerno.img` kaj Limine
  (`BOOTAA64.EFI`) kiel El Torito-bildon; la firmvaro venas el
  `qemu-efi-aarch64`, kaj `just ruli-iso-arm` startigas ĝin.
- Eniro akceptas EL2 aŭ EL1: se `CurrentEL` estas EL2, agordas
  `HCR_EL2.RW = 1` kaj `eret` al EL1. Poste: stako, `.bss`-nuligo, savo de la
  DTB-montrilo.
- Minimuma FDT-analizilo: `/memory`, `/chosen/linux,initrd-start|end`,
  `/soc/serial@9000000`, `/intc`, `/timer`; rezervaj regionoj en
  `EniraInformo::rezervindaj`. Se `/memory` mankas (Limine sub UEFI transdonas
  nur minimuman DTB-on), la analizilo legas la UEFI-memormapon el
  `/chosen/linux,uefi-mmap-*`.
- Moduloj: unu `-initrd` (la `horizonto`-bildo) mapiĝas al `Moduloj`; pluraj
  moduloj venos poste.
- Haltigo: PSCI `SYSTEM_OFF` per `hvc #0`; `wfi` en la buklo de `halti`.

## 9. Esceptoj, interrompoj kaj temporizilo

- `VBAR_EL1`: 16 eniroj po 128 bajtoj — kvar grupoj (Current EL SP0, Current EL
  SPx, Lower EL AArch64, Lower EL AArch32) × kvar specoj (Sync, IRQ, FIQ,
  SError).
- EL0-sync: `SVC` → sistemvoko; data abort → `PaĝaEraro` (procezo mortigata);
  instruction abort/undef → `ProtektaEraro` aŭ fatala, laŭ la komuna politiko.
- EL1-sync: paniko kun `ESR_EL1`, `FAR_EL1`, `ELR_EL1`.
- `InterrompaGardo` uzas `DAIF` anstataŭ `RFLAGS.IF`; la fasado jam havas la
  ĝustajn nomojn.
- GICv2 (`-M virt,gic-version=2`): distribuilo ĉe `0x0800_0000`, procesora interfaco
  ĉe `0x0801_0000`; `IAR`/`EOIR`; la tempila PPI estas 30.
- Ĝenerala temporizilo: `CNTFRQ_EL0`, `CNTP_TVAL_EL0 = frekvenco / 100`,
  `CNTP_CTL_EL0 = 1`; 100 Hz kiel sur `x86_64`, kun la sama antaŭokupa hoko
  `planisto::planumi()`.

## 10. Planado

- Kuntekstŝanĝo ĉe EL1 konservas `x19`–`x28`, `x29`, `x30`, `sp` kaj `DAIF`.
- Uzanta fadeno unue eniras per `eret`: `ELR_EL1 = enirpunkto`,
  `SPSR_EL1 = EL0t`, `SP_EL0 = stakpinto`, `x0`/`x1` = reĝimo/identigilo.
- Escepto el EL0 konservas la plenan kadron sur la kerna stako; reveno uzas
  `eret`, same kiel `iretq` sur `x86_64`.
- La ripoza fadeno kaj la kunhavigitaj kernaj mapoj restas samaj.

## 11. Fazoj

| Fazo | Versio | Enhavo | Akcepto |
| --- | --- | --- | --- |
| A0 | `0.4.3` (sen eldono) | Arkitektura refaktoro: `Kunteksto`, kadro, escepta klasifiko, `ELF_MAŜINO`, `instali_sistemvokon`; `x86_64` sen kondutŝanĝo | `fmt`, `clippy`, `just kontroli`, PVH kaj ISO verdaj |
| A1 | `0.4.3-arm.1` | `arm64`-skeleto: `Image`-kapo, EL2→EL1, PL011, FDT, identeca MMU, PSCI-haltigo, UEFI-ISO; `rustup target add aarch64-unknown-none` | `just ruli-arm` presas la version, la memora linio kaj la serian konzolon; paniko haltigas per PSCI; `just ruli-iso-arm` presas la saman version |
| A2 | `0.4.3-arm.2` | `VBAR_EL1`, GICv2, ĝenerala temporizilo, `DAIF`-gardo, kuntekstŝanĝo | 100 Hz-tikoj, 1000 tikoj, tri fadenoj interplektiĝas kaj finiĝas |
| A3 | `0.4.3-arm.3` | EL0 per `eret`, `svc #0`, `horizonto` por `aarch64`, ELF-maŝino 183, uzantaj esceptoj | la samaj IK-linioj kiel `x86_64`, inkluzive `-32`, sekreta paĝo kaj spiona data abort |
| A4 | `0.4.3-arm.4` | `ramfb` per `fw_cfg`, piksela fiksaĵo, `PXN`/`UXN`/`PAN`, dokumentaro | identa frambufa haketo sur ambaŭ arkoj; `VORTARO.md`, `MEJLOSTONOJ.md` kaj `AGENTS.md` ĝisdatigitaj |

A0 estas antaŭkondiĉo: ĝi ne ŝanĝas la `x86_64`-konduton kaj restas en la
ĉefa branĉo. A1–A4 disvolviĝis en `arm64` kaj estis kunfanditaj post kiam
ĉiu akcepto estis verda.

A1 (2026-11-14, `0.4.3-arm.1`) liveras la skeleto-bildon: `eniro.rs` kun la
`Image`-kapo kaj EL2→EL1, `seria.rs` (PL011), `fdt.rs` (memoro, initrd, seria
adreso), `pagado.rs` (identeca mapo per L0/L1 kaj 1 GiB-blokoj) kaj
`procesoro.rs` (PSCI `SYSTEM_OFF`). La akcepto estas verda:

```text
[MiKerno] Saluton, mi estas MiKerno v0.4.3-arm.1!
[kerno] memoro: 1 regionoj, 256 MiB uzeblaj
[kerno] seria konzolo: PL011 @ 0x9000000
```

QEMU finiĝas per la PSCI-haltigo; la sama haltigo kaptas panikon. La `x86_64`
konstruo, `clippy` kaj la PVH-scenaro restas senŝanĝaj.

Krome `just iso-arm` kunmetas UEFI-booteblan `mikerno-arm.iso`: la ISO enhavas
`mikerno.img` kaj Limine 12.9.6 (`BOOTAA64.EFI`) kiel El Torito-bildon. La
versio estas presata de la kerno mem, same kiel ĉe `x86_64`; `limine.conf` ne
portas version. Sub UEFI la memora linio venas el la UEFI-memormapo, do ĝi
raportas la konvenan memoron (ekzemple `8 regionoj, 229 MiB uzeblaj`) anstataŭ
la tuta `-m`-kvanto.

A2 (2026-11-14, `0.4.3-arm.2`) aldonas la interrompan vojon: `interrompo.rs`
instalas `VBAR_EL1` kun 16 eniroj po 128 bajtoj kaj konservas la plenan kadron
(`x0`–`x30`, `ELR_EL1`, `SPSR_EL1`, `ESR_EL1`, `FAR_EL1`); `gic.rs` pravalorizas
la GICv2-on (`0x0800_0000`/`0x0801_0000`, PPI 30) kaj konfirmas la interrompon
antaŭ la traktilo; `temporizilo.rs` programmas `CNTP_TVAL_EL0` per
`CNTFRQ_EL0`/100; `procesoro.rs` donas la `DAIF`-helpilojn; `planado.rs` faras
la kuntekstŝanĝon (`x19`–`x30`, `sp`, `DAIF`) kaj la unuan eniron per
trampolino. La komuna `kerno::planisto` kaj `interrompo` kompiliĝas sur ambaŭ
arkoj. La akcepto estas verda:

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

A3 (2026-11-15, `0.4.3-arm.3`) ligas la komunan kernon al `EL0` kaj al la
per-procesa `TTBR0_EL1`. `uzanta.rs` enhavas la `eret`-ŝelon (`SPSR_EL1 =
EL0t`, `SP_EL0 = stakpinto`, `x0`/`x1` = reĝimo/identigilo);
`sistemvoko.rs` liveras `svc #0` (`x8` = numero, `x0`–`x3` = argumentoj) kaj
la vektora ŝelko dispetĉas `SVC64` al `kerno::sistemvoko::voki`. `pagado.rs`
konstruas la procezan spacon: la kernaj identecaj eroj (`0..4 GiB`) restas
kunhavataj, la uzanta fenestro ricevas privatajn `L2`/`L3`-tabelojn; la radiko
ŝaltiĝas per `TTBR0_EL1` dum la kuntekstŝanĝo. La komuna kerno raportas la
`TTBR0`-radikon kaj `EL0t`-eniron, la frambufo kaj la APIC/SMAP-provoj restas
nur `x86_64`, kaj `horizonto` konstruiĝas por `aarch64-unknown-none` kun
ELF-maŝino `183`. La bildo liveriĝas kiel `-initrd` aŭ kiel ISO-modulo
(`horizonto.elf`). La akcepto estas verda per ambaŭ vojoj:

```text
[kerno] horizonto 4 pretas: EL0t, SP_EL0=0x101000000, SPSR=0x0, enirpunkto 0x1000006ec
[eraro] procezo 5: datena aborto ĉe rip=0x0000000100000400, far=0x0000000200001000, esr=0x92000007; la procezo estas mortigata
[memoro] izolado (bone): 64998 kadroj antaŭ, 64998 post
[kerno] fina stato: 5 mortintaj, 0 pretaj, 0 blokitaj; 1000 tikoj
```

La IK-linioj (inkluzive `revoko`, `-9`, `-28`, `-32` kaj la negativajn
Ring 3-kazojn) kongruas kun `x86_64`; sub UEFI la kadraj nombroj estas
`58465` pro la firmvara memoro, dum la ceteraj linioj restas samaj.

A4 (2026-11-15, `0.4.3-arm.4`) aldonas la grafikajn kaj firmigajn pecojn de
la paritateco:

- `arkitekturo/arm64/fw_cfg.rs` parolas kun la `fw_cfg`-MMIO de QEMU
  (`/fw-cfg@9020000`, `qemu,fw-cfg-mmio`): legas `FW_CFG_FILE_DIR`, trovas
  `etc/ramfb` kaj skribas per DMA 28-bajtan `RAMFBCfg` (ĉiuj kampoj
  grandendianaj) por 1280×800, `XR24`, paŝo 5120. La frambufo estas
  kontiguaj, identece mapitaj kadroj el `KadraAlokilo`.
- `fdt.rs` rekonas la `fw-cfg`-nodon. La analizilo nun nuligas la `reg`-staton
  ĉe ĉiu nodkomenco kaj konservas la bazan adreson sendepende de la ordo de
  `reg` kaj `compatible`; antaŭe frata nodo povis heredigi sian adreson (la
  PL011 ricevis la `fw_cfg`-adreson).
- `ekrano/` liveras `testi_pikselojn`: gradiento plus teksto, poste FNV-1a
  64-haketo de ĉiuj videblaj pikseloj. La tiparo elektiĝas per la sama regulo
  kiel sur `x86_64`, do la haketo estas komparebla.
- `PXN`/`UXN`/`PAN`: `ID_AA64MMFR1_EL1` decidas ĉu `PAN` ekzistas;
  `SCTLR_EL1.SPAN = 0` refermas la fenestron ĉe ĉiu escepta eniro, kaj
  `StacGardo` malfermas ĝin nur por la kernaj legadoj de uzantaj bufroj. La
  vektora ŝelko raportas `PAN`-malobservon (`EC = 0x25`, `FAR` en la uzanta
  fenestro) anstataŭ la ĝenerala escepto. `-C target-feature=+pan` nur
  permesas al LLVM enkodigi `msr pan`; la subteno mem estas kontrolata.
- `tools/run-qemu-arm.sh` pasigas la plataĵon `Image` (ne la ELF-on), ĉar
  QEMU transdonas la DTB-on en `x0` nur por la Linux-`Image`-kapo. La ISO-vojo
  implicite uzas `cortex-a76`: AAVMF 2025.11 blokiĝas sub TCG kun `-cpu max`.

La akcepto estas la sama haketo sur ambaŭ arkoj; jen `just ruli-arm`
(la ISO-vojo liveras identajn liniojn kaj ankaŭ la `PAN`-provon):

```text
[kerno] PXN subtenata: jes
[kerno] UXN subtenata: jes
[kerno] PAN subtenata: jes
[kerno] PXN ŝaltita; EL1 ne plenumas uzantajn paĝojn
[kerno] UXN ŝaltita; EL0 ne plenumas datumajn paĝojn
[kerno] PAN ŝaltita; EL1 legas uzantajn paĝojn nur en malfermita fenestro
[kerno] frambufo 1280x800, 32 bitoj/pikselo
[kerno] fina stato: 5 mortintaj, 0 pretaj, 0 blokitaj; 1000 tikoj
[ekrano] piksela fiksaĵo 1280x800, 32 bitoj/pikselo, tiparo 12x24; haketo 0xecf1f6dbe1421575
[memoro] PAN-provo: EL1 provos legi la uzantan folion ĉe 0x100000000

[eraro] PAN-malobservo: EL1 provis legi uzantan paĝon ĉe 0x0000000100000000 (esr=0x9600000f)
```

La `x86_64`-ISO (Limine, 1280×800, tiparo 12×24) presas la saman
`haketo 0xecf1f6dbe1421575`; la ceteraj scenaroj restas senŝanĝaj.

## 12. Riskoj kaj malŝarĝoj

| Risko | Malŝarĝo |
| --- | --- |
| FDT-analizado tro kreskas | Minimuma analizilo por `memory`, `initrd`, `serial`, `intc`, `timer` kaj `fw-cfg`; fiksa `virt`-rezerva agordo nur por diagnozo |
| EL2 kontraŭ EL1 ĉe starto | `CurrentEL`-kontrolo kaj malgranda EL2→EL1-ŝelo |
| GICv2 kontraŭ GICv3 | GICv2 unue, eksplicita `gic-version=2`; GICv3 poste |
| `PAN` mankas sur `cortex-a53` | Solvita en A4: la `-kernel`-vojo uzas `-cpu max`; la UEFI-ISO uzas `cortex-a76`, ĉar AAVMF 2025.11 blokiĝas sub TCG kun `max`; la raporto montras la aktivan firmigon |
| Du malantaŭoj drivas | A0 tenas la komunan nukleon en unu loko; M18 aldonos DI-matricon |
| Kani nur `x86_64` | La pruvoj restas sur la komuna kodo; `arm64`-specifaĵoj ricevas QEMU-testojn |
| `Limine` ne subtenas `arm64` | Malaktuala: Limine 12.9.6 havas `BOOTAA64.EFI`; la ISO estas UEFI-bootebla per El Torito, kaj BIOS-ISO ne estas celata |
| `ramfb`/`fw_cfg`-komplekseco | Solvita en A4 per `-device ramfb` kaj `qemu,fw-cfg-mmio`; UEFI `GOP` ne necesis |

## 13. Unuaj paŝoj

1. `rustup target add aarch64-unknown-none`; kontroli `qemu-system-aarch64`
   (jam havebla: QEMU 10.2.1).
2. A0: refaktori la komunajn tipojn kaj restarigi `just kontroli` kaj la du
   `x86_64`-vojojn. (Farita.)
3. A1: `linker-arm64.ld`, `eniro.rs`, `seria.rs`, `fdt.rs`, `pagado.rs`.
   (Farita.)
4. A1-akcepto: `[MiKerno] Saluton, mi estas MiKerno v0.4.3-arm.1!`,
   `[kerno] memoro: …`, `[kerno] seria konzolo: PL011 @ 0x9000000`. (Verda.)
5. A1-ISO: `just iso-arm` kaj `just ruli-iso-arm` per Limine kaj AAVMF.
   (Farita.)
6. A2: `VBAR_EL1`, GICv2, ĝenerala temporizilo, `DAIF`-gardo, kuntekstŝanĝo.
   (Farita; la akcepto estas verda.)
7. A3: EL0 per `eret`, `svc #0`, uzantaj esceptoj. (Farita; la akcepto estas
   verda per `-kernel`/`-initrd` kaj per la UEFI-ISO.)
8. A4: `ramfb` per `fw_cfg`, piksela fiksaĵo, `PAN` kaj dokumentaro.
   (Farita; la akcepto estas la identa haketo
   `0xecf1f6dbe1421575` per `-kernel` kaj per la UEFI-ISO.)

A0–A4 estas efektivigitaj kaj kunfanditaj en `main` kiel M15 (`0.5.0`).
