# Mejloŝtonoj de MiKerno

> Stato: 2026-11-15 · Versio de la workspace: 0.4.6
>
> Ĉi tiu dokumento registras kion MiKerno jam atingis, per kio tio estas
> pruvita, kaj kio venas poste. Ĝi estas la inĝeniera taglibro de la projekto,
> dum [README](../README.md) priskribas la nunan staton.

---

## 1. Superrigardo

| ID | Mejloŝtono | Stato | Ĉefa pruvo |
| --- | --- | --- | --- |
| M0 | Workspace, tri crate-oj, konstrua pipeline | Plenumita | `cargo build --workspace --release` |
| M1 | Du praŝarĝaj protokoloj kaj 32→64-bita eniro | Plenumita | QEMU PVH + Limine/ISO startoj |
| M2 | Praŝarĝa memora mapo kaj bitmapa kadro-alokilo | Plenumita | 7 regionoj, 255 MiB, funkciantaj paĝtabeloj |
| M3 | 4-nivela paĝado kaj propra adresa spaco | Plenumita | CR3-ŝalto, framebuffer-mapado |
| M4 | GDT, IDT, TSS kaj escepta raportado | Plenumita | sen neatenditaj esceptoj; 100 Hz-vektoro |
| M5 | xAPIC kaj PIT-kalibrita temporizilo | Plenumita | 1000 tikoj en ≈10 s |
| M6 | Framebuffer-konzolo kaj COM1 | Plenumita | `1280x800, 32 bpp` (ISO) |
| M7 | Antaŭokupa planisto kaj kuntekstŝanĝo | Plenumita | 3 fadenoj interplektitaj kaj finitaj |
| M8 | Mesaĝa IPC kun kapabloj | Plenumita | mesaĝoj liveritaj, fadeno 3 vekita |
| M9 | Kvalita pordego: fmt, clippy, Kani-pruvoj | Parte | fmt/clippy verdaj; 26 pruvoj maŝine plenumitaj kaj verdaj |
| M10 | Uzantspaca kontrakto (`libmikerno`, `horizonto`) | Plenumita | ABI difinita; `horizonto` plenumiĝas en Ring 3 |
| M11 | Ring 3 kaj sistemvokoj | Plenumita | `int 0x80` el Ring 0 kaj Ring 3, negativaj kazoj, ambaŭ praŝarĝaj vojoj |
| M12 | Per-procesaj adresspacoj | Plenumita | W^X laŭ sekcioj, `CR0.WP`, proceza izolado (spiono `#PF`, viktimo pluvivas), egalaj kadroj |
| M13 | Kerna memoro-firmigo: SMEP, SMAP, KASLR | Plenumita | SMEP, SMAP kaj KASLR pruvitaj per Ring 0-provoj kaj tri startoj |
| M14 | IPC-firmigo: objektoj, rajtoj, tempolimoj | Plenumita | Ring 0/Ring 3-scenaroj kaj AC/CR3/EOI-provoj verdaj (ABI 2) |
| M15 | Dormantaj temporiziloj, prioritatoj, SMP-esploro | Venonta | — |
| M16 | Konservado, dosiersistemo, ŝelo | Venonta | — |
| M17 | CI, testoj, plena pruvaro | Venonta | — |
| M18 | Aparata izolado: IOMMU, PCI/DMA | Post 1.0 | — |

### 1.1. Aparta ŝpuro: MiKerno-ARM

`arm64` estos konstruata kiel aparta ŝpuro kun fazoj A0–A4; ĝi ne anstataŭas
nek antaŭenigas M15–M17. La plano, la nuna kouplingo al `x86_64` kaj la cela
dosieruja strukturo estas en [ARM.md](ARM.md). Dum la ŝpuro la versio estas
antaŭeldono (`0.4.3-arm.N`); la kunfanda numero estos elektita laŭ la tiama
stato.

A0 (arkitektura refaktoro, sen kondutŝanĝo) estas plenumita la 2026-11-14 en
`0.4.3`: `Kunteksto`, `InterrompaKadro` kaj la escepta klasifiko translokiĝis
al la malantaŭo; `SistemvokaKadro`, `FirmigaRaporto` kaj `ELF_MAŜINO` eniris la
fasadon; `privataj_pdptoj` fariĝis `privataj_subtabeloj`. `just verify`, la
PVH-vojo kaj la ISO-vojo restas verdaj.

---

## 2. Plenumitaj mejloŝtonoj

### M0 — Workspace kaj konstrua pipeline

**Celo.** Unu `cargo`-workspace kun klara apartigo inter kerno, komuna ABI kaj
uzantspaco, riproduktebla per unu komando.

**Efektivigo.**

- `Cargo.toml` — workspace kun `mikerno`, `libmikerno`, `horizonto`;
  `edition = "2024"`, `rust-version = "1.85"` (altigita al `1.100` la
  2026-11-13; vidu §4.5), `resolver = "2"`.
- `.cargo/config.toml` — cela triopo `x86_64-unknown-none`, `rust-lld`,
  `relocation-model=pic`, kaj runner al `bash tools/run-qemu.sh` (WSL2/Linux).
- `justfile` — `build-all`, `build-kerno`, `build`, `run`, `iso`, `run-iso`,
  `verify`.
- `build.rs` en la kerno kaj en `horizonto` — ĉiu crate liveras sian propran
  `linker.ld`, do la kerna skripto ne aplikiĝas al la uzanta bildo.
- `unexpected_cfgs` permesas `cfg(kani)` sen avertoj.

**Pruvo.** `cargo build --workspace --release` finiĝas sen avertoj; la kerno
estas 72 KiB, `horizonto` 12 KiB.

**Resto.** Neniu CI; la pipeline estas permana.

---

### M1 — Du praŝarĝaj protokoloj kaj longa reĝimo

**Celo.** La sama kerna bildo ŝarĝiĝu kaj per `qemu -kernel` (Xen-PVH) kaj per
norma ISO (Limine + Multiboot2).

**Efektivigo.** `arkitekturo/x86_64/eniro.rs`:

- Multiboot2-kapo petas memormapon (etikedo 6) kaj 32-bitan framebuffer-on
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
page-blokojn al la paĝado.

**Efektivigo.**

- `memoro/mod.rs` — regionoj, tipoj (`Uzebla`, `Rezervita`, `Acpi`, `Nvs`,
  `Neuzebla`, `Malŝaltita`, `Pmem`, `Nekonata`), `MemoraMapo`, `EniraInformo`,
  `EniraEraro`.
- `mb2.rs` / `pvh.rs` — analizo de ambaŭ informaj strukturoj, inkluzive de
  memmapo, framebuffer, RSDP kaj rezervendaj regionoj.
- `memoro/kadro.rs` — bitaro por 4 GiB (1 048 576 kadroj, 128 KiB da stato).
  Ĉio estas komence okupata; la uzeblaj regionoj estas malmarkitaj; la malalta
  1 MiB, la kerna bildo kaj la praŝarĝaj strukturoj restas rezervitaj.
- `Kadro` portas nur indekson; `aloki()` skanas de monotona `sekva` kaj neniam
  redonas la saman kadron.

**Pruvo.** QEMU raportas 7 regionojn kaj 255 MiB uzeblajn; la paĝtabeloj kaj
framebuffer-mapado sukcese alokas kadrojn.

**Resto.** Neniu liberigo/Reuzo; neniu NUMA, neniu regiona agordo.

---

### M3 — 4-nivela paĝado kun propra adresa spaco

**Celo.** Anstataŭigi la 2 MiB-identecmapon de la ŝelko per detalaj 4 KiB-paĝoj
kaj ŝalti proprietan `CR3`.

**Efektivigo.** `arkitekturo/x86_64/pagxado.rs`:

- `X86AdresaSpaco` posedas radikan kadron; `krei`, `ŝalti`, `mapigi`, `traduki`.
- `PML4 → PDPT → PD → PT`; subaj tabeloj estas alokitaj kaj nuligitaj laŭbezone.
- `traduki` komprenas kaj 4 KiB foliojn kaj 2 MiB grandegajn paĝojn.
- `mapigi` rifuzas nealiniiĝintajn adresojn kaj jam mapitajn foliojn.
- Permesoj (0.2.5): `KERNA_KODO` (R+X), `KERNA_LEGA` (R+NX, ankaŭ por
  `.rodata` kaj moduloj), `KERNA_DATUMOJ` (RW+NX), `SKRIBA` (APIC-MMIO),
  `SKRIBA_SENKAŜE` (framebuffer, `PCD`); `CR0.WP` malpermesas skribon al
  nurlegaj folioj ankaŭ el Ring 0.
- `EFER.NXE` estas ŝaltita en `pravalorizi`.

**Pruvo.** Post `CR3`-ŝalto la kerno daŭre uzas la framebuffer-on (ISO) kaj la
APIC-on; neniu paĝa eraro aperas dum la tuta testa scenaro.

**Resto.** La identeca mapo jam respektas W^X laŭ sekcioj (0.2.5), SMEP
(0.3.0) kaj SMAP (0.3.1) baras Ring 0 de uzantaj folioj, kaj KASLR (0.3.2)
hazardigas la kernan bazadreson per ET_DYN-relokado; la per-procesa mapado
venis en M12. IOMMU-izolado restas en M18.

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

### M6 — Framebuffer-konzolo kaj COM1

**Celo.** Legi la framebuffer-on de Multiboot2 kaj prezenti tekston sen
eksteraj tiparoj.

**Efektivigo.**

- `seria.rs` — COM1, 115200 8N1, FIFO, `\r\n`.
- `ekrano/mod.rs` — 32-bpp konzolo, bitmapaj glifoj desegnitaj pikselo post
  pikselo, kolormaskoj el la framebuffer-informo, titolo
  «MiKerno: La estonteco nun.», malplenigo ĉe la fundo.
- `ekrano/fonto.rs` kaj `ekrano/fonto/*.bin` — kvar antaŭe rasterigitaj strekoj
  (8×16, 12×24, 16×32, 32×64) derivitaj el **DejaVu Sans Mono**. Ĉiu streko
  havas 111 glifojn: 95 ASCII, la dek du ĉapelitaj Esperantaj literoj (`ĉ Ĉ ĝ Ĝ
  ĥ Ĥ ĵ Ĵ ŝ Ŝ ŭ Ŭ`) kaj la citiloj `« » ‹ ›`. `glifo(tiparo, signo)` liveras
  la glifon; nur signoj mankantaj en la tiparo fariĝas `?`.
- `ekrano/mod.rs::elekti_tiparon` elektas la plej grandan strekon, kiu donas
  almenaŭ 80 kolumnojn kaj 24 vicojn, kun nombro da vicoj plej proksima al 33;
  tiel la teksto ne estas skalata kaj restas akra ĉe ĉiu kutima distingivo.
- La framebuffer kaj COM1 nun uzas la saman UTF-8-tekston.
- La strekoj estas generataj per `tools/font/generi.py` (Python 3 + Pillow);
  la DejaVu-permesilo estas en `third_party/dejavu/LICENSE.txt`.

**Pruvo.** La ISO-starto raportas `[kerno] framebuffer 1280x800, 32 bpp` kaj la
kerno uzas ĝin sen eraro; laŭ la elekta regulo tio donas 12×24 (106×33). La
2026-11-12 piksela kontrolo per `screendump` de QEMU konfirmis, ke la ĉapelitaj
literoj kaj la kvar citiloj desegniĝas sen interkovro; la glifoj restas 1-bitaj
bitmapoj en 32-bppa framebuffer sen kontraŭaliasingo. La generilo rifuzas
produkti strekon, se iu glifo elirus sian ĉelon.

**Resto.** Nur 32 bpp, malplenigo anstataŭ rulumo, neniu kursoro aŭ koloraj
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

### M8 — Mesaĝa IPC kun kapabloj

**Celo.** Provizi minimuman, kapablo-bazan mesaĝadon inter fadenoj.

**Efektivigo.** `kerno/ipc.rs`:

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
  `horizonto_start`, maldika `biblioteko` kaj propra `linker.ld`.
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
revenas al la defaŭlta TSS-stako). `planisto::switch` vokas `aktivigi` antaŭ
`ŝalti_kuntekston`; tio estas sekura ĉar ĉiuj fadenaj `Kunteksto`-j, la
planista stato, la kerna bildo kaj la kernostakoj kuŝas en la kerna mapo, kiun
ĉiu proceza spaco kunhavas, kaj `switch` ĉiam estas vokata kun `IF=0`.
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
   `.bss`, `.boot_bss` kaj la cetera uzebla RAM RW NX. La praŝarĝaj moduloj estas
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
   2 MiB-aliniiĝintan bazon en uzebla RAM (≥ 2 MiB, sub 4 GiB, ekster la
   devena bildo, la moduloj, la praŝarĝaj strukturoj kaj la framebuffer), kopias
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

## 3. Partaj mejloŝtonoj

### M9 — Kvalita pordego
**Celo.** Ĉiu ŝanĝo estu pruvebla per ripeteblaj komandoj.

**Havigita.**

- `cargo fmt --all -- --check` — pura.
- `cargo clippy --workspace --release` — sen avertoj.
- `cargo build --workspace --release` — sukcesa.
- 26 Kani-pruvoj: 6 en `memoro/kadro.rs`, 20 en `kerno/ipc.rs`; ĉiuj maŝine
  pruvitaj per Kani 0.68.0 (CBMC 6.11.0) en WSL2 Ubuntu; vidu §4.6.
- `just verify` difinas la formalan pordegon.

**Pruvoj kaj `unwind`-limoj.** La pruvoj grupiĝas laŭ la pruvata tavolo; `—`
signifas la defaŭltan `unwind`-valoron.

| Dosiero | Pruvoj | `unwind` |
| --- | --- | --- |
| `memoro/kadro.rs` | intervalo, preterpaso de uzataj kadroj, duobla liberigo, reuzo, sen duobla alokado, marki/malmarki | 70 (krom la lasta) |
| `kerno/ipc.rs` | mesaĝujo: ena longo, pleneco, preno, FIFO, FIFO inter fontoj, malpleno | 70/20/20/8/8/— |
| `kerno/ipc.rs` | teniloj kaj finpunktoj: kodigo, generacioj, referencoj, libera fendo | —/—/—/20 |
| `kerno/ipc.rs` | kapabloj: rajtoj de `SENDI`/`RICEVI`, `DONI`, transproceza izolado | — |
| `kerno/ipc.rs` | revoko, premrego, senda bito, fonta filtro | —/—/—/8 |
| `kerno/ipc.rs` | tempolimoj: politiko de `r10`, minimuma tiko | — |

**Mankanta.**

- La pruvoj spegulas la logikon per 64-kadra modelo anstataŭ pruvi la
  produktan `KadraAlokilo`.
- La formala pordego rulas nur loke (WSL2); neniu CI, neniu unuopa aŭ bitnivela
  testo, neniu `miri` aŭ `loom`.

**Sekva paŝo.** Instali Kani en CI, aldivi la pruvojn al la produkta kodo per
etaj abstraktaĵoj, kaj aldoni testojn por la arkitekture sendependaj partoj
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
[kerno] framebuffer ne havebla; nur COM1
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
[kerno] framebuffer 1280x800, 32 bpp
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
nur la framebuffer kaj la liberaj kadroj malsamas (`izolado (bone): 63507
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
5. **Piksela kontrolo nur per `screendump`.** La framebuffer estis vidita per
   QEMU `screendump` (2026-11-12): la titolo, la ĉapelitaj literoj kaj la
   citiloj desegniĝas ĝuste. La alta distingivo (16×32/32×64) ne estis provita
   en QEMU; ĝi dependas nur de la sama elekta funkcio, sed meritas apartan
   starton kun pli granda framebuffer. La aspekto sur reala aparato ankoraŭ
   ne estas kontrolita.
6. **`just run-iso` malfermas SDL-fenestron**; por senkapa kontrolo oni voku
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

La tuta workspace estis rekompilita de nulo per Rust 1.100.0:

| Kontrolo | Rezulto |
| --- | --- |
| `cargo build --workspace --release` | Sukceso en 5.76 s |
| `cargo clippy --workspace --release` | Sukceso, sen avertoj (post riparo de 12 novaj lintoj) |
| `cargo fmt --all -- --check` | Neniu diferenco |
| PVH-starto (`qemu64,+smep,+smap`) | Verda: `KASLR: kerna bazo 0x800000`, `reuzo (bone): 10 cikloj, 64839 kadroj antaŭ, 64839 post`, `izolado (bone): 64839 kadroj antaŭ, 64839 post`, 5 mortintaj fadenoj, `SMAP-malobservo` mortigas nur la Ring 3-procezon |
| ISO-starto (Limine, BIOS, `qemu64,+smep,+smap`) | Verda: `KASLR: kerna bazo 0x7800000`, framebuffer `1280x800, 32 bpp`, `izolado (bone): 64837 kadroj antaŭ, 64837 post`, 5 mortintaj fadenoj, 1000 tikoj |

La novaj lintoj estis `manual_is_multiple_of` (9) kaj `collapsible_if` (3); ili
estis riparitaj per `is_multiple_of()` kaj `let`-ĉenoj, do la konduto ne
ŝanĝiĝis. La ISO estis regenerita per la sama ilaro; la Release-aktivaĵo
`mikerno.iso` (v0.3.2) anstataŭiĝis (SHA-256
`7e8c2fb764cb1e55598fbfa5984d6c21b53e188286fd33453c5ed46df8d1674a`) kaj la
ISO-BIOS-starto pruvas la novan bildon.

### 4.6 Formala pordego per Kani (2026-11-13)

Medio: WSL2 Ubuntu, deponejo kopiita al la hejma dosierujo (ext4; `/mnt/d` estas
9p kaj tro malrapida por la kompilo kaj CBMC), Kani 0.68.0, CBMC 6.11.0.
`just verify` vokas
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
   la fonto; simbolaj payload-bajtoj nur pligrandigis la formulon.
2. `unwind` malkreskis al **8** en la du FIFO-pruvoj (maksimume tri mesaĝoj kaj
   tri `preni_de`-paŝoj) kaj al **20** kie la pruvo trairas la plenajn 16
   fendojn (`plena_mesagujo_rifuzas`, `preno_konservas_la_invarianton`).
3. `plenlonga_mesago` kun `unwind 70` aparte pruvas la plenan 64-bajtan buferan
   limon, do la malplena payload ne malstreĉas tiun garantion.

Post la ŝanĝoj la pinto de CBMC ĉe la antaŭe kadukiĝinta pruvo malaltiĝis de
pli ol 3 GB al malpli ol 1 GB, kaj la tuta pordo fariĝas verda.

---

## 5. Venontaj mejloŝtonoj

### M14 — IPC-firmigo

**Celo.** Apartigi finpunktojn de fadenoj kaj firmigi la kapablomodelon.

**Taskoj (Fazoj).**

| Fazo | Versio | Enhavo |
| --- | --- | --- |
| 1 | 0.4.0 | ABI 2: `Tenilo` (indekso + generacio), `Permesoj::REVOKI/PLENAJ`, erar-kodoj `-9/-11/-28/-32/-90/-110`, vokoj 6–11 rezervitaj (`-2`), `ABI_VERSIO = 2`; A1-identeca pruvo; 13 Kani-pruvoj |
| 2 | 0.4.1 | `FinaPunktoObjekto`-tabelo `FINAJ_PUNKTOJ`, generacioj, referenckalkulo, minimuma kapablotabelo, `KREI_FINPUNKTON`(6)/`FERMI`(11); malnova vojo per defaulta finpunkto |
| 3 | 0.4.2 | Permes-kontroloj, `DONI`(7), `SENDI`/`RICEVI` transiras al `rdi` = tenilo, du-nivela generacia kontrolo, `BOOTSTRAP_TENILO`; la fonta filtro venas kun `RICEVI_TEMPO` en Fazo 6 |
| 4 | 0.4.3 | `REVOKI`(8) tutfinpunkta, revokaj vekoj `-32`, purigo ĉe proceza detruo, `Result<Mesago, IpcEraro>` |
| 5 | 0.4.4 | Plen-vica premrego (sendblokado), identeco, `-90`, `IpcStatistiko` |
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
  kaj `tools/versio.sh --kontroli` (parto de `just verify`) kontrolas la
  dokumentojn.
- **C7.** `switch` jam konservas `RFLAGS` (inkl. AC); la AC-aserto restas
  disciplina kaj la kialo estas la `STAC`-vivo, ne la kuntekstŝanĝo.
- **C8.** La kapablotabelo havas 8 liniojn (`[[KapabloEniro; 16]; 8]`); linio
  `0` (kerno) restas ĉiam malplena.
- **C9.** Ne ekzistas novaj plenigaĵoj; testoj pruvas nulan payload-reston kaj
  la identecan transprenon.
- **C10.** Fazo 2 liveras verajn `Tenilo`-jn (minimuma kapablotabelo); Fazo 3
  ŝanĝas `rdi` al tenilo sen duobla modelo.
- **C11.** Montrila kontrolo okazas **antaŭ** tenila kontrolo; la malnovaj
  negativaj valoroj (`nurlega=-1`) restas.
- **C12.** Generacio komenciĝas ĉe `1`; indekso `< 16`; eluzita generacio
  emeritigas la fendon.
- **C13.** Buĝeto: ≈ 27 KiB da finpunktaj vicoj + 2 KiB da kapabloj + ≈ 1 KiB da
  IPC-stato, minus ≈ 15 KiB da malnova `MESAGUJOJ` → neta ≈ 15 KiB.
- **C14.** `REVOKI` estas ĉiam tutfinpunkta; `rsi` devas esti `0`; `FERMI` nur
  fermas la lokan tenilon.
- **C15.** `r10 = u32::MAX` senfina; `0` nebloka (`-11`); alie
  `ceil(ms/10)` tikoj kaj `-110`.
- **C16.** `ricevi_mesagon` transiros al `Result<Mesago, IpcEraro>` en Fazo 4;
  la shim de Fazo 2 validigos, ĉu la cela fadeno vivas, kaj rifuzos per `-3`.

**Progreso.**

| Dato | Fazo | Versio | Pruvo |
| --- | --- | --- | --- |
| 2026-11-13 | 1. ABI 2: `Tenilo`, permesoj, erar-kodoj, rezervitaj vokoj, A1-testo | 0.4.0 | `[ring3] procezo 4/5: idento (bone): sendinto=4/5 (uzanto asertis 999), ricevonto=4/5, resto_nula=jes`; `tenilo_kodigo_roundtrip`; 13 Kani-pruvoj; `size -A`: `.text` 76159, `.rodata` 50212, `.data` 4496, `.bss` 380928 B (bazo por la buĝeto C3) |
| 2026-11-13 | 2. `FinaPunktoObjekto`, generacioj, referencoj, kapablotabelo, `KREI_FINPUNKTON`(6)/`FERMI`(11), defaŭlta finpunkto | 0.4.1 | `[ipc] tabeloj (bone): kreitaj=13, plena=true, fermitaj=13, duobla_fermo=true, nova_generacio=true, malnova_tenilo=true, kerno=true`; `[ring3] procezo 4: finpunktoj (bone): tenilo=1025, fermo=0, duobla=-9, malvalida=-9, nulaj_permesoj=-1, nekonataj_permesoj=-1, rekrea=1281, malnova=-9`; `[ring3] negativaj (F2): celo6=-3, celo0=-3`; `[fadeno 2] ricevis mesaĝon: tipo 1, ricevonto 2 (finpunkta id)`; A1 senŝanĝa; 16 Kani-pruvoj; `size -A`: `.text` 89871, `.rodata` 50724, `.data` 34808, `.bss` 364544 B — RAM-kresko (`.data`+`.bss`) 13928 B ≤ 20 KiB (C3) |
| 2026-11-13 | 3. Permes-kontroloj, `DONI`(7), `SENDI`/`RICEVI` per tenilo, du-nivela kontrolo, `BOOTSTRAP_TENILO` | 0.4.2 | `[ipc] rajtoj (bone): sendi per RICEVI-nura → -1, ricevi per SENDI-nura → -1`; `[ipc] izolado de kapabloj (bone): procezo 7 uzas tenilon de procezo 6 → -9`; `[ipc] dono (bone): rajtoj 0b011 → 0b010, eskalado 0b100 → -1`; `[ring3] procezo 4: rajtoj (bone): sendi_per_ricevi=-1, ricevi_per_sendi=-1`; `[ring3] negativaj (F3): tenilo6=-9, tenilo0=-9`; `[sistemvoko] Ring 0-memtesto: skribo=0, nekonata=-2, nula=-1, troa=-1, cedo=0, krei=-28, doni=-9, fermi=-9`; `[ipc] tabeloj`, A1 kaj `finpunktoj` senŝanĝaj; 20 Kani-pruvoj; `size -A`: `.text` 86431, `.rodata` 51288, `.data` 36056, `.bss` 364544 B — RAM-kresko (`.data`+`.bss`) 1248 B (C3) |
| 2026-11-14 | 4. `REVOKI`(8) tutfinpunkta, revokaj vekoj `-32`, purigo ĉe proceza detruo, `Result<Mesago, IpcEraro>` | 0.4.3 | `[ipc] revoko (bone): sen_revoki=-1, revoko=0, donita=-9, fermo=-9, duobla=-9`; `[fadeno 1] revokis la defaŭltan finpunkton de fadeno 3`; `[fadeno 3] revoka veko (bone): kodo -32`; `[ipc] purigo de procezo 4/5: 1 kapabloj liberigitaj, 1 finpunktoj revokitaj`; `[ring3] procezo 4: revoko (bone): argumento=-1, sen_revoki=-1, fermo_sen=0, revoko=0, sendo=-9, fermo=-9, duobla=-9`; `[sistemvoko] Ring 0-memtesto: …, revoki=-9, revok_argumento=-1`; 21 Kani-pruvoj; `size -A`: `.text` 92799, `.rodata` 51808, `.data` 36104, `.bss` 364544 B — RAM-kresko (`.data`+`.bss`) 48 B (C3) |
| 2026-11-14 | 5. Sendblokado ĉe plena vico, sender-bita masko, `-90`, `IpcStatistiko` | 0.4.4 | `[fadeno 1] premrego (bone): 17 mesaĝoj senditaj; la lasta atendis liberan lokon`; `[fadeno 2] premrego (bone): 17 mesaĝoj, idento sendinto=1, ricevonto=2`; `[fadeno 2] sendblokada revoko (bone): enmetitaj=16, kodo=-32`; `[fadeno 3] revokis finpunkton 1; vekas la blokitan sendanton`; `[sistemvoko] Ring 0-memtesto: …, tro_longa=-90`; `[ipc] statistiko (bone): senditaj=37, ricevitaj=21, premregoj=2, revokoj=7`; `[kerno] fina stato: 5 mortintaj, 0 pretaj, 0 blokitaj`; 23 Kani-pruvoj; `size -A`: `.text` 99551, `.rodata` 52336, `.data` 36424, `.bss` 364544 B — RAM-kresko (`.data`+`.bss`) 320 B (C3) |
| 2026-11-14 | 6. `RICEVI_TEMPO`(9)/`SENDI_TEMPO`(10), `-110`, `-11`, absoluta limdato | 0.4.5 | `[fadeno 4] tempolimoj (bone): nebloka=-11 (0 tikoj), tempolimo=-110 (5 tikoj), mesaĝo=0/tipo 90, senfina=0/tipo 91`; `[ring3] procezo 5: tempolimoj (bone): nebloka=-11, tempolimo=-110, sendo=0, senfina=0, troa=-1`; `[ring3] procezo 6: tempolimoj (bone): nebloka=-11, tempolimo=-110, sendo=0, senfina=0, troa=-1`; `[memoro] izolado (bone): 64809 kadroj antaŭ, 64809 post`; `[ipc] statistiko (bone): senditaj=41, ricevitaj=25, premregoj=2, revokoj=7, tempolimoj=3`; `[kerno] fina stato: 6 mortintaj, 0 pretaj, 0 blokitaj`; 26 Kani-pruvoj; `size -A`: `.text` 102719, `.rodata` 52908, `.data` 36488, `.bss` 380928 B — RAM-kresko (`.data`+`.bss`) 16448 B ≤ 20 KiB (C3) |
| 2026-11-15 | 7. AC/CR3/EOI/ellasa revizio, pruva kompletigo, eldono | 0.4.6 | `[memoro] AC (bone): fermita=true, malfermita=true, interna=true, post_interna=true, fermita_post=true, ŝtelita=true, purigita=true`; `[kerno] gardoj (bone): malŝalto=true, interna=true, restarigo=true (IF antaŭe false)`; `[memoro] CR3 (bone): PCID=malŝaltita, aktiva=0x196000, kongrua=true, aliniita=true`; `[sistemvoko] Ring 0-memtesto: …, tro_longa=-90, sen_eoi=true`; `[apic] EOI (bone): 1000 konfirmoj, 1000 tikoj`; `[memoro] izolado (bone): 64799 kadroj antaŭ, 64799 post`; `[ipc] statistiko (bone): senditaj=41, ricevitaj=25, premregoj=2, revokoj=7, tempolimoj=3`; `[kerno] fina stato: 6 mortintaj, 0 pretaj, 0 blokitaj; 1000 tikoj`; PVH kaj ISO startoj verdaj; 29 Kani-pruvoj (3 novaj: `eoi_nur_por_apic_vektoroj`, `cr3_radiko_estas_pagx_aliniita`, `cr3_radiko_forprenas_nur_la_malaltajn_bitojn`); `size -A`: `.text` 103471, `.rodata` 53476, `.data` 36488, `.bss` 380928 B — RAM-kresko (`.data`+`.bss`) 0 B (C3) |

**Rezultoj de la revizio (Fazo 7).**

- **AC.** `firmigo::testi_ac` pruvas la tutan vivciklon de `StacGardo`: fermita
  antaŭ la unua gardo, malfermita dum ĝi, nedaŭra nestita fenestro, restarigo
  ĉe `Drop`, kaj forigo de `AC`, kiun Ring 3 estus lasinta ŝaltita. La
  interrompa dispetĉo haltas, se `CLAC` ne fermus la fenestron.
- **CR3.** La ISO-starto montris, ke Limine lasas `CR4.PCIDE` (bito 17)
  ŝaltita. `pagxado::pravalorizi` nun reŝargas `CR3` kun nula PCID kaj poste
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

### M15 — Temporiziloj kaj planado

**Celo.** Dormantaj fadenoj, prioritatoj kaj mezuro de plenumtempo.

**Taskoj.** Unuopa vekotemporizilo (min-heap), `dormi(md)`, prioritatoj aŭ
loteria planado, statistiko, eventuala SMP-esploro kun IPI.

**Akceptkriterioj.** Fadeno dormanta 100 ms vekiĝas en tolero; la planisto ne
malsatigas fadenojn; la ripoza kunteksto restas senŝanĝa.

---

### M16 — Konservado kaj ŝelo

**Celo.** Uzebla sistemo, ne nur demonstracio.

**Taskoj.** `virtio-blk`-pelilo, simpla dosiersistemo (ekz. FAT aŭ propra
read-only), ŝelo kun la devigaj Esperantaj komandnomoj `eldiri`, `listi`,
`legi`, `procezoj` kaj `mortigi` (FluoU §2.1); anglaj formoj ne ekzistas.

**Akceptkriterioj.** La ŝelo rulas en Ring 3, legas dosieron kaj lanĉas duan
procezon.

---

### M17 — Inĝeniera matureco

**Celo.** Ripetebla, pruvita kaj dokumentita projekto.

**Taskoj.** CI (Linux), Kani en CI, testoj por arkitekture sendependaj
moduloj, QEMU-matrico (BIOS/UEFI, diversaj memorkvantoj), `cargo-deny` /
permesila kontrolo, regenerado de la ISO en CI.

**Akceptkriterioj.** Ĉiu PR plenumas `fmt`, `clippy`, `build`, `kani` kaj la
QEMU-smokan teston; la ISO estas artefakto de CI, ne permana dosiero.

---

### M18 — Aparata izolado (post 1.0)

**Celo.** Malhelpi, ke aparato atingu arbitran fizikan memoron per DMA.

**Taskoj.** ACPI `DMAR`/`IVRS`-analizo, PCI-enumerado, radikaj kaj kuntekstaj
tabeloj por la IOMMU, DMA-mapoj por ĉiu aparato, interrompa readresado;
QEMU-matrico kun `intel-iommu`.

**Akceptkriterioj.** Aparato povas atingi nur siajn proprajn DMA-mapojn; nevalida
mapo produktas raportitan `FAULT` anstataŭ silentan memoran korupton. Ĉi tiu
mejloŝtono venas post 1.0, ĉar ĝi dependas de la PCI/aparata tavolo el M16.

---

## 6. Inĝenieraj decidoj

| Decido | Kialo | Kosto |
| --- | --- | --- |
| Neniu ekstera Rust-dependaĵo | Aŭdebleco, riparebleco, plena kontrolo | Pli da propra kodo |
| Du praŝarĝaj protokoloj | PVH por rapida `cargo run`; MB2 por reala ISO | Du analiziloj kaj du magioj |
| Esperanto en kodo kaj dokumentaro | Konsekvenco kaj distingiĝo | Malpli da kontribuantoj |
| Identeca mapo | Simpla praŝarĝo kaj aliro al MMIO | Ne plene izolita; W^X nur laŭ sekcioj |
| Statikaj kernaj strukturoj | Sen kerna heap; antaŭvidebla memoro | Fiksaj limoj (8 fadenoj, ...) |
| Interromp-malŝaltaj kritikaj sekcioj | Unuprocesora; simpla kaj sendependa de ŝlosiloj | Ne skaliĝas al SMP |
| `EOI` antaŭ la traktilo | La traktilo rajtas ŝanĝi kuntekston; la sekva tiko devas alveni | Pli da nestitaj interrompoj |
| PIT-kalibrado de la APIC | Busfrekvenco ne estas arkitekture legebla | 50 ms da starta prokrasto |
| 4 GiB fizika limo | Bitara indekso kaj identeca mapo | Ne subtenas pli da RAM |
| Kopiataj mesaĝoj | Sendanto povas tuj reuzi sian bufron | 64-bajta limo |
| Sama virtuala adreso por la sekreta paĝo | La pruvo montras fizikan izoladon, ne nur malsamajn adresojn | Nur unu paĝo por procezo; la dua paĝo restas viktima provo |
| Unu `planisto::lanĉi()` por ambaŭ procezoj | La du procezoj vere interplektiĝas; neniu dua starta vojo | La praŝarĝo devas decidi la tutan aron antaŭe |
| Kani-speguloj | Traktebla stato-spaco | La produkta kodo ne estas pruvita |

---

## 7. Teknika ŝuldo (prioritatigita)

| Prioritate | Ŝuldo | Efiko | Rimedo |
| --- | --- | --- | --- |
| Alta | Neniu `fork`/`exec`; procezoj estas preparitaj de la praŝarĝo | Ne eblas dinamika lanĉo el Ring 3 | M16 (`fork`/`exec`) |
| Alta | Ring 3-esceptoj krom `#PF`/`#GP` haltigas la sistemon | Ne rekaptebla eraro (ekz. `#UD`) | M12/M17 |
| Alta | Neniu CI nek testoj | Regresoj nerimarkeblaj | M17 |
| Alta | Malaktuala ISO | Konfuzaj testrezultoj | CI + versio |
| Malalta | Dokumentoj kaj `Cargo.toml` povas malkongrui | Konfuzaj eldonoj | `tools/versio.sh --kontroli` en `just verify` |
| Meza | IPC finpunkto = fadeno | Ne eblas sendependaj objektoj | M14 |
| Meza | Sekreta paĝo estas unu fiksa mapado por procezo | Neniu ĝenerala `mmap`/`brk` por uzantaj datumoj | M16 |
| Meza | Neniu IOMMU | Aparata DMA ne estas izolita | M18 |
| Meza | Framebuffer sen rulumo | Malbona uzsperto | M6-poste |
| Malalta | `Kunhavigita::akiri` prenas `&self` | Facila misuzo | reviziita kaj dokumentita (Fazo 7) |
| Malalta | Kani-modelo ≠ produkta kodo | Pruvoj ne rekte aplikeblas | M9 |

---

## 8. Difino de «1.0»

MiKerno atingos 1.0 kiam:

1. `horizonto` (aŭ alia uzanta procezo) ruliĝas en Ring 3 kun propra adresa
   spaco, sen povoj legi la kernon aŭ alian procezon.
2. Sistemvokoj `SKRIBI`, `FINI`, `CEDI`, `SENDI`, `RICEVI` funkcias kaj estas
   dokumentitaj en `libmikerno`.
3. IPC havas apartajn finpunktajn objektojn, rajtojn kaj tempolimojn.
4. Kadroj kaj paĝtabeloj estas liberigeblaj; longa testado ne elĉerpas memoron.
5. La ISO estas konstruata de CI, kaj `fmt`, `clippy`, `build`, `kani` kaj
   QEMU-smoka testo estas devigaj.
6. La dokumentaro (README, ĉi tiu dokumento, sistemvoka referenco) estas
   kompleta kaj kongrua kun la kodo.

---

*Ĉi tiu dokumento estas vivanta; ĝisdatigu ĝin kune kun ĉiu mejloŝtono.*
