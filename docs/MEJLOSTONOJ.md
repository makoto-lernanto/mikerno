# Mejloŝtonoj de MiKerno

> Stato: 2026-11-12 · Versio de la workspace: 0.1.5
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
| M9 | Kvalita pordego: fmt, clippy, Kani-pruvoj | Parte | fmt/clippy verdaj; 11 pruvoj verkitaj, Kani ne instalita |
| M10 | Uzantspaca kontrakto (`libmikerno`, `horizonto`) | Plenumita | ABI difinita; `horizonto` plenumiĝas en Ring 3 |
| M11 | Ring 3 kaj sistemvokoj | Plenumita | `int 0x80` el Ring 0 kaj Ring 3, negativaj kazoj, ambaŭ praŝarĝaj vojoj (0.1.5) |
| M12 | Per-procesaj adresspacoj | Venonta | — |
| M13 | IPC-firmigo: objektoj, rajtoj, tempolimoj | Venonta | — |
| M14 | Dormantaj temporiziloj, prioritatoj, SMP-esploro | Venonta | — |
| M15 | Konservado, dosiersistemo, ŝelo | Venonta | — |
| M16 | CI, testoj, plena pruvaro | Venonta | — |

---

## 2. Plenumitaj mejloŝtonoj

### M0 — Workspace kaj konstrua pipeline

**Celo.** Unu `cargo`-workspace kun klara apartigo inter kerno, komuna ABI kaj
uzantspaco, riproduktebla per unu komando.

**Efektivigo.**

- `Cargo.toml` — workspace kun `mikerno`, `libmikerno`, `horizonto`;
  `edition = "2024"`, `rust-version = "1.85"`, `resolver = "2"`.
- `.cargo/config.toml` — cela triopo `x86_64-unknown-none`, `rust-lld`,
  `relocation-model=static`, kaj Windows-runner al `tools\run-qemu.cmd`.
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
- Permesoj: `KERNELA` (RWX, identeca mapo), `SKRIBA` (APIC-MMIO),
  `SKRIBA_SENKAŜE` (framebuffer, `PCD`).
- `EFER.NXE` estas ŝaltita en `pravalorizi`.

**Pruvo.** Post `CR3`-ŝalto la kerno daŭre uzas la framebuffer-on (ISO) kaj la
APIC-on; neniu paĝa eraro aperas dum la tuta testa scenaro.

**Resto.** La identeca mapo estas RWX; ne ekzistas W^X, SMEP/SMAP aŭ per-procesa
mapado.

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
   `TSS.RSP0` restas M12-tasko.
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

## 3. Partaj mejloŝtonoj

### M9 — Kvalita pordego
**Celo.** Ĉiu ŝanĝo estu pruvebla per ripeteblaj komandoj.

**Havigita.**

- `cargo fmt --all -- --check` — pura.
- `cargo clippy --workspace --release` — sen avertoj.
- `cargo build --workspace --release` — sukcesa.
- 11 Kani-pruvoj: 5 en `memoro/kadro.rs`, 6 en `kerno/ipc.rs`.
- `just verify` difinas la formalan pordegon.

**Mankanta.**

- `cargo kani` ne estas instalita en la kontrolmedio (`error: no such command:
  kani`), do la pruvoj ne estis maŝine plenumitaj.
- La pruvoj spegulas la logikon per 64-kadra modelo anstataŭ pruvi la
  produktan `KadraAlokilo`.
- Neniu unuopa/bitnivela testo, neniu CI, neniu `miri` aŭ `loom`.

**Sekva paŝo.** Instali Kani en CI, aldivi la pruvojn al la produkta kodo per
etaj abstraktaĵoj, kaj aldoni testojn por la arkitekture sendependaj partoj
(`MemoraMapo`, `Mesagujo`, `PretaVico`, `VirtuaAdreso`).

---

## 4. Pruvprotokolo de 2026-11-12

Medio: Windows, Rust 1.99.1, QEMU 256 MiB, Git Bash.

### 4.1 Statika kontrolo

| Komando | Rezulto |
| --- | --- |
| `cargo build --workspace --release` | Sukceso, 5,20 s, sen avertoj |
| `cargo clippy --workspace --release` | Sukceso, sen avertoj |
| `cargo fmt --all -- --check` | Sen diferencoj |
| `cargo kani --version` | `error: no such command: kani` |
| `just iso` | ISO kreita, 2 184 sektoroj |

### 4.2 PVH-starto

Komando (senkapa QEMU, 25 s):

```sh
qemu-system-x86_64 -kernel target/x86_64-unknown-none/release/mikerno \
  -initrd target/x86_64-unknown-none/release/horizonto \
  -m 256M -serial stdio -display none -no-reboot -no-shutdown
```

Elĉerpaĵo:

```text
[MiKerno] Saluton, mi estas MiKerno v0.1.5!
[kerno] memoro: 7 regionoj, 255 MiB uzeblaj
[kerno] framebuffer ne havebla; nur COM1
[tss] TR ŝargita; kernostako pinto 0x...
[temporizilo] kalibrado: ... kalkuloj/s, komenca ...
[uzanto] horizonto: bildo 0x100000000..0x100001082, enirpunkto 0x1000000d0
...
[kerno] planisto ekas: 3 fadenoj, antaŭokupo ĉiu 10 ms
...
[kerno] horizonto ekas en Ring 3: CS=0x23, SS=0x1B, RFLAGS=0x202, enirpunkto 0x1000000d0
horizonto: la unua procezo vekis en Ring 3.
[ring3] negativaj: nekonata=-2, nula=-1, kerna=-1, troa=-1
[planisto] fadeno 4 finiĝis
[kerno] horizonto finiĝis; planisto revenis
[kerno] planisto revenis al la ripoza fadeno
...
[kerno] fina stato: 4 mortintaj, 0 pretaj, 0 blokitaj; 1000 tikoj
```

### 4.3 ISO-starto

Komando:

```sh
qemu-system-x86_64 -cdrom mikerno.iso -m 256M -serial stdio \
  -display none -no-reboot -no-shutdown
```

Elĉerpaĵo:

```text
[MiKerno] Saluton, mi estas MiKerno v0.1.5!
[kerno] memoro: 7 regionoj, 255 MiB uzeblaj
[kerno] framebuffer 1280x800, 32 bpp
...
[kerno] horizonto ekas en Ring 3: CS=0x23, SS=0x1B, RFLAGS=0x202, enirpunkto 0x1000000d0
horizonto: la unua procezo vekis en Ring 3.
[ring3] negativaj: nekonata=-2, nula=-1, kerna=-1, troa=-1
...
[kerno] fina stato: 4 mortintaj, 0 pretaj, 0 blokitaj; 1000 tikoj
```

### 4.4 Trovaĵoj de la kontrolo

1. **Malaktuala ISO (grava).** La unua ISO-starto printis
   `MiKerno v0.1.1`, dum la fonto enhavas `v0.1.0`: `iso_root/` kaj
   `mikerno.iso` estis malnovaj. Post `just iso` la ISO printis `v0.1.0` kaj
   plenumis la tutan scenaron. Ĉar ambaŭ estas git-ignoritaj, ĉi tio povas
   ripetiĝi; la `justfile` devus eventuale fari la version videbla aŭ la
   kontrolo devus ĉiam rekonstrui.
2. **Versio hardkodita.** `kerno/mod.rs`, `mikerno/boot/limine.conf` kaj
   `Cargo.toml` enhavas `0.1.5` sendepende. Oni devus derivi ĝin per
   `env!("CARGO_PKG_VERSION")` kaj `build.rs`.
3. **Kani mankas.** La formala pordego ne estas plenumebla en ĉi tiu medio;
   `just verify` malsukcesos ĝis `cargo kani` estos instalita.
4. **Piksela kontrolo nur per `screendump`.** La framebuffer estis vidita per
   QEMU `screendump` (2026-11-12): la titolo, la ĉapelitaj literoj kaj la
   citiloj desegniĝas ĝuste. La alta distingivo (16×32/32×64) ne estis provita
   en QEMU; ĝi dependas nur de la sama elekta funkcio, sed meritas apartan
   starton kun pli granda framebuffer. La aspekto sur reala aparato ankoraŭ
   ne estas kontrolita.
5. **`just run-iso` malfermas SDL-fenestron**; por senkapa kontrolo oni voku
   QEMU rekte aŭ uzu `QEMU_DISPLAY=none`.

---

## 5. Venontaj mejloŝtonoj

### M12 — Per-procesaj adresspacoj

**Celo.** Izoli uzantajn procezojn.

**Taskoj.** Nova `PML4` por ĉiu procezo; kopii la kernan duonon; mapi uzantajn
foliojn nur en la propra spaco; ŝalti `CR3` dum kuntekstŝanĝo; validigi
uzantajn montrilojn; malmapi kaj liberigi kadrojn ĉe eliro.

**Akceptkriterioj.** Du procezoj ne povas legi unu la alian; paĝa eraro en unu
procezo ne haltigas la kernon; la kadro-alokilo reuzas liberigitajn kadrojn.

**Antaŭkondiĉo.** Publika `liberigi` en `KadraAlokilo` kaj liberigo de paĝtabeloj.

---

### M13 — IPC-firmigo

**Celo.** Apartigi finpunktojn de fadenoj kaj firmigi la kapablomodelon.

**Taskoj.** Propraj finpunktaj objektoj; kapablotabelo por procezo; rajtoj kaj
revoko; tempolimoj kaj sendblokado; pli da Kani-pruvoj pri la vico kaj la
kapabloj.

**Akceptkriterioj.** Mesaĝo portas `sendinto`/`ricevonto`; revoko malvalidigas
kapablon; plena vico ne perdas mesaĝojn; tempolimo vekas la ricevanton.

---

### M14 — Temporiziloj kaj planado

**Celo.** Dormantaj fadenoj, prioritatoj kaj mezuro de plenumtempo.

**Taskoj.** Unuopa vekotemporizilo (min-heap), `dormi(md)`, prioritatoj aŭ
loteria planado, statistiko, eventuala SMP-esploro kun IPI.

**Akceptkriterioj.** Fadeno dormanta 100 ms vekiĝas en tolero; la planisto ne
malsatigas fadenojn; la ripoza kunteksto restas senŝanĝa.

---

### M15 — Konservado kaj ŝelo

**Celo.** Uzebla sistemo, ne nur demonstracio.

**Taskoj.** `virtio-blk`- pelilo, simpla dosiersistemo (ekz. FAT aŭ propra
read-only), ŝelo kun `echo`, `ls`, `cat`, `ps`, `kill`.

**Akceptkriterioj.** La ŝelo rulas en Ring 3, legas dosieron kaj lanĉas duan
procezon.

---

### M16 — Inĝeniera matureco

**Celo.** Ripetebla, pruvita kaj dokumentita projekto.

**Taskoj.** CI (Linux + Windows), Kani en CI, testoj por arkitekture sendependaj
moduloj, QEMU-matrico (BIOS/UEFI, diversaj memorkvantoj), `cargo-deny` /
permesila kontrolo, `LICENSE`, regenerado de la ISO en CI.

**Akceptkriterioj.** Ĉiu PR plenumas `fmt`, `clippy`, `build`, `kani` kaj la
QEMU-smokan teston; la ISO estas artefakto de CI, ne permana dosiero.

---

## 6. Inĝenieraj decidoj

| Decido | Kialo | Kosto |
| --- | --- | --- |
| Neniu ekstera Rust-dependaĵo | Aŭdebleco, riparebleco, plena kontrolo | Pli da propra kodo |
| Du praŝarĝaj protokoloj | PVH por rapida `cargo run`; MB2 por reala ISO | Du analiziloj kaj du magioj |
| Esperanto en kodo kaj dokumentaro | Konsekvenco kaj distingiĝo | Malpli da kontribuantoj |
| Identeca mapo | Simpla praŝarĝo kaj aliro al MMIO | Ne W^X, ne izolita |
| Statikaj kernaj strukturoj | Sen kerna heap; antaŭvidebla memoro | Fiksaj limoj (8 fadenoj, ...) |
| Interromp-malŝaltaj kritikaj sekcioj | Unuprocesora; simpla kaj sendependa de ŝlosiloj | Ne skaliĝas al SMP |
| `EOI` antaŭ la traktilo | La traktilo rajtas ŝanĝi kuntekston; la sekva tiko devas alveni | Pli da nestitaj interrompoj |
| PIT-kalibrado de la APIC | Busfrekvenco ne estas arkitekture legebla | 50 ms da starta prokrasto |
| 4 GiB fizika limo | Bitara indekso kaj identeca mapo | Ne subtenas pli da RAM |
| Kopiataj mesaĝoj | Sendanto povas tuj reuzi sian bufron | 64-bajta limo |
| Kani-speguloj | Traktebla stato-spaco | La produkta kodo ne estas pruvita |

---

## 7. Teknika ŝuldo (prioritatigita)

| Prioritate | Ŝuldo | Efiko | Rimedo |
| --- | --- | --- | --- |
| Alta | Neniu uzanta izolado (komuna `PML4`, komuna kern-stako `RSP0`) | Procezoj vidas unu la alian | M12 |
| Alta | Neniu liberigo de kadroj | Memoro elĉerpiĝas post longa uzo | M12 |
| Alta | Identeca mapo RWX | Malobservo de W^X | M12/M16 |
| Alta | Neniu CI nek testoj | Regresoj nerimarkeblaj | M16 |
| Alta | Malaktuala ISO | Konfuzaj testrezultoj | CI + versio |
| Meza | Versio hardkodita en 3 lokoj | Malkongruaj eldonoj | `env!` + `build.rs` |
| Meza | IPC finpunkto = fadeno | Ne eblas sendependaj objektoj | M13 |
| Meza | Framebuffer sen rulumo | Malbona uzsperto | M6-poste |
| Meza | Esceptoj haltigas | Ne rekaptebla eraro | M12 |
| Malalta | `Kunhavigita::akiri` prenas `&self` | Facila misuzo | revizio |
| Malalta | Kani-modelo ≠ produkta kodo | Pruvoj ne rekte aplikeblas | M9 |
| Malalta | Neniu `LICENSE` | Jura neklaro | M16 |

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
