# MiKerno

> «La estonteco nun.»

MiKerno estas eta, memstara x86_64-kerno verkita tute en Rust (`#![no_std]`,
`#![no_main]`), sen eksteraj Rust-dependaĵoj kaj sen C-kodo en la kerno mem. La
kodo, la protokoloj kaj la dokumentaro estas en Esperanto.

La celo de la projekto estas fari legeblan, hobleblan kaj pruveblan
demonstration de kompleta unuprocesora kerno: de la unua instrukcio en 32-bita
reĝimo ĝis antaŭokupa planisto, mesaĝa IPC kaj uzanta procezo en Ring 3.

- **Versio:** 0.2.1 (workspace)
- **Arkitekturo:** x86_64, unuprocesora
- **Cela triopo:** `x86_64-unknown-none`
- **Eldono de Rust:** 2024, `rust-version = 1.85`
- **Statuso:** funkcianta prototipo — vidu [Mejloŝtonoj](docs/MEJLOSTONOJ.md)

---

## Enhavo

1. [Kapabloj](#kapabloj)
2. [Rapida starto](#rapida-starto)
3. [Arĥitekturo](#arĥitekturo)
4. [Praŝarĝo kaj eniro](#praŝarĝo-kaj-eniro)
5. [Memoro](#memoro)
6. [Interrompoj, APIC kaj temporizilo](#interrompoj-apic-kaj-temporizilo)
7. [Planisto](#planisto)
8. [IPC kaj kapabloj](#ipc-kaj-kapabloj)
9. [Eligo: COM1 kaj framebuffer](#eligo-com1-kaj-framebuffer)
10. [Uzantspaco](#uzantspaco)
11. [Konstruado kaj kontrolado](#konstruado-kaj-kontrolado)
12. [Testa scenaro](#testa-scenaro)
13. [Strukturo de la dosieroj](#strukturo-de-la-dosieroj)
14. [Limigoj kaj sekva laboro](#limigoj-kaj-sekva-laboro)
15. [Kontribuado](#kontribuado)
16. [Licenco kaj triaj komponantoj](#licenco-kaj-triaj-komponantoj)

---

## Kapabloj

| Kapablo | Stato | Kie |
| --- | --- | --- |
| Du praŝarĝaj protokoloj: Xen-PVH (`qemu -kernel`) kaj Multiboot2 (Limine/ISO) | Funkcias | `mikerno/src/arkitekturo/x86_64/eniro.rs` |
| 32-bita → PAE → longa reĝimo, propraj praŝarĝaj paĝtabeloj | Funkcias | `eniro.rs`, `mikerno/boot/linker.ld` |
| Memora mapo el la praŝarĝilo, rezervitaj regionoj | Funkcias | `mb2.rs`, `pvh.rs`, `memoro/mod.rs` |
| Modula liverado: Multiboot2 tipo 3 kaj PVH-modlisto (`horizonto`) | Funkcias | `mb2.rs`, `pvh.rs`, `memoro/mod.rs` |
| Bitmapa kadro-alokilo por la unuaj 4 GiB, kun `liberigi` kaj rekuzo | Funkcias | `memoro/kadro.rs` |
| Kaskada forigo de adresa spaco (`detruu`) kun liberigo de paĝtabeloj | Funkcias | `arkitekturo/x86_64/pagxado.rs` |
| Per-procesa adresa spaco: privata PML4, kunhavataj kernaj mapoj (`krei_kunhavantan`) | Funkcias (fundamento) | `kerno/procezo.rs`, `pagxado.rs` |
| Strikta uzanta mapado (`mapi_uzantan`): fenestra limkontrolo kaj devigata bito `U` | Funkcias | `memoro/virtuala.rs`, `kerno/uzantspaco.rs` |
| 4-nivela paĝado (PML4 → PDPT → PD → PT), NX, PCD | Funkcias | `arkitekturo/x86_64/pagxado.rs` |
| GDT, 256-vektora IDT, escepta raportado | Funkcias | `gdt.rs`, `idt.rs`, `interrompo/mod.rs` |
| Uzantaj DPL3-segmentoj kaj `int 0x80`-pordego (interrompa, `0xEE`) | Funkcias | `gdt.rs`, `idt.rs`, `kerno/sistemvoko.rs` |
| Kvin sistemvokoj (`SENDI`/`RICEVI`/`SKRIBI`/`FINI`/`CEDI`), unueca `Mesago`, montrila kontrolo | Funkcias (Ring 0 kaj 3) | `kerno/sistemvoko.rs`, `arkitekturo/x86_64/sistemvoko.rs` |
| TSS kun `RSP0` por Ring 3 → Ring 0 | Funkcias | `tss.rs` |
| xAPIC (MMIO), malŝalto de 8259-PIC, spuria vektoro `0xFF` | Funkcias | `apic.rs` |
| 100 Hz perioda temporizilo, kalibrita kontraŭ la PIT | Funkcias | `temporizilo.rs` |
| Antaŭokupa rondiga planisto, 8 fadenoj + ripoza kunteksto | Funkcias | `kerno/planisto.rs` |
| Mesaĝa IPC: finpunktoj, kapabloj, blokado/vekado | Funkcias | `kerno/ipc.rs` |
| Framebuffer-teksta konzolo (4 strekoj, 8×16 … 32×64) kaj COM1 | Funkcias | `ekrano/`, `seria.rs` |
| Komuna kerna/uzanta ABI (`libmikerno`) | Difinita | `libmikerno/` |
| Uzanta fenestro super 4 GiB, ELF64-ŝargo kun bito `U`, 64 KiB stako | Funkcias | `kerno/uzantspaco.rs`, `memoro/virtuala.rs` |
| Ring 3-transiro per `iretq` (`CS=0x23`, `SS=0x1B`, `IF=1`) | Funkcias | `arkitekturo/x86_64/uzanta.rs` |
| Unua uzanta procezo `horizonto`: `SKRIBI` + `FINI` el Ring 3 | Funkcias | `horizonto/`, `kerno/sistemvoko.rs` |
| 12 formalaj pruvoj per Kani | Verkitaj | `kadro.rs`, `ipc.rs` |

> `horizonto` estas liverata kiel praŝarĝa modulo, muntata super 4 GiB kun la
> bito `U` tra ĉiuj paĝtabelaj niveloj, kaj ekzekutas en Ring 3 per `iretq`.
> Ĝi skribas sian saluton per `int 0x80` kaj eliras per `fini(0)`; la planisto
> tiam revenas al la ripoza kunteksto. La proceza adresa spaco jam ekzistas
> (privata PML4 kun kunhavataj kernaj mapoj), sed `CR3` ankoraŭ ne ŝanĝiĝas dum
> kuntekstŝanĝo kaj `TSS.RSP0` restas komuna (M12-fazoj 3–5).

---

## Rapida starto

### Postuloj

| Ilo | Versio / noto |
| --- | --- |
| Rust | 1.85 aŭ pli nova (eldono 2024); projekto kontrolita per 1.99.1 |
| Cela triopo | `x86_64-unknown-none` |
| QEMU | `qemu-system-x86_64` (Windows: `C:\Program Files\qemu`) |
| `just` | Opcia, sed rekomendata por la difinitaj taskoj |
| Git Bash | Bezonata sur Windows por `tools/run-qemu.sh` |
| Kani | Nur por `just verify` (`cargo kani`) |

Instalu la celan triopon:

```sh
rustup target add x86_64-unknown-none
```

### Konstrui kaj ruli

```sh
just build-all     # konstruas la tutan workspace (kerno, libmikerno, horizonto)
just build         # konstruas nur la kernon (= build-kerno)
just run           # QEMU -kernel, rekte per la PVH-noto (sen ISO)
just iso           # kreas mikerno.iso per Limine + Multiboot2
just run-iso       # rulas la ISO-on (QEMU-fenestro montras la Limine-menuon)
just verify        # formalaj pruvoj per Kani (postulas `cargo kani`)
```

Ekvivalentaj rektaj komandoj:

```sh
cargo build --workspace --release
cargo build -p mikerno --release
cargo run -p mikerno --release          # uzas la `runner`-on el .cargo/config.toml
```

`.cargo/config.toml` fiksas la celon `x86_64-unknown-none` kaj, sur Windows,
lanĉas QEMU per `tools\run-qemu.cmd`. Sur Linux vi povas voki rekte:

```sh
tools/run-qemu.sh release
tools/run-qemu.sh iso
QEMU_DISPLAY=none tools/run-qemu.sh iso   # senkapa ISO-starto
```

En la QEMU-konzolo: `Ctrl+A x` tuj finas QEMU, `Ctrl+A c` malfermas la
monitoron (tie `quit`). La `-serial mon:stdio` muntado estas necesa ĉar
Windows-QEMU forprenas `ENABLE_PROCESSED_INPUT` de la konzolo.

> **Atentu:** `mikerno.iso` kaj `iso_root/` estas ignorataj de git. Post ĉiu
> kerna ŝanĝo rerulu `just iso`; alie la ISO enhavos malnovan bildon. Tio
> efektive okazis dum la kontrolo de 2026-11-12 (la ISO montris version
> `v0.1.1`, dum la fonto estis `v0.1.0`).

---

## Arĥitekturo

La workspace havas tri crate-ojn:

| Crate | Speco | Rolo |
| --- | --- | --- |
| `mikerno` | `no_std` binara | La kerno: praŝarĝo, memoro, interrompoj, planisto, IPC, konzolo |
| `libmikerno` | `no_std` biblioteko | Komunaj `#[repr(C)]`-difinoj: mesaĝoj, kapabloj, sistemvokaj numeroj |
| `horizonto` | `no_std` binara | La unua uzanta procezo, ligita jam super 4 GiB (`0x1_0000_0000`) |> **Noto:** la tuta workspace uzas `code-model=large`, ĉar `horizonto` estas
> ligita super 4 GiB; la statika reloka modelo restas, do neniu dinamika ligilo
> aperas.

Dependa grafo:

```text
horizonto ──> libmikerno
mikerno    (neniu ekstera dependaĵo)
```

La kerna nukleo estas arkitekture sendependa: ĝi tuŝas la platformon nur tra
`crate::arkitekturo`. La sola realigita malantaŭo estas `x86_64`.

### Starta fluo

```text
firmvaro / QEMU / Limine
        │
        ▼
32-bita enirŝelko (_komenco)          eniro.rs
  · nuligas 6 paĝojn, konstruas identecan mapon de 4 GiB per 2 MiB paĝoj
  · PAE + LME + PG, GDT, saltas en longan reĝimon
  · nuligas .bss, starigas stakon, vokas kerna_eniro
        │
        ▼
kerna_eniro                          main.rs
  · pravalorizi: COM1, GDT, IDT, EFER.NXE
  · analizi_eniron: Multiboot2 aŭ Xen-PVH
        │
        ▼
kerno::ĉefa                          kerno/mod.rs
  · kadro-alokilo ← memora mapo + rezervoj (inkl. praŝarĝaj moduloj)
  · provo de liberigo: proviza PML4 → mapo → `detruu`, kadroj reakiritaj
  · nova PML4 (4 KiB paĝoj), identeca mapo, framebuffer (PCD)
  · modulaj regionoj identece mapitaj, CR3 ŝalto, konzolo, TSS/RSP0
  · APIC + temporizilo (100 Hz), 3 testaj fadenoj
  · uzantspaco: ELF64-segmentoj + stako super 4 GiB, bito `U` (`mapi_uzantan`)
  · procezo 4: privata PML4, kernaj mapoj kunhavataj, fenestro malplena
  · `horizonto` en Ring 3 per `iretq`; `FINI` revenigas la planiston
  · IPC-testo, planisto, 10 s da observado, halto
```

---

## Praŝarĝo kaj eniro

`eniro.rs` estas unu `global_asm!`-bloko; ekstera asemblero ne necesas. Ĝi
subtenas du protokolojn samtempe:

1. **Xen-PVH** — `qemu-system-x86_64 -kernel`. La ELF enhavas
   `XEN_ELFNOTE_PHYS32_ENTRY`; `EBX` montras al `hvm_start_info`. La magio
   estas `0x336e_c578` (`pvh.rs`). `horizonto` venas per `-initrd` kiel ero de
   `modlist_paddr` (32-bajta `hvm_modlist_entry`).
2. **Multiboot2** — Limine el ISO. La kapo petas la memormapon (etikedo 6) kaj
   32-bitan framebuffer-on (etikedo 5). `EAX` estas `0x36d7_6289`, `EBX` montras
   al la informo (`mb2.rs`). `horizonto` venas per `module_path:` en
   `limine.conf` kiel etikedo de tipo 3.

La ŝelko unue identece mapas la unuajn 4 GiB per 2 MiB grandegaj paĝoj, por ke
la 64-bita kodo restu atingebla tuj post `CR0.PG`. La tabeloj (`_pml4`, `_pdpt`,
`_pd0..3`) kaj la 16 KiB-a praŝarĝa stako vivas en `.boot_bss`, aparte de
`.bss`: la longreĝima eniro nuligas `.bss` uzante ĝuste tiujn paĝojn.

La liga skripto metas la kernon je 1 MiB (`mikerno/boot/linker.ld`) kaj
eksportas `__kerna_komenco`, `__kerna_fino`, `__bss_komenco`, `__bss_fino`.

---

## Memoro

### Kadro-alokilo

`memoro/kadro.rs` tenas bitaron por la unuaj 4 GiB: 1 048 576 kadroj da 4 KiB,
t.e. 128 KiB da bitaro en `.bss`. La alokilo:

1. markas ĉion okupata;
2. malmarkas la uzeblajn regionojn de la praŝarĝila memora mapo;
3. denove rezervas la malaltan 1 MiB (BIOS/legacy), la kernan bildon
   (`__kerna_komenco..__kerna_fino`) kaj la regionojn el la praŝarĝilo
   (Multiboot2-informo, PVH-memmapo, RSDP).

`aloki()` skanas de `sekva` antaŭen. `liberigi(kadro)` malmarkas la biton,
nuligas la enhavon (por ke sekva procezo ne vidu malnovajn datumojn) kaj
retreatigas `sekva`, do la kadro estas reuzata tuj; duobla liberigo estas
senefika. `liberaj()` raportas la nombron de liberaj kadroj.

`X86AdresaSpaco::detruu` (tra `AdresaSpaco`) malmapas la tutan uzantan
fenestron, liberigas la foliajn kadrojn kaj ĉiujn paĝtabelojn, kiuj restas
malplenaj, plus la radikan kadron. Tabeloj kun kernaj mapoj restas netuŝitaj,
ĉar la intervalo estas limigita al la uzanta fenestro. Ĉe starto la nukleo
plenumas provon: ĝi kreas provizan spacon, mapas unu paĝon, detruas ĝin kaj
komparas la nombrojn de liberaj kadroj.

### Paĝado

`X86AdresaSpaco` konstruas freŝan PML4-on kaj mapas per 4 KiB folioj:

- la malaltan 1 MiB,
- la kernan bildon (aparte, ĉar iuj praŝarĝiloj raportas ĝin rezervita),
- ĉiujn uzeblajn regionojn (identece, `KERNELA` = RWX),
- la framebuffer-on (`SKRIBA_SENKAŜE`: skribebla, neplenumebla, `PCD`),
- la APIC-MMIO-on (`SKRIBA`).

`EFER.NXE` estas ŝaltita antaŭ ol oni uzas biton 63. `mapigi` rifuzas
nealiniiĝintajn adresojn kaj jam mapitajn foliojn (`Konflikto`); `traduki`
komprenas ankaŭ 2 MiB grandegajn paĝojn. La identeca mapo estas RWX, kio estas
oportuna sed ne sekura longtempe (vidu [Limigoj](#limigoj-kaj-sekva-laboro)).

`krei_kunhavantan(&self, …)` konstruas procezan spacon super ekzistanta kerna
spaco. Por ĉiu ĉeesta PML4-ero ĝi kontrolas la suban PDPT-on: se ĝi enhavas
iun ajn eron kun `U=1`, aŭ se la ero enhavas parton de la uzanta fenestro, oni
alokas **privatan** PDPT-on kaj kopias nur la `U=0`-erojn; alie la PDPT-kadro
estas kunhavata laŭvorte. Ĉar la fenestro kuŝas sub `PML4[0]`, tiu ero neniam
estas kunhavata — alie novaj uzantaj mapoj aperus ankaŭ en la fonta spaco. La
PML4-eroj estas kopiitaj kiel krudaj `u64`-valoroj (fizika adreso + flagoj),
neniam dereferencitaj. `mapi_uzantan` kaj `mapi_uzantan_regionon` devigas la
intervalon `[UZANTA_BAZO, UZANTA_FINO)`, kontrolas `checked_add` kontraŭ
ciferpremo kaj ŝaltas `U` ĉe la folio kaj ĉiuj gepatraj niveloj.

---

## Interrompoj, APIC kaj temporizilo

- **GDT** (`gdt.rs`): nula ero, kerna kodo `0x08` (L=1), datumoj `0x10`, TSS
  `0x18`. `instali()` reŝargas `CS` per malproksima `retfq`.
- **IDT** (`idt.rs`): 256 vektoroj. Vektoroj kun erarkodo (8, 10–14, 17, 21, 29,
  30) uzas apartan ŝablonon. Ĉiuj enirpunktoj saltas al `isr_komuna`, kiu
  konservas 15 ĝeneralajn registrojn plus `rsp`, vokas
  `interrompa_dispetcho` kaj finas per `iretq`. La aranĝo de
  `InterrompaKadro` estas kontrolita per `const`-asertoj.
- **Dispetĉo** (`interrompo/mod.rs`): la aparata `EOI` estas sendita *antaŭ* la
  traktilo por vektoroj ≥ `0x20` (krom la spuria `0xFF`), ĉar traktilo rajtas
  ŝanĝi la kuntekston kaj la sekva tiko devas povi alveni dum la nova fadeno
  plenumas. Nekonataj esceptoj (`< 0x20`) raportas `rip`, `rsp`, `rflags`,
  `erarkodo` kaj — por vektoro 14 — `CR2`, poste haltigas la sistemon.
- **APIC** (`apic.rs`): la 8259-PIC estas maskita, la APIC-bazo venas el
  `IA32_APIC_BASE`, la spuria vektoro estas `0xFF`. Restas en MMIO-reĝimo (la
  x2APIC-bito estas nuligita de la masko).
- **Temporizilo** (`temporizilo.rs`): perioda reĝimo, 100 Hz, divido 16. Ĉar la
  APIC-busa frekvenco ne estas arkitekture legebla, ĝi estas mezurita dum ≈50 ms
  kontraŭ PIT-kanalo 2 (1,193182 MHz). Se la kalibrado malsukcesas, oni supozas
  norman 100 MHz-buson (`6 250 000` kalkuloj/s). La starta testo mezuris
  ≈62,6M kalkuloj/s en QEMU.

---

## Planisto

`kerno/planisto.rs` realigas unuprocesoran antaŭokupan rondigan planiston:

- maksimume **8 fadenoj** plus la **ripoza fadeno** (id `0`), kiu portas la
  praŝarĝan kuntekston;
- cirkla preta vico kun unu plia loko, do `kapo == vosto` signifas malplena;
- stato `Preta`, `Kuranta`, `Blokitaj`, `Mortinta`;
- la temporizila traktilo vokas `planumi()` ĉiun 10 ms; fadeno povas cedi
  (`cedi`), blokiĝi (`bloki`) aŭ finiĝi (`morti`);
- kuntekstŝanĝo (`planado.rs`) konservas 18 registreojn (`rax`…`r15`, `rsp`,
  `rip`, `rflags`) per `#[repr(C)]` `Kunteksto`; la unua eniro uzas artefaritan
  stakon kun `fadeno_trampolino`, kiu ŝaltas interrompojn kaj transdonas la
  fadenan identigilon en `rax` → `rdi`;
- ĉiu fadeno havas 16 KiB stakon; la testaj stakoj estas statikaj;
- la uzanta fadeno (id 4) estas planita post la testaj: ĝia enirpunkto
  transiras al Ring 3 per `iretq`, kaj `SYS_FINI` mortigas ĝin, do la planisto
  elektas la ripozan kuntekston kaj la observado daŭras.

`switch` revenas nur kiam la malnova kunteksto estas planita denove. La ripoza
kunteksto konservas sian interrompan kadron sur la stako, do `planumi` povas
reveni tra la dispetĉo kaj `iretq` kiam la ripoza fadeno rekaptas la procesoron.

---

## IPC kaj kapabloj

`kerno/ipc.rs` difinas minimuman mesaĝan IPC-on:

- `FinaPunkto` (`id`, `Permesoj`) kaj derivita `Kapablo` — la nura rajto uzi
  finpunkton;
- permesoj `SENDI`, `RICEVI`, `ĈIUJ` (bita aro);
- mesaĝo kun `tipo` (u64) kaj ĝis **64 bajtoj** da datumaro; la datumaro estas
  kopiata, do la sendanto rajtas tuj reuzi sian bufron;
- ĉiu fadeno havas vicon por **16 mesaĝoj** (`Mesagujo`), FIFO inter la sama
  fonto;
- `sendi_mesagon` metas la mesaĝon kaj **vekas** la celon se ĝi estis blokita;
- `ricevi_mesagon(fonto)` blokas la vokanton ĝis mesaĝo de tiu fonto alvenas;
  la ripoza kunteksto (`id 0`) neniam blokiĝas — ĝi ricevas malplenan mesaĝon.

Ĉiuj operacioj tenas interrompojn malŝaltitaj per `InterrompaGardo`, kiu
restarigas la antaŭan `RFLAGS.IF`-staton ĉe `Drop`.

Nuna limigo: la finpunkta identigilo estas rekte la fadena identigilo, kaj ne
ekzistas aparta kapablotabelo, rajtorevoko aŭ tempolimoj.

---

## Eligo: COM1 kaj framebuffer

- **COM1** (`seria.rs`): 16550 UART ĉe `0x3F8`, 115200 baudoj, 8N1, demandata
  reĝimo, `\n` → `\r\n`. Ĉi tio estas la arkitektura konzolo kaj ĉiam funkcias.
- **Framebuffer** (`ekrano/`): 32-bpp Linuksa framebuffer el Multiboot2, desegnita
  per antaŭe rasterigitaj bitmapaj tiparoj derivitaj el **DejaVu Sans Mono**.
  Ekzistas kvar strekoj — 8×16, 12×24, 16×32 kaj 32×64 — kaj la konzolo elektas
  la plej grandan, kiu donas almenaŭ 80 kolumnojn kaj 24 vicojn, kun nombro da
  vicoj plej proksima al 33. Ĉiu streko enhavas la 95 ASCII-signojn kaj 16
  aldonajn glifojn: `ĉ Ĉ ĝ Ĝ ĥ Ĥ ĵ Ĵ ŝ Ŝ ŭ Ŭ` kaj `« » ‹ ›`. Aliaj
  ne-ASCII-signoj fariĝas `?`. Postuloj: bazo 4-bajte aliniiĝinta, `bpp == 32`,
  larĝo ≥ 320, alto ≥ 32, `paŝo ≥ larĝo × 4`. La seriala eligo restas plena
  UTF-8. Kiam la tekstfluo atingas la fundon, la ekrano estas malplenigita (ne
  vere rulumita).

  Tipika elekto: 800×600 → 8×16 (100×37), 1280×800 → 12×24 (106×33),
  1920×1080 → 16×32 (120×33), 2560×1440 → 16×32 (160×45), 3840×2160 → 32×64
  (120×33). La strekoj estas generataj per `tools/font/generi.py`; la fonto
  mem ne estas enŝovita, nur la bitaj tabeloj.
- Se framebuffer mankas (Xen-PVH aŭ malnova firmvaro), la konzolo restas
  malŝaltita kaj la eligo okazas nur sur COM1.

---

## Uzantspaco

### Komuna kontrakto

`libmikerno` fiksas la komunan kontrakton:

- `Mesago` (`#[repr(C)]`, 64-bajta datumaro, `tipo`, `sendinto`, `ricevonto`),
- `FinaPunkto`, `Kapablo`, `Permesoj`,
- sistemvokaj numeroj: `SYS_SENDI = 1`, `SYS_RICEVI = 2`, `SYS_SKRIBI = 3`,
  `SYS_FINI = 4`, `SYS_CEDI = 5`,
- registra ABI: `rax` = numero, `rdi`/`rsi`/`rdx`/`r10` = argumentoj, `rax` =
  rezulto; la voko estas `int 0x80`.

### Uzanta fenestro

La uzanta virtuala spaco estas rezervita supre de la kerna identeca mapo:

| Limo | Valoro |
| --- | --- |
| `UZANTA_BAZO` | `0x1_0000_0000` (4 GiB) |
| `UZANTA_FINO` | `0x80_0000_0000` (512 GiB) |
| `UZANTA_STAKO_SUPRO` | `UZANTA_BAZO + 16 MiB` = `0x1_0100_0000` |
| `UZANTA_STAKO_GRANDE` | 64 KiB (`0x100ff0000..0x101000000`) |

`PaĝaPermesoj` havas la biton `uzanta`; la mapado metas `U` en ĉiun nivelon
(PML4 → PDPT → PD → PT), do `kontroli_uzantan` povas pruvi per unu promenado,
ke la tuta vojo estas alirebla el Ring 3. La folia nivelo ricevas `W` nur por
skribaj regionoj kaj `NX` por datumoj kaj stako.

### Ŝargo de la bildo

`kerno/uzantspaco.rs` legas la modulon kiel ELF64:

1. validigas la kaplinion (magio, klaso 64, LE, maŝino 62), la tabelon de
   programaj kaplinioj kaj ĉiun `PT_LOAD`-eron (dosiera longo ≤ memora longo,
   dosieraj kaj virtualaj limoj ene de la modulo kaj de la uzanta fenestro,
   maksimume 16 segmentoj);
2. kalkulas la bildajn limojn kaj kontrolas, ke la enirpunkto estas ene;
3. muntas paĝon post paĝo: ĉiu folio ricevas la **unuon** de la permesoj de ĉiuj
   segmentoj, kiuj ĝin tuŝas (LLVM ofte kunigas la segmentojn sur unu paĝo),
   poste oni nuligas la kadron kaj kopias la dosierajn bajtojn de ĉiu segmento;
4. mapas la 64 KiB-an stakon ĉe `UZANTA_STAKO_SUPRO`.

Post tio `pravalorizi()` raportas la bildajn limojn, la enirpunkton kaj la
stakon, kaj rulas memkontrolon: skribo/lego tra la uzanta fenestro el Ring 0
devas sukcesi, dum nurlega skribo, kerna adreso, nemapita adreso, translima
intervalo kaj adreso ekster la fenestro devas esti rifuzitaj laŭ la bito `U`.

`horizonto` estas liverata de ambaŭ praŝarĝaj vojoj — Multiboot2 tipo 3 el
`limine.conf`, aŭ la PVH-modlisto el `-initrd` (``just run`` aldonas ĝin
aŭtomate).

### Proceza adresa spaco

`kerno/procezo.rs` donas la abstrakton `Procezo`: identigilo, privata `Spaco`,
estonta kerna stakpinto (`kerna_stako_pinto`, muntos ĝin Fazo 3) kaj stato
(`Preta`, `Kuranta`, `Mortinta`). La nukleo estas procezo `0` sen ero en la
tabelo; la tabelo tenas ĝis 8 procezojn. `Fadeno` nun portas `procezo`-idon,
kaj la planisto spegulas `Kuranta`/`Preta`/`Mortinta` en la proceza stato.

Ĉe starto la nukleo, post la ŝargo de `horizonto`, kreas procezon `4` per
`krei_kunhavantan`. La konzolo konfirmas ĝin per:

```text
[procezo] 4: Preta, kerna stako 0x0, kerna mapo kunhavata, uzanta fenestro privata
```

La kontrolo legas `traduki(0x1000)` (kerna mapo ĉeestas) kaj
`traduki(UZANTA_BAZO)` (neniu uzanta folio kopiiĝis). `CR3` ankoraŭ ne
ŝanĝiĝas dum planado; tion kaj la per-procesan `TSS.RSP0` alportos la sekvaj
fazoj de M12.

### Transiro al Ring 3

Post la testaj fadenoj la praŝarĝa kunteksto aldonas la uzantan fadenon (id 4)
kaj planas ĝin. `arkitekturo/x86_64/uzanta.rs` konstruas la `iretq`-kadron sur
la fadena stako:

```text
push SS=0x1B
push RSP=UZANTA_STAKO_SUPRO - 8
push RFLAGS=0x202        (IF=1 + rezervita bito 1)
push CS=0x23
push RIP=enirpunkto
iretq
```

La stakpinto estas malaltigita je 8 por ke la x86-64-ABI vidu `rsp % 16 == 8`
ĉe la funkcia eniro. La `int 0x80`-pordego havas DPL3, do Ring 3 rajtas voki
`SKRIBI` kaj `FINI`; la CPU ŝaltas al `TSS.RSP0` por ĉiu reveno al Ring 0.
Post `fini(0)` la fadeno mortas kaj la planisto revenas al la ripoza
kunteksto, kiu daŭrigas la observadon.

Antaŭ la eliro `horizonto` kontrolas la negativajn kontraktojn el Ring 3: vojo
`0x7FFF` (nekonata), nula montrilo, kerna adreso `0x1000` kaj troa longo
`4097`. Ĉiuj devas redoni respektive `-2` aŭ `-1`; se iu ajn sukcesus, la
uzanta programo printus `[ring3] negativaj: MALBONE` anstataŭ la valorojn.

### Dispetĉilo

La kerna dispetĉilo (`arkitekturo/x86_64/sistemvoko.rs`) legas
`rax`/`rdi`/`rsi`/`rdx`/`r10` el la interrompa kadro, plenumas la vokon kaj
skribas la rezulton reen en `kadro.rax`. La mesaĝa aranĝo venas el
`libmikerno::Mesago`, do la kerno kaj Ring 3 kopias la saman `#[repr(C)]`
strukturon; la kongruo estas fiksita per kompila-tempaj asertoj.
`kerno/sistemvoko.rs` efektivigas `SENDI`, `RICEVI`, `SKRIBI`, `FINI` kaj
`CEDI`: Ring 3-bufroj estas ne-nulaj, limigitaj kaj ene de la anoncitaj bilda
kaj stakaj regionoj (la limoj venas el `uzantspaco::bildaj_limoj()`);
`SYS_FINI` vokas `planisto::morti()` kaj neniam revenas.
Ring 0-memtesto pruvas la vojon per `int 0x80` en la unua fadeno (`SYS_CEDI`
tie vere cedas la procesoron).

---

## Konstruado kaj kontrolado

Kontrolo farita la **2026-11-12** per Rust 1.99.1 sur Windows:

```sh
cargo build --workspace --release
#   Finished `release` profile [optimized] target(s) in 5.20s

cargo clippy --workspace --release
#   Finished `release` profile ... (neniu averto)

cargo fmt --all -- --check
#   (neniu diferenco)
```

Ambaŭ praŝarĝaj vojoj estis praktike testitaj per QEMU 256 MiB:

| Vojo | Rezulto |
| --- | --- |
| PVH (`-kernel` + `-initrd`) | 7 regionoj, 255 MiB uzeblaj, sen framebuffer, bildo `0x100000000..0x100001082`, 8/8 kontroloj, `horizonto` kaj negativaj kazoj en Ring 3, 1000 tikoj, 4 fadenoj mortintaj |
| Limine/Multiboot2 (ISO) | framebuffer `1280x800, 32 bpp`, bildo `0x100000000..0x100001082`, 8/8 kontroloj, `horizonto` kaj negativaj kazoj en Ring 3, 1000 tikoj, 4 fadenoj mortintaj |

`just verify` ne estis rulita en ĉi tiu medio, ĉar `cargo kani` ne estas
instalita. La 12 pruvoj (6 pri la kadro-alokilo, 6 pri la mesaĝujo) estas
verkitaj kaj kompiliĝas nur sub `cfg(kani)`.

### Kion vi devus vidi

Ĉe sukcesa starto la seriala konzolo montras (mallongigite):

```text
[MiKerno] Saluton, mi estas MiKerno v0.2.1!
[sistemvoko] pordego 0x80 (DPL3) instalita
[kerno] memoro: 7 regionoj, 255 MiB uzeblaj
[memoro] liberigo (bone): 65146 kadroj antaŭ, 65146 post
[kerno] framebuffer 1280x800, 32 bpp        (nur ISO)
[tss] TR ŝargita; kernostako pinto 0x...
[temporizilo] kalibrado: ... kalkuloj/s, komenca ...
[uzanto] horizonto: bildo 0x100000000..0x100001082, enirpunkto 0x1000000d0
[uzanto] stako 0x100ff0000..0x101000000 (64 KiB), pinto 0x101000000
[uzanto] kontrolo (bone): skriba=true lega=true nurlega_skribo=false kerna=false nemapita=false translima=false eksterfenestra=false enirpunkta=true
[uzanto] memoro: marko 0x4d694b65726e6f21, legita 0x4d694b65726e6f21
[procezo] 4: Preta, kerna stako 0x0, kerna mapo kunhavata, uzanta fenestro privata
[kerno] APIC 0 kaj paĝado pretaj; temporizilo 100 Hz
[kerno] IPC: mesaĝo sendita al fadeno 2
[kerno] planisto ekas: 3 fadenoj, antaŭokupo ĉiu 10 ms
saluton el Ring 0 tra int 0x80
[sistemvoko] Ring 0-memtesto: skribo=0, nekonata=-2, nula=-1, troa=-1, cedo=0
...
[planisto] fadeno 3 finiĝis
[kerno] horizonto ekas en Ring 3: CS=0x23, SS=0x1B, RFLAGS=0x202, enirpunkto 0x1000000d0
horizonto: la unua procezo vekis en Ring 3.
[ring3] negativaj: nekonata=-2, nula=-1, kerna=-1, troa=-1
[planisto] fadeno 4 finiĝis
[kerno] horizonto finiĝis; planisto revenis
[kerno] planisto revenis al la ripoza fadeno
...
[kerno] fina stato: 4 mortintaj, 0 pretaj, 0 blokitaj; 1000 tikoj
```

---

## Testa scenaro

`kerno::ĉefa` kreas tri fadenojn kun statikaj 16 KiB-stakoj:

1. **Fadeno 2** tuj blokiĝas en `ricevi_mesagon(0)`; la praŝarĝo sendas al ĝi
   mesaĝon `tipo 1` («saluton el la praŝarĝo»), kiu vekas ĝin.
2. Ĉiuj tri fadenoj dek fojojn printas sian progreson kaj cedas la procesoron.
3. En la tria fojo **fadeno 1** sendas mesaĝon `tipo 2` al la blokita
   **fadeno 3**, kiu vekiĝas kaj printas ĝin.
4. Post la lasta fojo ĉiu fadeno finiĝas; la planisto revenas al la ripoza
   kunteksto.
5. La praŝarĝo aldonas la uzantan fadenon: `horizonto` ekas en Ring 3 per
   `iretq`, skribas sian saluton per `int 0x80`, kontrolas la negativajn
   kazojn (nekonata voko, nula kaj kerna montrilo, troa longo) kaj eliras per
   `fini(0)`. `SYS_FINI` mortigas la fadenon, do la planisto denove elektas la
   ripozan kuntekston.
6. La praŝarĝo observas 1000 tikojn (≈10 s ĉe 100 Hz), printas la finan staton
   kaj haltas per `hlt`.

---

## Strukturo de la dosieroj

```text
.
├── Cargo.toml                  # workspace (membroj, versio, lints)
├── justfile                    # build / run / iso / verify
├── .cargo/config.toml          # cela triopo, rust-lld, QEMU-runner
├── mikerno/
│   ├── Cargo.toml
│   ├── build.rs                # liveras boot/linker.ld nur al la kerno
│   ├── boot/
│   │   ├── linker.ld           # 1 MiB, .boot_bss aparte de .bss
│   │   └── limine.conf         # ISO-menuo (Multiboot2)
│   └── src/
│       ├── main.rs             # enirpunkto, `skribu!`, paniko
│       ├── kunhavigita.rs      # UnsafeCell + Sync (unuprocesora)
│       ├── arkitekturo/
│       │   ├── mod.rs          # platforma fasado
│       │   └── x86_64/
│       │       ├── mod.rs      # pravalorizi (seria, gdt, idt, nxe)
│       │       ├── eniro.rs    # 32→64-bita ŝelko, MB2-kapo, PVH-noto
│       │       ├── mb2.rs      # Multiboot2-analizo
│       │       ├── pvh.rs      # Xen-PVH-analizo
│       │       ├── gdt.rs      # GDT + TSS-priskribo
│       │       ├── idt.rs      # IDT kaj interrompaj ŝablonoj
│       │       ├── tss.rs      # RSP0, 16 KiB kerna stako
│       │       ├── apic.rs     # xAPIC MMIO
│       │       ├── temporizilo.rs # PIT-kalibrita 100 Hz
│       │       ├── pagxado.rs  # PML4/PDPT/PD/PT, CR3
│       │       ├── planado.rs  # kuntekstŝanĝo (asm)
│       │       ├── sistemvoko.rs # 0x80-dispetĉilo el la kadro
│       │       ├── procesoro.rs# hlt, cli/sti, CR2
│       │       ├── uzanta.rs   # `iretq` al Ring 3
│       │       └── seria.rs    # COM1
│       ├── memoro/
│       │   ├── mod.rs          # regionoj, mapo, EniraInformo, Modulo
│       │   ├── kadro.rs        # bitmapa alokilo + Kani-pruvoj
│       │   └── virtuala.rs     # adresoj, permesoj, uzanta fenestro
│       ├── interrompo/mod.rs   # kadro, traktiloj, RAII-gardo
│       ├── kerno/
│       │   ├── mod.rs          # ĉefa sekvenco, testaj fadenoj
│       │   ├── objekto.rs      # Fadeno, Kunteksto, stato
│       │   ├── procezo.rs      # Procezo, proceza stato, privata spaco
│       │   ├── planisto.rs    # rondiga planisto
│       │   ├── sistemvoko.rs   # SENDI/RICEVI/SKRIBI/FINI/CEDI
│       │   ├── uzantspaco.rs   # ELF64-ŝargo, stako, U-kontrolo
│       │   └── ipc.rs          # mesaĝoj, kapabloj + Kani-pruvoj
│       └── ekrano/
│           ├── mod.rs          # framebuffer-konzolo kaj elekto de streko
│           ├── fonto.rs        # Tiparo, TIPAROJ, glifo()
│           └── fonto/*.bin     # kvar rasterigitaj strekoj (DejaVu)
├── libmikerno/
│   └── src/{lib,kapablo,mesago,syscall}.rs
├── horizonto/
│   ├── build.rs, linker.ld     # 0x1_0000_0000 (4 GiB)
│   └── src/{main,biblioteko}.rs
├── tools/
│   ├── run-qemu.sh / .cmd
│   ├── font/generi.py          # DejaVu → bitmapaj strekoj
│   ├── limine/                 # praŝarĝilo (binaraj dosieroj)
│   └── xorriso/                # ISO-kreado sur Windows
├── third_party/dejavu/         # permesilo de la fonto
└── docs/
    ├── MEJLOSTONOJ.md          # mejloŝtonoj kaj pruvaro
    └── VORTARO.md              # Esperanta terminologio
```

---

## Limigoj kaj sekva laboro

- **Ring 3 unuprocesa**: ĉiu procezo jam havas privatan `PML4` kun propra
  fenestro, sed la planisto ankoraŭ ne ŝaltas `CR3`, do `horizonto` plenumas en
  la kerna spaco; `TSS.RSP0` estas komuna kaj mankas per-fadena kernostako.
- **Neniu aŭtomata liberigo**: `KadraAlokilo::liberigi` kaj `detruu` ekzistas,
  sed ĝis M12-fazoj 3–5 neniu proceza eliro vokas ilin; la alokilo do restas
  preskaŭ monotona dum la vivo de la sistemo.
- **Identeca mapo RWX**: oportunas por la prototipo, sed malobservas W^X;
  SMEP/SMAP, KASLR kaj APIC/IOMMU-izolado mankas.
- **Unuprocesora**: neniu SMP, afineco, prioritatoj aŭ dormantaj temporiziloj.
- **Fiksaj limoj**: 8 fadenoj, 16 mesaĝoj po fadeno, 64 bajtoj po mesaĝo,
  4 GiB fizika memoro.
- **IPC**: finpunkta id == fadena id; ne ekzistas apartaj objektoj, rajtorevoko,
  tempolimoj aŭ sendblokado.
- **Esceptoj haltigas** la sistemon; ne ekzistas rekaptebla paĝa eraro aŭ
  demando-paĝado.
- **Framebuffer**: nur 32 bpp, kvar fiksaj strekoj (8×16 … 32×64), 111 glifoj
  (95 ASCII + Esperanto + citiloj), malplenigo anstataŭ rulumo.
- **Pruvoj**: 12 Kani-pruvoj spegulas la bitan logikon sur 64 kadroj; la produkta
  kodo mem ne estas pruvita. `just verify` postulas apartan Kani-instalon.
- **Neniu CI nek testoharness**: la kontrolo estas `build` + `clippy` + `fmt` +
  QEMU-smoka testo.
- **Versio hardkodita** en `kerno/mod.rs`, `limine.conf` kaj `Cargo.toml`.
- **ISO povas malaktualiĝi** (vidita dum la kontrolo); ĉiam `just iso` post
  kerna ŝanĝo.

La plena, numerita vojkarto kun akceptkriterioj estas en
[docs/MEJLOSTONOJ.md](docs/MEJLOSTONOJ.md).

---

## Kontribuado

- La kodo, komentoj, identigiloj kaj dokumentaro estas en **Esperanto**.
- Uzu `rustfmt` kaj `cargo clippy --workspace --release` antaŭ ol sendi ŝanĝojn.
- Ne aldonu eksterajn crate-ojn sen klara kialo; la kerno celas resti sen
  dependaĵoj.
- Ĉiu `unsafe` bloko dokumentu sian sekurecan kontrakton.
- Por novaj arĥitekturaj malantaŭoj, respektu la fasadon
  `arkitekturo/mod.rs`; la nukleo ne rajtas uzi `x86_64`-specifan kodon rekte.
- Pruvu novan logikon per Kani, se ĝi estas sendependa de aparataro.

---

## Licenco kaj triaj komponantoj

La projekto ankoraŭ ne havas `LICENSE`-dosieron; aldonu ĝin antaŭ publika
disdono.

Enigitaj triaj komponantoj:

| Komponanto | Loko | Noto |
| --- | --- | --- |
| Limine | `tools/limine/` | Praŝarĝilo por la ISO (Multiboot2) |
| xorriso | `tools/xorriso/` | Nur por krei la ISO-on sur Windows |
| DejaVu Sans Mono (bitmapaj strekoj) | `mikerno/src/ekrano/fonto/` | Rasterigita el DejaVu Sans Mono; permesilo Bitstream Vera + Arev en `third_party/dejavu/LICENSE.txt` |
| `tools/font/generi.py` | `tools/font/` | Generilo por la kvar strekoj; postulas Python 3 kaj Pillow |

La bitaj tabeloj estas derivaĵoj de DejaVu Sans Mono. La permesilo postulas,
ke la kopirajtaj kaj permesaj tekstoj estu kunigitaj kun la fontprogramaro —
tial `third_party/dejavu/LICENSE.txt` estas parto de la deponejo. La nomoj
"Bitstream" kaj "Vera" ne estas uzataj por la derivaĵoj. Kontrolu la
koncernajn permesilojn antaŭ ol redistribui la ISO-on.

Fontdosiero uzita por generi la tabelojn: `DejaVuSansMono.ttf`, SHA-256
`602ec86b8948cfcd956482fe64f94c36c867770149ef2f791d4613f443bcecb3`.

---

*Ĉi tiu dokumento estas parto de la projekto kaj estas verkita en Esperanto,
same kiel la kodo.*
