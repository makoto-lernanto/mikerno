# MiKerno

> «La estonteco nun.»

> **Pri la aŭtoreco.** `makoto-lernanto` estas la arkitekto de ĉi tiu projekto:
> li difinas la celojn, la arĥitekturon kaj la inĝenieran pensmanieron. La tuta
> efektiva kodo — same kiel la dokumentaro kaj la testoj — estis verkita de
> **Qoder**, la AI-programada asistanto. La homa kontribuo estas la ideo, ne la
> klavoj.

MiKerno estas eta, memstara x86_64-kerno verkita tute en Rust (`#![no_std]`,
`#![no_main]`), sen eksteraj Rust-dependaĵoj kaj sen C-kodo en la kerno mem. La
kodo, la protokoloj kaj la dokumentaro estas en Esperanto.

La celo de la projekto estas fari legeblan, hobleblan kaj pruveblan
demonstration de kompleta unuprocesora kerno: de la unua instrukcio en 32-bita
reĝimo ĝis antaŭokupa planisto, mesaĝa IPC kaj uzanta procezo en Ring 3.

- **Versio:** 0.4.3 (workspace)
- **Arkitekturo:** x86_64, unuprocesora
- **Cela triopo:** `x86_64-unknown-none`
- **Eldono de Rust:** 2024, `rust-version = 1.100`
- **Statuso:** funkcianta prototipo — vidu [Mejloŝtonoj](docs/MEJLOSTONOJ.md)
- **GUI:** deviga kadro FluoU — vidu [docs/FLUOU.md](docs/FLUOU.md)

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
| Preciza liberigo de privataj `PDPT`-oj (inkl. `PML4[0]`) sen tuŝi kunhavatajn kernajn mapojn | Funkcias | `pagxado.rs`, `kerno/procezo.rs` |
| Per-procesa adresa spaco: privata PML4, kunhavataj kernaj mapoj (`krei_kunhavantan`) | Funkcias | `kerno/procezo.rs`, `pagxado.rs` |
| Proceza detruo ĉe `SYS_FINI` kaj ĉe mortiga Ring 3-escepto (`#PF`, `#GP`) | Funkcias | `kerno/planisto.rs`, `interrompo/mod.rs` |
| `CR3`-ŝalto dum kuntekstŝanĝo kaj per-procesa `TSS.RSP0` (16 KiB kerna stako) | Funkcias | `kerno/planisto.rs`, `kerno/procezo.rs`, `tss.rs` |
| Strikta uzanta mapado (`mapi_uzantan`): fenestra limkontrolo kaj devigata bito `U` | Funkcias | `memoro/virtuala.rs`, `kerno/uzantspaco.rs` |
| 4-nivela paĝado (PML4 → PDPT → PD → PT), NX, PCD | Funkcias | `arkitekturo/x86_64/pagxado.rs` |
| SMEP: Ring 0 ne povas plenumi uzantajn foliojn (`CR4.SMEP`), kun CPUID-provo kaj gracia malŝalto | Funkcias | `arkitekturo/x86_64/firmigo.rs`, `procesoro.rs` |
| SMAP: Ring 0 legas/skribas uzantajn foliojn nur en `STAC`/`CLAC`-fenestroj (`CR4.SMAP`), kun RAII-gardo | Funkcias | `arkitekturo/x86_64/firmigo.rs`, `kerno/sistemvoko.rs` |
| GDT, 256-vektora IDT, escepta raportado | Funkcias | `gdt.rs`, `idt.rs`, `interrompo/mod.rs` |
| Uzantaj DPL3-segmentoj kaj `int 0x80`-pordego (interrompa, `0xEE`) | Funkcias | `gdt.rs`, `idt.rs`, `kerno/sistemvoko.rs` |
| Naŭ sistemvokoj per `int 0x80`: `SKRIBI`/`FINI`/`CEDI`/`SENDI`/`RICEVI`/`KREI_FINPUNKTON`/`DONI`/`REVOKI`/`FERMI`, unueca `Mesago`, montrila kontrolo | Funkcias (Ring 0 kaj 3) | `kerno/sistemvoko.rs`, `arkitekturo/x86_64/sistemvoko.rs` |
| ABI 2: `Tenilo` kun du-nivela generacia kontrolo, kapablotabelo, `KREI_FINPUNKTON`/`DONI`/`REVOKI`/`FERMI`, revokaj vekoj `-32`, purigo ĉe proceza detruo | Funkcias (ekde 0.4.3) | `libmikerno/`, `kerno/ipc.rs` |
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
| Ring 3-escepta kapto: `#PF`/`#GP` mortigas la procezon kaj la sistemo daŭras | Funkcias | `interrompo/mod.rs` |
| Samtempaj procezoj `4`/`5` kun sama sekreta adreso sed malsamaj kadroj | Funkcias | `kerno/mod.rs`, `kerno/uzantspaco.rs` |
| Izolado de adresa spaco: spiono `#PF` ĉe nur-viktima paĝo, viktimo pluvivas | Funkcias | `horizonto/`, `interrompo/mod.rs` |
| 21 formalaj pruvoj per Kani | Verkitaj | `kadro.rs`, `ipc.rs` |

> `horizonto` estas liverata kiel praŝarĝa modulo, muntata super 4 GiB kun la
> bito `U` tra ĉiuj paĝtabelaj niveloj, kaj ekzekutas en Ring 3 per `iretq`.
> La praŝarĝo preparas du procezojn antaŭ la unua planista voko, do ili rulas
> samtempe: procezo `4` (viktimo) kaj procezo `5` (spiono). Ambaŭ ricevas
> sekretan paĝon ĉe la sama virtuala adreso `0x2_0000_0000`, sed malsamajn
> kadrojn; ĉiu legas sian propran identigilon. Procezo `5` legas duan paĝon
> `0x2_0000_1000`, kiu ekzistas nur en la spaco de procezo `4`, kaj ricevas
> `#PF` (`erarkodo=0x4`): la kerno mortigas ĝin, `detruu` liberigas ĝian tutan
> adresan spacon — inkl. la privatan `PDPT`-on sub `PML4[0]` — kaj la planisto
> daŭrigas. Procezo `4` pluvivas kaj finiĝas per `SYS_FINI`. La bildo estas
> ŝargata nur en la privata adresa spaco; kiam la planisto elektas ĝian fadenon,
> ĝi ŝaltas `CR3` al tiu spaco kaj `TSS.RSP0` al la propra 16 KiB-a kerna stako.
> La libera kadraro egalas antaŭ kaj post la scenaro.

---

## Rapida starto

### Windows: antaŭkondiĉo — WSL2

La projekto estas konstruata kaj rulata nur ene de WSL2 (Ubuntu). Windows servas
nur kiel gastiganto: `cmd`, PowerShell, Git Bash kaj la Windows-eldonoj de QEMU
kaj xorriso ne estas uzataj de la konstruo.

En PowerShell kun administraj rajtoj, unufoje agordu WSL2 kaj Ubuntu:

```powershell
wsl --install -d Ubuntu
```

Post restarto kaj la unua agordo de Ubuntu, ene de la WSL-terminalo:

```sh
sudo apt update
sudo apt install -y build-essential qemu-system-x86 xorriso
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup target add x86_64-unknown-none
cargo install just          # opcia
```

- **Grafikaĵo.** Por vidi la QEMU-fenestron necesas WSLg (Windows 11, aŭ
  ĝisdatigita Windows 10). Sen WSLg uzu `QEMU_DISPLAY=none` por senhoma rulado.
- **Loko de la deponejo.** Klonu en la Linuksan dosiersistemon (ekz.
  `~/mikerno`), ne en `/mnt/c/…`: la 9p-muntado estas multe pli malrapida, kaj
  Kani/CBMC povas elĉerpi la memoron dum la konstruo.
- **Kani** (nur por `just verify`) instaliĝas aparte per
  `cargo install --locked kani-verifier` kaj `cargo kani setup`.

### Postuloj

| Ilo | Versio / noto |
| --- | --- |
| Rust | 1.100 aŭ pli nova (eldono 2024); projekto kontrolita per 1.100.0 |
| Cela triopo | `x86_64-unknown-none` |
| QEMU | `qemu-system-x86_64` (`sudo apt install qemu-system-x86`) |
| xorriso | Nur por `just iso` (`sudo apt install xorriso`) |
| `just` | Opcia, sed rekomendata por la difinitaj taskoj |
| bash | Lanĉilo de `tools/run-qemu.sh` |
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

`.cargo/config.toml` fiksas la celon `x86_64-unknown-none` kaj lanĉas QEMU per
`bash tools/run-qemu.sh`. Vi povas voki la lanĉilon ankaŭ rekte:

```sh
tools/run-qemu.sh release
tools/run-qemu.sh iso
QEMU_DISPLAY=none tools/run-qemu.sh iso   # senkapa ISO-starto
```

En la QEMU-konzolo: `Ctrl+A x` tuj finas QEMU, `Ctrl+A c` malfermas la
monitoron (tie `quit`). La `-serial mon:stdio` muntado estas necesa ĉar `Ctrl+C`
alie sendiĝas al la guest kiel bajto `0x03`. En WSL2 sen WSLg uzu
`QEMU_DISPLAY=none` por la ISO-vojo.

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
> ligita super 4 GiB. La kerno estas ligita kun `relocation-model=pic` (ET_DYN)
> por KASLR; `horizonto` ligas kun `-no-pie` kaj restas ET_EXEC kun statikaj
> relokoj. Neniu dinamika ligilo aŭ ekstera dependaĵo aperas.

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
  · nuligas .bss, starigas stakon, aplikas la RELATIVE-relokojn
  · vokas kerna_eniro
        │
        ▼
kerna_eniro                          main.rs
  · hazardigo (KASLR): elektas 2 MiB-aliniiĝintan bazon, kopias la bildon
    tien, relokas la kopion kaj saltas en ĝin (`kerna_eniro_kopio`)
        │
        ▼
kerna_eniro_kopio                    main.rs
  · pravalorizi: COM1, GDT, IDT, EFER.NXE
  · analizi_eniron: Multiboot2 aŭ Xen-PVH
        │
        ▼
kerno::ĉefa                          kerno/mod.rs
  · kadro-alokilo ← memora mapo + rezervoj (inkl. praŝarĝaj moduloj)
  · provo de liberigo: proviza PML4 → mapo → `detruu`, kadroj reakiritaj
  · nova PML4 (4 KiB paĝoj), identeca mapo, framebuffer (PCD)
  · modulaj regionoj identece mapitaj, CR3 ŝalto, konzolo, TSS/RSP0
  · APIC + temporizilo (100 Hz)
  · reuzoprovo: 10× krei/munti/detrui procezan spacon, liberaj kadroj egalaj
  · IPC-testo, 3 testaj fadenoj (procezo 0), planisto
  · procezoj 4 (`REĜIMO_SEKRETO`) kaj 5 (`REĜIMO_SPIONO`) preparitaj kune,
    ambaŭ Pretaj, interrompoj malŝaltitaj; unu `planisto::lanĉi()`
  · sama virtuala sekreta paĝo `0x2_0000_0000`, malsamaj kadroj kaj id
  · procezo 5 legas la duan, nur-viktiman paĝon `0x2_0000_1000` → `#PF`
    (`erarkodo=0x4`, `U=1`, `P=0`) → la procezo mortigata, la planisto daŭras
  · procezo 4 pluvivas kaj eliras per `SYS_FINI`; `detrui` liberigas ambaŭ
  · `[memoro] izolado`: liberaj kadroj egalaj antaŭ kaj post
  · 10 s da observado, halto
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
la 64-bita kodo restu atingebla tuj post `CR0.PG`. Ĉar la bildo estas ET_DYN,
la ŝelko trovas sian bazon per `call`/`pop` kaj uzas nur nombrajn deŝovojn ene
de `.boot_text`: GDT ĉe `0x200`, GDT-montrilo ĉe `0x220`, Multiboot2-kapo ĉe
`0x280`, la unuaj paĝtabeloj ĉe `0x1000..0x7000` kaj la 64-bita eniro ĉe
`0x7000`. La paĝtabeloj restas en `.boot_text` (nurlegaj post la unua
`CR3`-ŝalto), dum la 16 KiB-a praŝarĝa stako translokiĝis en `.bss`, kiun la
nova spaco mapas skribebla.

Post la unua Rust-voko `hazardigo` (`hazardigo.rs`) legas la memormapon,
elektas hazardan 2 MiB-aliniiĝintan bazon en uzebla RAM (aŭ `RDRAND`, aŭ
`RDTSC` miksita per SplitMix64), kopias la bildon tien, aplikas la
`R_X86_64_RELATIVE`-relokojn al la kopio kaj saltas en ĝin per
`kerna_salto_al_kopio`. Se la kondiĉoj mankas, la bildo restas ĉe la liga bazo
kaj la kerno raportas `KASLR malŝaltita`.

La liga skripto metas la kernon je 1 MiB (`mikerno/boot/linker.ld`) kaj
eksportas `__kerna_komenco`, `__kerna_fino`, `__bss_komenco`, `__bss_fino`,
`__rela_komenco` kaj `__rela_fino`. Ĉiuj limoj kuŝas ene de sekcio, do ili
ricevas RELATIVE-relokon kaj sekvas la hazardan bazadreson; `__kerna_fino`
preterpasas la plenan bildon inkluzive de `.bss`.

---

## Memoro

### Kadro-alokilo

`memoro/kadro.rs` tenas bitaron por la unuaj 4 GiB: 1 048 576 kadroj da 4 KiB,
t.e. 128 KiB da bitaro en `.bss`. La alokilo:

1. markas ĉion okupata;
2. malmarkas la uzeblajn regionojn de la praŝarĝila memora mapo;
3. denove rezervas la malaltan 1 MiB (BIOS/legacy), la kernan bildon
   (`__kerna_komenco..__kerna_fino`), la devenan praŝarĝan bildon ĉe
   `origino` (ĝiaj kadroj tenas la aktivajn praŝarĝajn paĝtabelojn ĝis la unua
   `CR3`-ŝalto) kaj la regionojn el la praŝarĝilo (Multiboot2-informo,
   PVH-memmapo, RSDP).

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

- la malaltan 1 MiB kaj ĉiujn uzeblajn regionojn identece kiel RW+NX,
- la kernan bildon laŭ sekciaj limoj el `boot/linker.ld`: `.text` (`RX`),
  `.rodata` (`R`+`NX`) kaj `.data`/`.bss` (`RW`+`NX`),
- la framebuffer-on (`SKRIBA_SENKAŜE`: skribebla, neplenumebla, `PCD`),
- la APIC-MMIO-on (`SKRIBA`).

`EFER.NXE` kaj `CR0.WP` estas ŝaltitaj antaŭ ol oni uzas la biton 63 aŭ
nurlegajn mapojn: `CR0.WP` devigas ankaŭ Ring 0 respekti la biton `W`, do
nurlega paĝo estas vera barilo, ne konsilo. `mapigi` rifuzas nealiniiĝintajn
adresojn kaj jam mapitajn foliojn (`Konflikto`); `traduki` komprenas ankaŭ 2 MiB
grandegajn paĝojn. Neniu paĝo estas samtempe skribebla kaj plenumebla. La
praŝarĝa modulo estas mapita nurlega antaŭ la identeca mapado, do ĝiaj paĝoj
restas `R`+`NX` anstataŭ fariĝi skribebla RAM. `permesoj(adreso)` legas la
efektivajn bitojn el la folio, kaj la starta memtesto printas
`[memoro] W^X (bone): .text RX, .rodata R NX, .data/.bss RW NX, modulo R NX`.

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
  `erarkodo` kaj — por vektoro 14 — `CR2`. Escepto el Ring 0 haltigas la
  sistemon. Escepto `#PF` (14) aŭ `#GP` (13) el Ring 3 apartenas al la uzanta
  procezo: la kerno printas `[eraro] procezo N …` kun `rip`/`erarkodo`/`CR2`,
  vokas `planisto::morti()` (kiu detruas la procezan spacon) kaj daŭrigas kun
  la sekva fadeno anstataŭ `halti()`.
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
- antaŭ ĉiu `switch` la planisto aktivigas la celan procezon: se la fizika
  radiko diferencas de la aktiva `CR3`, ĝi ŝaltas `CR3` al la proceza `PML4`;
  samtempe ĝi agordas `TSS.RSP0` al la proceza 16 KiB-a kerna stako (aŭ al la
  defaŭlta kerna stako por procezo `0`);
- la uzantaj fadenoj (id 4 kaj 5) estas planitaj post la testaj: iliaj
  enirpunktoj transiras al Ring 3 per `iretq` kun la reĝimo en `RDI` kaj la
  proceza identigilo en `RSI`. La du procezoj estas pretaj samtempe antaŭ la
  unua planista voko, do ili interplektiĝas. Fadeno 5 legas la duan sekretan
  paĝon, kiu ekzistas nur en la spaco de fadeno 4, kaŭzas `#PF` kaj estas
  mortigata; fadeno 4 pluvivas kaj eliras per `SYS_FINI`. Ambaŭokaze `morti`
  detruas la procezan spacon, do la planisto elektas la ripozan kuntekston,
  reŝaltas la kernan `CR3` kaj la observado daŭras.

`switch` revenas nur kiam la malnova kunteksto estas planita denove. La ripoza
kunteksto konservas sian interrompan kadron sur la stako, do `planumi` povas
reveni tra la dispetĉo kaj `iretq` kiam la ripoza fadeno rekaptas la procesoron.
La `CR3`-ŝalto okazas antaŭ la registra konservo: la fadenaj kontekstoj, la
planisto kaj la kernaj stakoj vivas en la kerna bildo, kiun ĉiu proceza spaco
kunhavas.

---

## IPC kaj kapabloj

`kerno/ipc.rs` difinas minimuman mesaĝan IPC-on:

- `FinaPunktoObjekto` posedas la mesaĝvicon kaj la atendoliston; finpunktaj
  tabeloj portas `stato`, `generacio`, `referencoj` kaj identigilon;
- `Tenilo` (u32): 8 bitoj da fenda indekso + 24 bitoj da fenda generacio;
  `0` estas ĉiam malvalida, la generacio komenciĝas ĉe `1` kaj la fendo
  emeritiĝas ĉe `0xFFFFFF`;
- permesoj `SENDI`, `RICEVI`, `ĈIUJ`, `REVOKI`, `PLENAJ` (bitoj `0b001`…
  `0b111`); `DONI` nur malpliiĝigas ilin;
- ĉiu procezo havas `KAPABLOJ[procezo][fendo]` (linio `0` rezerviĝas al la
  kerno kaj restas malplena, C8);
- mesaĝo kun `tipo` (u64) kaj ĝis **64 bajtoj** da datumaro; la datumaro estas
  kopiata, do la sendanto rajtas tuj reuzi sian bufron;
- ĉiu finpunkto havas vicon por **16 mesaĝoj** (`Mesagujo`), FIFO inter la sama
  fonto; la unua blokita ricevanto estas vekata de la sendinto;
- `SENDI`/`RICEVI` prenas veran `Tenilo`-n en `rdi`; la dua nivelo de la
  validigo kontrolas, ke la finpunkta objekto estas aktiva kaj havas la
  generacion, kiun la kapablofendo memoras — malnova tenilo ne reviviĝas;
- `REVOKI` (8) estas **tutfinpunkta** (C14): ĝi postulas la biton `REVOKI`,
  forigas la finpunktan objekton kaj ĉiujn ĝiajn kapablofendojn en ĉiu procezo;
  blokita ricevanto estas vekata per `-32`; `FERMI` nur fermas la lokan
  tenilon;
- malplena permeso aŭ manko de la koncerna bito redonas `-1`, nevalida aŭ
  eksdata tenilo `-9`; la montrila kontrolo venas antaŭ la tenila (C11);
- la finpunkta generacio estas pliigata ĉe revoko kaj ĉe proceza detruo, do
  neniu malnova tenilo reviviĝas;
- `registri_fadenon` instalas la transiran `BOOTSTRAP_TENILO` (`0x100`: fendo
  `0`, unua generacio) por ĉiu procezo, do `horizonto` povas uzi sian
  defaŭltan finpunkton sen antaŭa `KREI_FINPUNKTON`;
- proceza detruo (ankaŭ la antaŭaj erarvojoj de `prepari_uzantan_procezon`)
  vokas `ipc::purigi_procezon`: ĝi malpliigas la referencojn de fremdaj
  finpunktoj, liberigas ĉiujn kapablofendojn de la procezo kaj revokas la
  finpunktojn, kiujn ĝi posedis;
- la kernaj fadenoj (procezo `0`) ne havas kapablotabelon; ili uzas la
  internajn `sendi_mesagon`/`ricevi_mesagon` (C8).

Ĉiuj operacioj tenas interrompojn malŝaltitaj per `InterrompaGardo`, kiu
restarigas la antaŭan `RFLAGS.IF`-staton ĉe `Drop`.

Nuna limigo: unu finpunkto memoras nur unu blokitan ricevanton; tempolimoj,
sendblokado kaj `fork`/`exec`-distribuo de kapabloj venas en la sekvaj fazoj.

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
- `FinaPunkto`, `Kapablo`, `Permesoj`, `Tenilo`, `BOOTSTRAP_TENILO`,
- sistemvokaj numeroj: `SYS_SENDI = 1`, `SYS_RICEVI = 2`, `SYS_SKRIBI = 3`,
  `SYS_FINI = 4`, `SYS_CEDI = 5`, `SYS_KREI_FINPUNKTON = 6`, `SYS_DONI = 7`,
  `SYS_REVOKI = 8`, `SYS_FERMI = 11`; la tempaj vokoj 9–10 ankoraŭ respondas
  `-2`,
- `Tenilo` — 32-bita kapabla tenilo (8 bitoj da indekso + 24 bitoj da
  generacio, generacio ekde `1`); la novaj erar-kodoj estas `-9`, `-11`, `-28`,
  `-32`, `-90` kaj `-110`,
- registra ABI: `rax` = numero, `rdi`/`rsi`/`rdx`/`r10` = argumentoj, `rax` =
  rezulto; la voko estas `int 0x80`;
- starta ABI: `horizonto_start(reĝimo: u64, identigilo: u64)` — la kerno
  transdonas la reĝimon en `RDI` (`REĜIMO_SEKRETO = 2`, `REĜIMO_SPIONO = 3`) kaj
  la procezan identigilon en `RSI`.

### Uzanta fenestro

La uzanta virtuala spaco estas rezervita supre de la kerna identeca mapo:

| Limo | Valoro |
| --- | --- |
| `UZANTA_BAZO` | `0x1_0000_0000` (4 GiB) |
| `UZANTA_FINO` | `0x80_0000_0000` (512 GiB) |
| `UZANTA_STAKO_SUPRO` | `UZANTA_BAZO + 16 MiB` = `0x1_0100_0000` |
| `UZANTA_STAKO_GRANDE` | 64 KiB (`0x100ff0000..0x101000000`) |
| `SEKRETA_ADRESO` | `0x2_0000_0000` (sama en ĉiu procezo, malsama kadro) |
| `SEKRETA_DUA_ADRESO` | `0x2_0000_1000` (nur en la spaco de la viktimo) |

`PaĝaPermesoj` havas la biton `uzanta`; la mapado metas `U` en ĉiun nivelon
(PML4 → PDPT → PD → PT), do `kontroli_uzantan` povas pruvi per unu promenado,
ke la tuta vojo estas alirebla el Ring 3. La folia nivelo ricevas `W` nur por
skribaj regionoj kaj `NX` por datumoj kaj stako; skribebla paĝo neniam estas
plenumebla (`W^X`). Sistemvokoj kun Ring 3-montrilo kontrolas ne nur la
anoncitan bildan/stakan regionon, sed ankaŭ ĉiun folian permeson: `SYS_SKRIBI`
kaj `SYS_SENDI` postulas legeblan folion, `SYS_RICEVI` skribeblan. Rifuzo estas
raportita per `[eraro] procezo N: montrilo … ne havas la postulatan folian permeson`.

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
   paĝo, kiu postulus samtempe `W` kaj `X`, estas rifuzata (`NevalidaWX`);
4. mapas la 64 KiB-an stakon ĉe `UZANTA_STAKO_SUPRO`.

Post tio `pravalorizi()` redonas la bildajn limojn, la enirpunkton kaj la
stakon, kaj rulas memkontrolon: skribo/lego tra la uzanta fenestro el Ring 0
devas sukcesi, dum nurlega skribo, kerna adreso, nemapita adreso, translima
intervalo kaj adreso ekster la fenestro devas esti rifuzitaj laŭ la bito `U`.
La kontrolo ankaŭ legas la foliajn permesojn de la enirpunkto (`RX`) kaj de la
stako (`RW`+`NX`). Ĉar la ŝargo okazas en la proceza spaco, la marka
skribo/lego provizore ŝaltas tiun `CR3` (kun interrompoj malŝaltitaj) kaj poste
restarigas la kernan radikon.

`horizonto` estas liverata de ambaŭ praŝarĝaj vojoj — Multiboot2 tipo 3 el
`limine.conf`, aŭ la PVH-modlisto el `-initrd` (``just run`` aldonas ĝin
aŭtomate).

### Proceza adresa spaco

`kerno/procezo.rs` donas la abstrakton `Procezo`: identigilo, privata `Spaco`,
propra 16 KiB-a kerna stako (`kerna_stako_pinto`), stato (`Preta`, `Kuranta`,
`Mortinta`), la bildaj limoj kaj enirpunkto de la propra bildo, kaj la adreso de
la sekreta paĝo. La nukleo estas procezo `0` sen ero en la tabelo; la tabelo
tenas ĝis 8 procezojn. La stakoj estas statikaj eroj de la proceza
fenestro en `.bss`; ilia bazo estas 16-bajte aliniiĝinta kaj la grando estas
multoblo de 16, do ankaŭ la pinto taŭgas por la x86-64-ABI kaj por la
interrompa kadro. `Fadeno` portas `procezo`-idon, kaj la planisto spegulas
`Kuranta`/`Preta`/`Mortinta` en la proceza stato.

Ĉe starto la nukleo unue registras sian propran radikon
(`agordi_kernan_spacon`), poste kreas la procezojn `4` kaj `5` per
`krei_kunhavantan` kaj **ŝargas la bildon rekte en ĉiun spacon**. Ĉiu procezo
memoras siajn bildajn limojn, enirpunkton kaj sekretan adreson. La konzolo
konfirmas per:

```text
[procezo] 4: Preta, kerna stako 0x138d90, PML4 0x207000, kerna mapo kunhavata, bildo nur en la proceza spaco jes, sekreta paĝo 0x200000000
```

La kontrolo legas `traduki(0x1000)` en la proceza spaco (kerna mapo ĉeestas)
kaj `traduki(bilda_komenco)` en la kerna spaco (la bildo mankas tie, do ĝi
ekzistas nur en la propra spaco). Dum planado `aktivigi(id)` komparas la aktivan
`CR3` kun la proceza `PML4` kaj ŝaltas ĝin nur se ili diferencas; por procezo
`0` ĝi revenas al la kerna radiko. Samtempe `TSS.RSP0` montras al la kerna
stako de la elektita procezo.

`krei_kunhavantan` registras en `X86AdresaSpaco` kiujn `PML4`-indeksojn ĝi
privatigis (`privataj_pdptoj`). La indekso `0` ĉiam estas privata: ĝi portas
kaj la identecan mapon de la unuaj 4 GiB kaj la uzantan fenestron. `detruu`
tial unue malmapas la fenestron (folioj, poste malplenaj `PT`/`PD`), poste
liberigas **nur la registritajn privatajn `PDPT`-ojn** — neniam la kunhavatajn
kernajn subtableojn — kaj laste la radikan `PML4`-on. Se post la fenestra
liberigo restus `U=1`-ero en privata `PDPT`, la spaco estus difektita kaj
`detruu` rifuzas liberigi ĝin (`Konflikto`) anstataŭ riski kernan mapon.
`KadraAlokilo::liberigi` nuligas la kadron kaj traktas duoblan liberigon kiel
senefikan.

### Transiro al Ring 3

La praŝarĝo **preparas ambaŭ `horizonto`-procezojn antaŭ la unua planista
voko**: procezo `4` (`REĜIMO_SEKRETO`, la viktimo) kaj procezo `5`
(`REĜIMO_SPIONO`, la spiono) ricevas propran `PML4`, propran 16 KiB-an kernan
stakon, bildon, stakon kaj sekretan paĝon, kaj ambaŭ registriĝas kiel `Preta`
dum interrompoj restas malŝaltitaj. Nur poste `planisto::lanĉi()` ekas unufoje,
do la du procezoj vere interplektiĝas anstataŭ ruli sinsekve.

Ĉiu fadeno unue plenumas sur la kerna stako de sia procezo (la sama pinto, kiun
`TSS.RSP0` uzos poste). `arkitekturo/x86_64/uzanta.rs` konstruas la
`iretq`-kadron sur tiu stako:

```text
push SS=0x1B
push RSP=UZANTA_STAKO_SUPRO - 8
push RFLAGS=0x202        (IF=1 + rezervita bito 1)
push CS=0x23
push RIP=enirpunkto
RDI = reĝimo             (2 = SEKRETO, 3 = SPIONO)
RSI = proceza identigilo (4 aŭ 5)
iretq
```

La stakpinto estas malaltigita je 8 por ke la x86-64-ABI vidu `rsp % 16 == 8`
ĉe la funkcia eniro; `RDI` kaj `RSI` transdonas la reĝimon kaj la identigilon al
`extern "C" fn horizonto_start(reĝimo: u64, identigilo: u64)` (`REĜIMO_SEKRETO`
kaj `REĜIMO_SPIONO` el `libmikerno::starto`). La `int 0x80`-pordego havas DPL3,
do Ring 3 rajtas voki `SKRIBI`, `CEDI` kaj `FINI`; la CPU ŝaltas al `TSS.RSP0`
por ĉiu reveno al Ring 0.

### Izolado de la sekreta paĝo

`uzantspaco::mapi_sekreton` alokas freŝan, nuligitan kadron, mapas ĝin kiel
`U=1`-folio ĉe la **sama virtuala adreso** `SEKRETA_ADRESO`
(`0x2_0000_0000`) en la spaco de ĉiu procezo, kaj skribas la magian numeron
`0x4D69_4B65_726E_6F21` kaj la procezan identigilon per la identeca mapo. Ĉar la
virtuala adreso samas sed la fizikaj kadroj malsamas, ĉiu procezo legas sian
propran valoron: `4` legas `id 4`, `5` legas `id 5`.

La viktimo krome mapas **duan** paĝon ĉe `SEKRETA_DUA_ADRESO`
(`0x2_0000_1000`); la spiono ne havas tiun mapadon. Kiam la spiono legas
`0x2_0000_1000`, la paĝtabela promenado malsukcesas en la privata `PDPT` de la
spiono kaj la CPU liveras `#PF` kun `erarkodo=0x4` (`U=1`, `P=0`). La kerno
printas `[eraro] procezo 5 …`, mortigas la procezon, `detrui` liberigas ĝian
tutan spacon, kaj la planisto revenas al la ripoza kunteksto — neniun `halti()`.
Procezo `4` pluvivas, printas plurajn rondojn kaj finiĝas per `fini(0)`.

La dua paĝo estas la plej forta pruvo: ĝi montras, ke proceso `5` ne povas legi
adreson, kiu ekzistas nur en la adresa spaco de proceso `4`, eĉ kiam ambaŭ
uzas la saman virtualan bazon por la unua sekreta paĝo.

Antaŭ la eliro la unua procezo ankaŭ kontrolas la negativajn kontraktojn el
Ring 3: vojo `0x7FFF` (nekonata), nula montrilo, kerna adreso `0x1000`, troa
longo `4097` kaj skriba `SYS_RICEVI` en la nurlega enirpaĝo. Ĉiuj devas redoni
respektive `-2` aŭ `-1`; se iu ajn sukcessus, la uzanta programo printus
`[ring3] negativaj: MALBONE` anstataŭ la valorojn. La provo skribi en la
plenumeblan enirpaĝon ankaŭ lasas la kernan protokolon
`… ne havas la postulatan folian permeson`. Ekde Fazo 3 la sama procezo ankaŭ
sendas per du malvalidaj teniloj (`6` kun ne-kongrua generacio kaj `0`), kaj
ambaŭ devas redoni `-9`. Ekde Fazo 4 ĝi krome revokas finpunkton: `REVOKI` kun
ne-nula `rsi` redonas `-1`, finpunkto sen la bito `REVOKI` redonas `-1`, kaj
post revoko la sama tenilo redonas `-9` por `SENDI`, `FERMI` kaj dua `REVOKI`.

### Dispetĉilo

La kerna dispetĉilo (`arkitekturo/x86_64/sistemvoko.rs`) legas
`rax`/`rdi`/`rsi`/`rdx`/`r10` el la interrompa kadro, plenumas la vokon kaj
skribas la rezulton reen en `kadro.rax`. La mesaĝa aranĝo venas el
`libmikerno::Mesago`, do la kerno kaj Ring 3 kopias la saman `#[repr(C)]`
strukturon; la kongruo estas fiksita per kompila-tempaj asertoj.
`kerno/sistemvoko.rs` efektivigas `SENDI`, `RICEVI`, `SKRIBI`, `FINI`, `CEDI`,
`KREI_FINPUNKTON`, `DONI`, `REVOKI` kaj `FERMI`: Ring 3-bufroj estas ne-nulaj,
limigitaj kaj ene de la anoncitaj bilda kaj stakaj regionoj (la limoj venas el
la nuna procezo mem, `Procezo::bildaj_limoj()`); la tenilaj vokoj validigas la
kapablon en du niveloj (fendo + finpunkta generacio); `SYS_FINI` vokas
`planisto::morti()` kaj neniam revenas.
Ring 0-memtesto pruvas la vojon per `int 0x80` en la unua fadeno (`SYS_CEDI`
tie vere cedas la procesoron).

---

## Konstruado kaj kontrolado

Kontrolo farita la **2026-11-14** per Rust 1.100.0 (la minimuma subtenata
versio) en WSL2 (Ubuntu):

```sh
cargo build --workspace --release
#   Finished `release` profile [optimized] target(s) in 1.41s

cargo clippy --workspace --release
#   Finished `release` profile ... (neniu averto)

cargo fmt --all -- --check
#   (neniu diferenco)

bash tools/versio.sh --kontroli
#   (neniu malkongruo)
```

Ĉiuj tri praŝarĝaj vojoj estis praktike testitaj per QEMU 256 MiB (matrico de
la 2026-11-12 per Rust 1.99.1; la PVH- kaj ISO-BIOS-vojoj ripetiĝis la
2026-11-14 per 1.100.0 kun la sama scenaro: PVH `KASLR: kerna bazo 0x1200000`,
`izolado (bone): 64823 kadroj antaŭ, 64823 post`; ISO BIOS `KASLR: kerna bazo
0x7a00000`, `izolado (bone): 64821 kadroj antaŭ, 64821 post`):

| Vojo | Rezulto |
| --- | --- |
| PVH (`-kernel` + `-initrd`) | 7 regionoj, 255 MiB uzeblaj, sen framebuffer, `KASLR: kerna bazo 0x…` (malsama ĉiu starto), reuzoprovo `10 cikloj, 64823 kadroj antaŭ, 64823 post`, procezoj `4` kaj `5` samtempe pretaj, sama sekreta adreso `0x200000000`, id `4`/`5`, spiono `#PF` ĉe `0x200001000` (`erarkodo=0x4`), viktimo pluvivas kaj finas per `SYS_FINI`, `[ipc] purigo de procezo 5/4`, `34`/`35` kadroj liberigitaj, `izolado (bone): 64823 kadroj antaŭ, 64823 post`, 1000 tikoj, 5 fadenoj mortintaj |
| Limine/Multiboot2 (ISO, BIOS) | framebuffer `1280x800, 32 bpp`, `KASLR: kerna bazo 0x…`, reuzoprovo `10 cikloj, 64821 kadroj antaŭ, 64821 post`, procezoj `4` kaj `5` samtempe pretaj, sama sekreta adreso `0x200000000`, id `4`/`5`, spiono `#PF` ĉe `0x200001000` (`erarkodo=0x4`), viktimo pluvivas kaj finas per `SYS_FINI`, `[ipc] purigo de procezo 5/4`, `34`/`35` kadroj liberigitaj, `izolado (bone): 64821 kadroj antaŭ, 64821 post`, 1000 tikoj, 5 fadenoj mortintaj |
| Limine (ISO, UEFI/OVMF) | sama scenaro; `KASLR: kerna bazo 0x…`, liberaj kadroj `63367` antaŭ/post, framebuffer `1280x800, 32 bpp`, 1000 tikoj, 5 fadenoj mortintaj |

Por M13 Fazo 2 la sama scenaro ruliĝis kun `-cpu qemu64,+smep,+smap`: la
kerno raportas `SMEP subtenata: jes` kaj `SMAP subtenata: jes`, la tuta
M12-scenaro restas verda kaj nur la lasta memprovo haltigas ĝin, per
`[eraro] procezo 0: SMAP-malobservo … (erarkodo=0x1)`. La Ring 3-linioj kaj
`[uzanto] memoro: marko …, legita …` pruvas la pozitivan vojon: `SKRIBI`
legas la uzantan bufron kaj la ŝargilo skribas la provan paĝon nur en
`STAC`-fenestro. Kun `-cpu qemu64` la kerno raportas `ne` por ambaŭ trajtoj,
preterlasas la provon kaj finas la tutan scenaron sen escepto.

Por M13 Fazo 3 (KASLR) la bildo estas ET_DYN kaj ĉiu starto elektas alian
bazon: tri sinsekvaj PVH-startoj montris `0x5400000`, `0xd200000` kaj
`0x3000000`; ISO BIOS `0x4200000`, UEFI/OVMF `0x4a00000`. Ĉiuj ceteraj linioj
restis verdaj, inkluzive de `W^X` kaj de la izolado. Kun `-cpu qemu64` la
KASLR-vojo same funkcias (`0x8c00000`) kaj nur SMEP/SMAP estas preterlasitaj.

`just verify` unue kontrolas la dokumentan version per `tools/versio.sh
--kontroli` kaj poste vokas
`cargo kani -p mikerno -Z unstable-options --ignore-global-asm`; la 21 pruvoj
(6 pri la kadro-alokilo, 15 pri la IPC) pruviĝis verde per Kani 0.68.0.

### Kion vi devus vidi

Ĉe sukcesa starto la seriala konzolo montras (mallongigite):

```text
[MiKerno] Saluton, mi estas MiKerno v0.4.3!
[kerno] KASLR: kerna bazo 0x… (deŝovo 0x…); la bildo venis de 0x100000
[kerno] SMEP subtenata: jes
[kerno] SMAP subtenata: jes
[kerno] SMEP ŝaltita; Ring 0 ne povas plenumi uzantajn paĝojn.
[kerno] SMAP ŝaltita; Ring 0 legas uzantajn bufrojn nur en STAC-fenestroj.
[sistemvoko] pordego 0x80 (DPL3) instalita
[kerno] memoro: 7 regionoj, 255 MiB uzeblaj
[memoro] liberigo (bone): 64956 kadroj antaŭ, 64956 post
[memoro] W^X (bone): .text RX, .rodata R NX, .data/.bss RW NX, modulo R NX
[kerno] framebuffer ne havebla; nur COM1    (PVH; ISO montras 1280x800)
[tss] TR ŝargita; kernostako pinto 0x123ce10
[temporizilo] kalibrado: ... kalkuloj/s, komenca ...
[kerno] APIC 0 kaj paĝado pretaj; temporizilo 100 Hz
[procezo] 6 detruita: 1 privataj PDPT-oj, PML4 0x213000, 27 kadroj liberigitaj
... (10 reuzaj cikloj)
[memoro] reuzo (bone): 10 cikloj, 64823 kadroj antaŭ, 64823 post
[ipc] tabeloj (bone): kreitaj=13, plena=true, fermitaj=13, duobla_fermo=true, nova_generacio=true, malnova_tenilo=true, kerno=true
[ipc] rajtoj (bone): sendi per RICEVI-nura → -1, ricevi per SENDI-nura → -1
[ipc] izolado de kapabloj (bone): procezo 7 uzas tenilon de procezo 6 → -9
[ipc] dono (bone): rajtoj 0b011 → 0b010, eskalado 0b100 → -1
[ipc] revoko (bone): sen_revoki=-1, revoko=0, donita=-9, fermo=-9, duobla=-9
[kerno] IPC: mesaĝo sendita al fadeno 2
[kerno] planisto ekas: 3 fadenoj, antaŭokupo ĉiu 10 ms
saluton el Ring 0 tra int 0x80
[sistemvoko] Ring 0-memtesto: skribo=0, nekonata=-2, nula=-1, troa=-1, cedo=0, krei=-28, doni=-9, fermi=-9, revoki=-9, revok_argumento=-1
...
[fadeno 1] revokis la defaŭltan finpunkton de fadeno 3
[fadeno 3] revoka veko (bone): kodo -32
...
[planisto] fadeno 3 finiĝis
[uzanto] horizonto: bildo 0x100000000..0x100005158, enirpunkto 0x1000020f0
[uzanto] stako 0x100ff0000..0x101000000 (64 KiB), pinto 0x101000000
[uzanto] kontrolo (bone): skriba=true lega=true nurlega_skribo=false kerna=false nemapita=false translima=false eksterfenestra=false enirpunkta=true enir_rx=true stako_rw=true
[uzanto] memoro: marko 0x4d694b65726e6f21, legita 0x4d694b65726e6f21
[procezo] 4: Preta, kerna stako 0x1254e10, PML4 0x213000, kerna mapo kunhavata, bildo nur en la proceza spaco jes, sekreta paĝo 0x200000000
[kerno] horizonto 4 pretas: CS=0x23, SS=0x1B, RFLAGS=0x202, enirpunkto 0x1000020f0
[uzanto] ... kontrolo (bone) ...
[procezo] 5: Preta, kerna stako 0x1258e10, PML4 0x236000, kerna mapo kunhavata, bildo nur en la proceza spaco jes, sekreta paĝo 0x200000000
[kerno] horizonto 5 pretas: CS=0x23, SS=0x1B, RFLAGS=0x202, enirpunkto 0x1000020f0
[kerno] du procezoj pretaj; unu planista voko
horizonto: procezo vekis en Ring 3.
[ring3] procezo 5: idento (bone): sendinto=5 (uzanto asertis 999), ricevonto=5, resto_nula=jes
[ring3] procezo 5: finpunktoj (bone): tenilo=257, fermo=0, duobla=-9, malvalida=-9, nulaj_permesoj=-1, nekonataj_permesoj=-1, rekrea=513, malnova=-9
[ring3] procezo 5: rajtoj (bone): sendi_per_ricevi=-1, ricevi_per_sendi=-1
[ring3] procezo 5: revoko (bone): argumento=-1, sen_revoki=-1, fermo_sen=0, revoko=0, sendo=-9, fermo=-9, duobla=-9
[eraro] procezo 5: montrilo 0x1000 (longo 8) estas ekster la anoncita regiono
[eraro] procezo 5: montrilo 0x1000020f0 (longo 96, skriba true) ne havas la postulatan folian permeson
[ring3] negativaj: nekonata=-2, nula=-1, kerna=-1, troa=-1, nurlega=-1
[ring3] negativaj (F3): tenilo6=-9, tenilo0=-9
[ring3] procezo 5: sekreta paĝo kongruas (magio 0x4d694b65726e6f21, id 5)
[ring3] procezo 5: spionas la viktimon
horizonto: procezo vekis en Ring 3.
[ring3] procezo 4: rajtoj (bone): sendi_per_ricevi=-1, ricevi_per_sendi=-1
[ring3] procezo 4: revoko (bone): argumento=-1, sen_revoki=-1, fermo_sen=0, revoko=0, sendo=-9, fermo=-9, duobla=-9
[eraro] procezo 4: montrilo 0x1000 (longo 8) estas ekster la anoncita regiono
[eraro] procezo 4: montrilo 0x1000020f0 (longo 96, skriba true) ne havas la postulatan folian permeson
[ring3] negativaj: nekonata=-2, nula=-1, kerna=-1, troa=-1, nurlega=-1
[ring3] procezo 4: sekreta paĝo kongruas (magio 0x4d694b65726e6f21, id 4)
[ring3] procezo 4: laboras (fojo 1)
[ring3] procezo 5: legas la duan sekretan paĝon ĉe 0x200001000

[eraro] procezo 5: paĝa eraro ĉe rip=0x0000000100001cdb, erarkodo=0x4, CR2=0x0000000200001000; la procezo estas mortigata
[ipc] purigo de procezo 5: 1 kapabloj liberigitaj, 1 finpunktoj revokitaj
[planisto] fadeno 5 finiĝis
[procezo] 5 detruita: 1 privataj PDPT-oj, PML4 0x236000, 34 kadroj liberigitaj
[ring3] procezo 4: laboras (fojo 2)
...
[ring3] procezo 4: laboras (fojo 6)
[ring3] procezo 4: pluvivas post la spiono; finas per SYS_FINI
[ipc] purigo de procezo 4: 1 kapabloj liberigitaj, 1 finpunktoj revokitaj
[planisto] fadeno 4 finiĝis
[procezo] 4 detruita: 1 privataj PDPT-oj, PML4 0x213000, 35 kadroj liberigitaj
[memoro] izolado (bone): 64823 kadroj antaŭ, 64823 post
[kerno] planisto revenis al la ripoza fadeno
...
[kerno] fina stato: 5 mortintaj, 0 pretaj, 0 blokitaj; 1000 tikoj
[memoro] SMAP-provo: Ring 0 provos legi la uzantan folion ĉe 0x100000000
[eraro] procezo 0: SMAP-malobservo ĉe 0x0000000100000000 (erarkodo=0x1)
```

---

## Testa scenaro

`kerno::ĉefa` kreas tri fadenojn kun statikaj 16 KiB-stakoj:

1. **Fadeno 2** tuj blokiĝas en `ricevi_mesagon(0)`; la praŝarĝo sendas al ĝi
   mesaĝon `tipo 1` («saluton el la praŝarĝo»), kiu vekas ĝin.
2. **Fadeno 3** blokiĝas sur sia defaŭlta finpunkto (`ricevi_mesagon(1)`), kaj
   ĉiuj tri fadenoj dek fojojn printas sian progreson kaj cedas la procesoron.
3. En la tria fojo **fadeno 1** revokas la defaŭltan finpunkton de fadeno 3 per
   `revoki_defaŭltan`; la blokita fadeno 3 vekiĝas kun `-32` (`Revokita`) kaj
   printas `[fadeno 3] revoka veko (bone): kodo -32`.
4. Post la lasta fojo ĉiu fadeno finiĝas; la planisto revenas al la ripoza
   kunteksto.
5. Antaŭ la uzantaj procezoj la praŝarĝo rulas la reuzoprovo: ĝi kreas la
   procezan spacon kun la sama bildo kaj stako, detruas ĝin kaj ripetas dek
   fojojn. La nombro de liberaj kadroj devas esti egala antaŭ kaj post.
6. La praŝarĝo preparas **du** procezojn antaŭ iu ajn planista voko, dum
   interrompoj restas malŝaltitaj: procezo `4` (`REĜIMO_SEKRETO`, la viktimo)
   kaj procezo `5` (`REĜIMO_SPIONO`). Ĉiu ricevas sekretan paĝon ĉe la sama
   virtuala adreso `0x2_0000_0000`, sed malsaman kadron enhavantan sian propran
   identigilon; la viktimo krome mapas duan paĝon ĉe `0x2_0000_1000`. Nur poste
   `planisto::lanĉi()` ekas; la du procezoj tiel interplektiĝas.
7. Ĉiu procezo ekas en Ring 3 per `iretq` (`RDI`=reĝimo, `RSI`=identigilo) kaj
   legis sian sekretan paĝon: `4` vidas `id 4`, `5` vidas `id 5`. Tio pruvas, ke
   la sama virtuala adreso montras malsamajn fizikajn kadrojn.
8. Procezo `5` legas la duan paĝon `0x2_0000_1000`, kiun ĝia spaco ne havas. La
   `#PF` el Ring 3 kun `erarkodo=0x4` (`U=1`, `P=0`) estas kaptita; la kerno
   printas `[eraro] procezo 5 …`, mortigas la procezon kaj `detruu` liberigas
   ĝian tutan spacon. La planisto elektas procezon `4`; neniu `halti()` okazas.
9. Procezo `4` pluvivas la spionon, printas la rondojn `2`–`6` kaj eliras per
   `fini(0)`; `SYS_FINI` detruas ankaŭ ĝian spacon. Antaŭ la detruo
   `ipc::purigi_procezon` liberigas ĝiajn kapablojn kaj revokas ĝian
   finpunkton (`[ipc] purigo de procezo 4 …`); la sama okazas por la spiono.
   La praŝarĝo logas `[memoro] izolado (bone)` kaj la liberaj kadroj egalas
   antaŭ kaj post.
10. La praŝarĝo observas la ceterajn 1000 tikojn (≈10 s ĉe 100 Hz), printas la
    finan staton `5 mortintaj` kaj haltas per `hlt`.

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
│   │   ├── linker.ld           # 1 MiB, ET_DYN, RELATIVE-relokoj
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
│       │   ├── sistemvoko.rs   # SKRIBI/FINI/CEDI/SENDI/RICEVI/KREI_FINPUNKTON/DONI/REVOKI/FERMI
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
│   ├── run-qemu.sh             # QEMU-lanĉilo por WSL2/Linux
│   ├── font/generi.py          # DejaVu → bitmapaj strekoj
│   └── limine/                 # praŝarĝilo (binaraj dosieroj)
├── third_party/dejavu/         # permesilo de la fonto
└── docs/
    ├── FLUOU.md                # deviga GUI-gvidilo (FluoU)
    ├── FLUOU-montro.html       # vida montro de la gvidilo
    ├── MEJLOSTONOJ.md          # mejloŝtonoj kaj pruvaro
    └── VORTARO.md              # Esperanta terminologio
```

---

## Limigoj kaj sekva laboro

- **Neniu `fork`/`exec`**: la du `horizonto`-procezoj ekzistas kaj interplektiĝas
  ekde la starto, sed Ring 3 ne povas krei novan procezon nek anstataŭigi sian
  bildon; la praŝarĝo decidas la aron antaŭ la unua planista voko.
- **Statikaj kernaj stakoj**: la kernaj stakoj estas fiksa tabelo de ok
  statikaj 16 KiB blokoj en `.bss` anstataŭ kadraj alokoj; la proceza detruo
  ne resorbas tiujn blokojn.
- **Proceza detruo estas antaŭkondiĉa**: `detrui` postulas, ke neniu alia
  fadeno de la procezo plenumas kaj ke la vokanto tenas interrompojn
  malŝaltitaj; kun unu uzanta fadeno po procezo tio ĉiam validas.
- **W^X, SMEP, SMAP kaj KASLR**: la identeca mapo estas dividita laŭ sekcioj
  (`.text` RX, `.rodata` R NX, datumoj RW NX) kaj `CR0.WP` protektas la
  nurlegajn paĝojn ankaŭ el Ring 0; `CR4.SMEP` malpermesas al Ring 0 plenumi
  uzantajn foliojn (0.3.0), kaj `CR4.SMAP` malpermesas legi aŭ skribi ilin
  ekster `STAC`/`CLAC`-fenestro (0.3.1). `hazardigo` (0.3.2) kopias la ET_DYN
  bildon al hazarda 2 MiB-aliniiĝinta bazo kaj relokas ĝin; la kerno restas en
  la malalta duono, do la identeca mapo kaj ĉiuj ekzistantaj scenaroj
  senŝanĝe funkcias. Alta-duona kerno ne estis necesa por tiu ĉi firmigo, kaj
  IOMMU-izolado venas en M18 (post 1.0).
- **Unuprocesora**: neniu SMP, afineco, prioritatoj aŭ dormantaj temporiziloj.
- **Fiksaj limoj**: 8 fadenoj, 16 mesaĝoj po fadeno, 64 bajtoj po mesaĝo,
  4 GiB fizika memoro.
- **IPC**: la finpunktaj objektoj havas generaciojn kaj referenckalkulon, kaj
  `SENDI`/`RICEVI` uzas kapablo-tenilojn kun permesa kaj du-nivela generacia
  kontrolo; `REVOKI` estas tutfinpunkta kaj proceza detruo purigas la
  kapablojn; mankas tempolimoj, sendblokado kaj `fork`/`exec`-distribuo de
  kapabloj (Fazoj 5–6).
- **Ring 3-esceptoj nur `#PF`/`#GP`**: tiuj du mortigas la procezon; aliaj
  esceptoj — ankaŭ el Ring 3 — restas fatalaj, kaj demando-paĝado ne
  ekzistas.
- **Framebuffer**: nur 32 bpp, kvar fiksaj strekoj (8×16 … 32×64), 111 glifoj
  (95 ASCII + Esperanto + citiloj), malplenigo anstataŭ rulumo.
- **Pruvoj**: 21 Kani-pruvoj spegulas la bitan logikon sur 64 kadroj kaj
  malgrandaj lokaj kapablotabeloj; la produkta kodo mem ne estas pruvita.
  `just verify` postulas apartan Kani-instalon.
- **Neniu CI nek testoharness**: la kontrolo estas `build` + `clippy` + `fmt` +
  QEMU-smoka testo.
- **Versio**: la sola fonto estas `Cargo.toml`; la saluta linio legas
  `env!("CARGO_PKG_VERSION")` dum kompilado, do `just run` kaj la ISO montras
  la saman version, kaj `limine.conf` ne portas version. La dokumentojn
  kontrolas `tools/versio.sh --kontroli` (parto de `just verify`), kaj
  `--ĝisdatigi` ilin reskribas.
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

La originala kodo de MiKerno estas publikigita sub la permesilo **MIT** (vidu
[`LICENSE`](LICENSE)). Vi rajtas uzi, modifi, kunfandi, publikigi, disdoni kaj
vendi la programaron, kondiĉe ke la kopirajta avizo kaj la permesila teksto
restu kunigitaj kun la disdonataĵo. La programaro estas provizita "tia, kia ĝi
estas", sen ia ajn garantio.

MiKerno elektis MIT anstataŭ Apache-2.0 ĉar la kerno estas malgranda kaj sen
eksteraj dependaĵoj, kaj ĉar MIT restas kongrua kun GPLv2 — tio faciligas reuzon
de la koderoj en aliaj kernaj projektoj.

La triaj komponantoj listigitaj sube havas siajn proprajn permesilojn; la MIT
permesilo ne anstataŭas ilin.

Enigitaj triaj komponantoj:

| Komponanto | Loko | Noto |
| --- | --- | --- |
| Limine | `tools/limine/` | Praŝarĝilo por la ISO (Multiboot2) |
| DejaVu Sans Mono (bitmapaj strekoj) | `mikerno/src/ekrano/fonto/` | Rasterigita el DejaVu Sans Mono; permesilo Bitstream Vera + Arev en `third_party/dejavu/LICENSE.txt` |
| `tools/font/generi.py` | `tools/font/` | Generilo por la kvar strekoj; postulas Python 3 kaj Pillow |

`xorriso` ne estas enigita en la deponejon; `just iso` uzas la sisteman
pakaĵon (vidu [Postuloj](#postuloj)).

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
