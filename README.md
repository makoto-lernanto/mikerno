# MiKerno

> «La estonteco nun.»

> **Pri la aŭtoreco.** `makoto-lernanto` estas la arkitekto de ĉi tiu projekto:
> li difinas la celojn, la arĥitekturon kaj la inĝenieran pensmanieron. La tuta
> efektiva kodo — same kiel la dokumentaro kaj la testoj — estis verkita de
> **Qoder**, la AI-programada asistanto, kaj ankaŭ kunlabore kun **OpenCode**
> (modeloj **DeepSeek V4.1 Flash Free** kaj **MiMo-V2.6-Flash Free**) kaj
> **GitHub Copilot**. La homa
> kontribuo estas la ideo, ne la klavoj.

MiKerno estas eta, memstara x86_64-kerno verkita tute en Rust (`#![no_std]`,
`#![no_main]`), sen eksteraj Rust-dependaĵoj kaj sen C-kodo en la kerno mem. La
kodo, la protokoloj kaj la dokumentaro estas en Esperanto.

La celo de la projekto estas fari legeblan, hobleblan kaj pruveblan
demonstration de kompleta unuprocesora kerno: de la unua instrukcio en 32-bita
reĝimo ĝis antaŭokupa planisto, mesaĝa IK kaj uzanta procezo en Ring 3.

- **Versio:** 0.9.3 (Cargo-laborloko)
- **Eldonkandidato:** `v0.9.3` (antaŭeldono por `v1.0.0`; la plej nova
  stabila eldono estas `v0.9.2`)
- **Arkitekturo:** `x86_64` kaj `arm64` (AArch64), unuprocesora
- **Cela triopo:** `x86_64-unknown-none`
- **Eldono de Rust:** 2024, `rust-version = 1.100`
- **Statuso:** funkcianta prototipo — vidu [Mejloŝtonoj](docs/MEJLOSTONOJ.md)
- **GU:** deviga kadro FluoU — vidu [docs/FLUOU.md](docs/FLUOU.md)
- **Retejo:** [makoto-lernanto.github.io/mikerno](https://makoto-lernanto.github.io/mikerno/)

---

## Enhavo

1. [Kapabloj](#kapabloj)
2. [Rapida starto](#rapida-starto)
3. [Arĥitekturo](#arĥitekturo)
4. [Praŝarĝo kaj eniro](#praŝarĝo-kaj-eniro)
5. [Memoro](#memoro)
6. [Interrompoj, APIC kaj temporizilo](#interrompoj-apic-kaj-temporizilo)
7. [Planisto](#planisto)
8. [IK kaj kapabloj](#ik-kaj-kapabloj)
9. [Eligo: COM1 kaj frambufo](#eligo-com1-kaj-frambufo)
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
| Kaskada forigo de adresa spaco (`detruu`) kun liberigo de paĝtabeloj | Funkcias | `arkitekturo/x86_64/pagado.rs` |
| Preciza liberigo de privataj `PDPT`-oj (inkl. `PML4[0]`) sen tuŝi kunhavatajn kernajn mapojn | Funkcias | `pagado.rs`, `kerno/procezo.rs` |
| Per-procesa adresa spaco: privata PML4, kunhavataj kernaj mapoj (`krei_kunhavantan`) | Funkcias | `kerno/procezo.rs`, `pagado.rs` |
| Proceza detruo ĉe `SYS_FINI` kaj ĉe mortiga Ring 3-escepto (`#PF`, `#GP`, `#DE`, `#OF`, `#UD`) | Funkcias | `kerno/planisto.rs`, `interrompo/mod.rs` |
| `CR3`-ŝalto dum kuntekstŝanĝo kaj per-procesa `TSS.RSP0` (16 KiB kerna stako) | Funkcias | `kerno/planisto.rs`, `kerno/procezo.rs`, `tss.rs` |
| Strikta uzanta mapado (`mapi_uzantan`): fenestra limkontrolo kaj devigata bito `U` | Funkcias | `memoro/virtuala.rs`, `kerno/uzantspaco.rs` |
| 4-nivela paĝado (PML4 → PDPT → PD → PT), NX, PCD | Funkcias | `arkitekturo/x86_64/pagado.rs` |
| SMEP: Ring 0 ne povas plenumi uzantajn foliojn (`CR4.SMEP`), kun CPUID-provo kaj gracia malŝalto | Funkcias | `arkitekturo/x86_64/firmigo.rs`, `procesoro.rs` |
| SMAP: Ring 0 legas/skribas uzantajn foliojn nur en `STAC`/`CLAC`-fenestroj (`CR4.SMAP`), kun RAII-gardo | Funkcias | `arkitekturo/x86_64/firmigo.rs`, `kerno/sistemvoko.rs` |
| GDT, 256-vektora IDT, escepta raportado | Funkcias | `gdt.rs`, `idt.rs`, `interrompo/mod.rs` |
| Uzantaj DPL3-segmentoj kaj `int 0x80`-pordego (interrompa, `0xEE`) | Funkcias | `gdt.rs`, `idt.rs`, `kerno/sistemvoko.rs` |
| Dek unu sistemvokoj per `int 0x80`: `SKRIBI`/`FINI`/`CEDI`/`SENDI`/`RICEVI`/`KREI_FINPUNKTON`/`DONI`/`REVOKI`/`FERMI`/`RICEVI_TEMPO`/`SENDI_TEMPO`, unueca `Mesago`, montrila kontrolo | Funkcias (Ring 0 kaj 3) | `kerno/sistemvoko.rs`, `arkitekturo/x86_64/sistemvoko.rs` |
| ABI 2: `Tenilo` kun du-nivela generacia kontrolo, kapablotabelo, `KREI_FINPUNKTON`/`DONI`/`REVOKI`/`FERMI`, revokaj vekoj `-32`, sendblokado ĉe plena vico, `-90`, `IkStatistiko`, tempolimoj `-110`/`-11`, purigo ĉe proceza detruo | Funkcias (ekde 0.4.5) | `libmikerno/`, `kerno/ik.rs` |
| TSS kun `RSP0` por Ring 3 → Ring 0 | Funkcias | `tss.rs` |
| xAPIC (MMIO), malŝalto de 8259-PIC, spuria vektoro `0xFF` | Funkcias | `apic.rs` |
| 100 Hz perioda temporizilo, kalibrita kontraŭ la PIT | Funkcias | `temporizilo.rs` |
| Antaŭokupa rondiga planisto, 8 fadenoj + ripoza kunteksto | Funkcias | `kerno/planisto.rs` |
| Mesaĝa IK: finpunktoj, kapabloj, blokado/vekado | Funkcias | `kerno/ik.rs` |
| Frambufa teksta konzolo (4 strekoj, 8×16 … 32×64) kaj COM1 | Funkcias | `ekrano/`, `seria.rs` |
| Komuna kerna/uzanta ABI (`libmikerno`) | Difinita | `libmikerno/` |
| Uzanta fenestro super 4 GiB, ELF64-ŝargo kun bito `U`, 64 KiB stako | Funkcias | `kerno/uzantspaco.rs`, `memoro/virtuala.rs` |
| Ring 3-transiro per `iretq` (`CS=0x23`, `SS=0x1B`, `IF=1`) | Funkcias | `arkitekturo/x86_64/uzanta.rs` |
| Unua uzanta procezo `horizonto`: `SKRIBI` + `FINI` el Ring 3 | Funkcias | `horizonto/`, `kerno/sistemvoko.rs` |
| Ring 3-escepta kapto: `#PF`, `#GP`, `#DE`, `#OF` kaj `#UD` mortigas nur la procezon kaj la sistemo daŭras | Funkcias | `interrompo/mod.rs` |
| Samtempaj procezoj `4`/`5` kun sama sekreta adreso sed malsamaj kadroj | Funkcias | `kerno/mod.rs`, `kerno/uzantspaco.rs` |
| Izolado de adresa spaco: spiono `#PF` ĉe nur-viktima paĝo, viktimo pluvivas | Funkcias | `horizonto/`, `interrompo/mod.rs` |
| 29 formalaj pruvoj per Kani | Verkitaj | `kadro.rs`, `ik.rs` |

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
- **Kani** (nur en DI) instaliĝas aparte per
  `cargo install --locked kani-verifier` kaj `cargo kani setup`.

### Postuloj

| Ilo | Versio / noto |
| --- | --- |
| Rust | 1.100 aŭ pli nova (eldono 2024); projekto kontrolita per 1.100.0 |
| Cela triopo | `x86_64-unknown-none`; `aarch64-unknown-none` por `arm64` |
| QEMU | `qemu-system-x86_64` (`sudo apt install qemu-system-x86`) |
| QEMU (arm64) | Nur por `just ruli-arm` kaj `just ruli-iso-arm`: `qemu-system-aarch64`; por la ISO ankaŭ UEFI-firmware (`sudo apt install qemu-system-arm qemu-efi-aarch64`) |
| xorriso | Nur por `just iso` kaj `just iso-arm` (`sudo apt install xorriso`) |
| dosfstools / mtools | Nur por `just iso-arm` (`sudo apt install dosfstools mtools`) |
| `just` | Opcia, sed rekomendata por la difinitaj taskoj |
| bash | Lanĉilo de `tools/run-qemu.sh` kaj `tools/run-qemu-arm.sh` |
| python3 | SNDS-bildo por `tools/krei_snds_img.sh` (kutime jam instalita) |
| Kani | Nur en DI (`cargo kani`) |
| Miri | Nur en DI aŭ per `just miri`: nokta Rust kun `miri` kaj `rust-src` |
| cargo-deny | Nur por `just permesiloj` kaj la DI-tasko `permesiloj-kontroli` (`cargo install cargo-deny --locked`) |

Instalu la celajn triopojn:

```sh
rustup target add x86_64-unknown-none
rustup target add aarch64-unknown-none   # por la ARM-ŝpuro
```

### Konstrui kaj ruli

```sh
just konstrui-ciujn     # konstruas la tutan Cargo-laborlokon (kerno, libmikerno, horizonto)
just konstrui         # konstruas nur la kernon (= konstrui-kerno)
just ruli           # QEMU -kernel, rekte per la PVH-noto (sen ISO)
just iso           # kreas mikerno.iso per Limine + Multiboot2
just ruli-iso       # rulas la ISO-on (QEMU-fenestro montras la Limine-menuon)
just konstrui-arm     # arm64: plata `Image` + `horizonto`
just ruli-arm       # arm64: konstruas kaj startigas la platan `Image`
just iso-arm       # UEFI-bootebla `mikerno-arm.iso` (Limine + plata `Image`)
just ruli-iso-arm   # konstruas kaj rulas `mikerno-arm.iso` per AAVMF
just kontroli        # unu-testoj, formato, statika analizo (ambaŭ arkoj) kaj konstruo
just testoj         # unu-testoj de `kernologiko` sur la gastiganto (sen QEMU)
just miri           # la samaj unu-testoj sub Miri (postulas noktan Rust)
just permesiloj     # permesila kaj dependaĵa kontrolo laŭ `deny.toml`
```

`just ruli-arm` bezonas la celan triopon `aarch64-unknown-none`
(`rustup target add aarch64-unknown-none`) kaj `qemu-system-aarch64`.
En `0.5.0` la arm64-kerno eniras `EL0` per `eret`,
plenumas `svc #0`, posedas per-procezajn `TTBR0`-spacojn kaj rulas la saman
IK-scenaron kiel `x86_64`; `horizonto` venas kiel `-initrd` aŭ kiel
ISO-modulo. La frambufo venas el `ramfb` per `fw_cfg` (1280×800, `XR24`),
`PXN`/`UXN`/`PAN` estas aktivaj, kaj la piksela fiksaĵo donas la saman
haketon kiel la `x86_64`-ISO. Esceptoj el `EL0` — abortoj kaj nekonata kaŭzo
(ekz. `udf`) — mortigas nur la kulpan procezon.
`just iso-arm` krome postulas `xorriso`, `mkfs.fat` (dosfstools) kaj
`mmd`/`mcopy` (mtools); `just ruli-iso-arm` bezonas UEFI-firmwaren
(`qemu-efi-aarch64`, alirebla kiel `QEMU_EFI.fd` aŭ per `QEMU_ARM_EFI`).
La ISO-vojo implicite uzas `cortex-a76` (AAVMF 2025.11 blokiĝas sub TCG kun
`-cpu max`); `QEMU_ARM_CPU` anstataŭigas la elekton.

Ekvivalentaj rektaj komandoj:

```sh
cargo build --workspace --release
cargo build -p mikerno --release
cargo run -p mikerno --release          # uzas la agordon `runner` el .cargo/config.toml
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

La DI-matrico praŝarĝas la saman lanĉilon kun `QEMU_MASINO` (`q35` aŭ `virt`),
`QEMU_MEMORO` (`128M` aŭ `256M`) kaj, por x86_64, `QEMU_UEFI=1` (OVMF). Loke
la saman kontrolon ebligas:

```sh
QEMU_DISPLAY=none tools/kontroli-qemu.sh 60 bash tools/run-qemu.sh iso
QEMU_DISPLAY=none QEMU_UEFI=1 QEMU_MASINO=q35 \
    tools/kontroli-qemu.sh 60 bash tools/run-qemu.sh iso
```

La kontrolilo atendas la Ligo-prompton `ligo:~>`, sendas `fini` kaj postulas
`[kerno] fina stato` sen `MALBONE`.

`just ruli-arm` kaj `just ruli-iso-arm` startigas bildojn el la `arm64`-branĉo:
la unua lanĉas la antaŭe konstruitan kernon (`mikerno.img`), la dua la ISO-on en
QEMU kun UEFI. Ĉe la ISO la fenestro montras la UEFI/Limine-ekranon; la kerna
eligo venas per la seria konzolo en la terminalo, ĉar la arm64-kerno ankoraŭ ne
havas frambufon. Sen WSLg uzu `QEMU_DISPLAY=none just ruli-iso-arm`. La bildoj
estas kreataj per `just konstrui-arm` kaj `just iso-arm` en la `arm64`-branĉo;
`main` nur startigas ilin.

> **Atentu:** `mikerno.iso` kaj `iso_root/` estas ignorataj de git; DI
> konstruas la ISO-ojn kiel artefaktojn `iso-bildoj`. Post kerna ŝanĝo rerulu
> `just iso` por loka ISO; alie ĝi enhavos malnovan bildon. Tio efektive
> okazis dum la kontrolo de 2026-11-12 (la ISO montris version `v0.1.1`, dum
> la fonto estis `v0.1.0`).

---

## Arĥitekturo

La Cargo-laborloko havas kvin kestojn:

| Kesto | Speco | Rolo |
| --- | --- | --- |
| `mikerno` | `no_std` binara | La kerno: praŝarĝo, memoro, interrompoj, planisto, IK, konzolo |
| `kernologiko` | `no_std` biblioteko | Arkitekture sendependa logiko (memoro, adresoj, vicoj, SNDS) kun gastigantaj unu-testoj |
| `libmikerno` | `no_std` biblioteko | Komunaj `#[repr(C)]`-difinoj: mesaĝoj, kapabloj, sistemvokaj numeroj |
| `horizonto` | `no_std` binara | La unua uzanta procezo, ligita jam super 4 GiB (`0x1_0000_0000`) |
| `saluto` | `no_std` binara | Malgranda testoproceso lanĉata el SNDS per `ruli` |> **Noto:** la tuta Cargo-laborloko uzas `code-model=large`, ĉar `horizonto` estas
> ligita super 4 GiB. La kerno estas ligita kun `relocation-model=pic` (ET_DYN)
> por KASLR; `horizonto` ligas kun `-no-pie` kaj restas ET_EXEC kun statikaj
> relokoj. Neniu dinamika ligilo aŭ ekstera dependaĵo aperas.

Dependa grafo:

```text
horizonto ──> libmikerno
mikerno    ──> kernologiko ──> libmikerno
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
  · nova PML4 (4 KiB paĝoj), identeca mapo, frambufo (PCD)
  · modulaj regionoj identece mapitaj, CR3 ŝalto, konzolo, TSS/RSP0
  · APIC + temporizilo (100 Hz)
  · reuzoprovo: 10× krei/munti/detrui procezan spacon, liberaj kadroj egalaj
  · IK-testo, 3 testaj fadenoj (procezo 0), planisto
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
   32-bitan frambufon (etikedo 5). `EAX` estas `0x36d7_6289`, `EBX` montras
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
elektas hazardan 2 MiB-aliniiĝintan bazon en uzebla ĉefmemoro (aŭ `RDRAND`, aŭ
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
- la frambufon (`SKRIBA_SENKAŜE`: skribebla, neplenumebla, `PCD`),
- la APIC-MMIO-on (`SKRIBA`).

`EFER.NXE` kaj `CR0.WP` estas ŝaltitaj antaŭ ol oni uzas la biton 63 aŭ
nurlegajn mapojn: `CR0.WP` devigas ankaŭ Ring 0 respekti la biton `W`, do
nurlega paĝo estas vera barilo, ne konsilo. `mapigi` rifuzas nealiniiĝintajn
adresojn kaj jam mapitajn foliojn (`Konflikto`); `traduki` komprenas ankaŭ 2 MiB
grandegajn paĝojn. Neniu paĝo estas samtempe skribebla kaj plenumebla. La
praŝarĝa modulo estas mapita nurlega antaŭ la identeca mapado, do ĝiaj paĝoj
restas `R`+`NX` anstataŭ fariĝi skribebla ĉefmemoro. `permesoj(adreso)` legas la
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
  sistemon. Escepto `#PF` (14), `#GP` (13), `#DE` (0), `#OF` (4) aŭ `#UD` (6)
  el Ring 3 apartenas al la uzanta
  procezo: la kerno printas `[eraro] procezo N …` kun `rip`/`erarkodo`/`CR2`,
  vokas `planisto::morti()` (kiu detruas la procezan spacon) kaj daŭrigas kun
  la sekva fadeno anstataŭ `halti()`. `horizonto` provas tion per propra reĝimo,
  kiu plenumas `ud2`; la kerna raporto konfirmas `#UD-izolado (bone)`.
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
  `rip`, `rflags`) per `#[repr(C)]` `arkitekturo::Kunteksto`; la unua eniro
  uzas artefaritan stakon kun `fadeno_trampolino`, kiu ŝaltas interrompojn kaj
  transdonas la fadenan identigilon en la unua argumento;
- ĉiu fadeno havas 16 KiB stakon; la testaj stakoj estas statikaj;
- antaŭ ĉiu `ŝanĝi` la planisto aktivigas la celan procezon: se la fizika
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

`ŝanĝi` revenas nur kiam la malnova kunteksto estas planita denove. La ripoza
kunteksto konservas sian interrompan kadron sur la stako, do `planumi` povas
reveni tra la dispetĉo kaj `iretq` kiam la ripoza fadeno rekaptas la procesoron.
La `CR3`-ŝalto okazas antaŭ la registra konservo: la fadenaj kontekstoj, la
planisto kaj la kernaj stakoj vivas en la kerna bildo, kiun ĉiu proceza spaco
kunhavas.

---

## IK kaj kapabloj

`kerno/ik.rs` difinas minimuman mesaĝan IK-on:

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
- plena vico **premegas la sendanton**: ĝi atendas en la vico de la finpunkto
  (bitmasko `sendantoj`, bito `id − 1`) kaj vekiĝas, kiam `preni_de` liberigas
  lokon; nur la ripoza kunteksto ne povas bloki kaj ricevas `-4`; revoko de la
  finpunkto vekas ĉiujn atendantojn per `-32`;
- `SENDI` redonas `-90` (`TRO_LONGA_MESAGO`), se `Mesago.longo` superas
  `MAKSA_MESAGA_LONGO`;
- `IkStatistiko` nombras la enmetitajn kaj prenitajn mesaĝojn, la premregojn,
  la revokojn kaj la tempolimajn vekojn; la starta scenaro printas ĝin apud la
  planista raporto;
- `RICEVI_TEMPO` (9) kaj `SENDI_TEMPO` (10) interpretas `r10` laŭ C15:
  `u32::MAX` = senfina atendo, `0` = nebloka (`-11` se la operacio ne eblas),
  alie `ceil(ms/10)` tikoj kaj veko per `-110`; la limdato estas absoluta kaj
  fiksiĝas ĉe la voko, do ripeta planista veko ne plilongigas ĝin; `rdx` de
  `RICEVI_TEMPO` estas la fonta filtro (`0` = ajna sendinto);
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
- `registri_fadenon` instalas la transiran `KOMENCA_TENILO` (`0x100`: fendo
  `0`, unua generacio) por ĉiu procezo, do `horizonto` povas uzi sian
  defaŭltan finpunkton sen antaŭa `KREI_FINPUNKTON`;
- proceza detruo (ankaŭ la antaŭaj erarvojoj de `prepari_uzantan_procezon`)
  vokas `ik::purigi_procezon`: ĝi malpliigas la referencojn de fremdaj
  finpunktoj, liberigas ĉiujn kapablofendojn de la procezo kaj revokas la
  finpunktojn, kiujn ĝi posedis;
- la kernaj fadenoj (procezo `0`) ne havas kapablotabelon; ili uzas la
  internajn `sendi_mesagon`/`ricevi_mesagon` (C8).

Ĉiuj operacioj tenas interrompojn malŝaltitaj per `InterrompaGardo`, kiu
restarigas la antaŭan `RFLAGS.IF`-staton ĉe `Drop`.

Nuna limigo: unu finpunkto memoras nur unu blokitan ricevanton; `fork`/`exec`-
distribuo de kapabloj venas en la sekvaj fazoj (M17).

---

## Eligo: COM1 kaj frambufo

- **COM1** (`seria.rs`): 16550 UART ĉe `0x3F8`, 115200 baudoj, 8N1, demandata
  reĝimo, `\n` → `\r\n`. Ĉi tio estas la arkitektura konzolo kaj ĉiam funkcias.
- **Frambufo** (`ekrano/`): 32-bita Linuksa frambufo el Multiboot2, desegnita
  per antaŭe rasterigitaj bitmapaj tiparoj derivitaj el **DejaVu Sans Mono**.
  Sur `arm64` la frambufon kreas la kerno mem: `fw_cfg` liveras la dosieron
  `etc/ramfb`, kaj `ramfb` montras kontiguajn ĉefmemorajn kadrojn (1280×800, `XR24`,
  paŝo 5120).
  Ekzistas kvar strekoj — 8×16, 12×24, 16×32 kaj 32×64 — kaj la konzolo elektas
  la plej grandan, kiu donas almenaŭ 80 kolumnojn kaj 24 vicojn, kun nombro da
  vicoj plej proksima al 33. Ĉiu streko enhavas la 95 ASCII-signojn kaj 16
  aldonajn glifojn: `ĉ Ĉ ĝ Ĝ ĥ Ĥ ĵ Ĵ ŝ Ŝ ŭ Ŭ` kaj `« » ‹ ›`. Aliaj
  ne-ASCII-signoj fariĝas `?`. Postuloj: bazo 4-bajte aliniiĝinta, `bitoj_po_pikselo == 32`,
  larĝo ≥ 320, alto ≥ 32, `paŝo ≥ larĝo × 4`. La seriala eligo restas plena
  UTF-8. Kiam la tekstfluo atingas la fundon, la ekrano estas malplenigita (ne
  vere rulumita).

  Tipika elekto: 800×600 → 8×16 (100×37), 1280×800 → 12×24 (106×33),
  1920×1080 → 16×32 (120×33), 2560×1440 → 16×32 (160×45), 3840×2160 → 32×64
  (120×33). La strekoj estas generataj per `tools/font/generi.py`; la fonto
  mem ne estas enŝovita, nur la bitaj tabeloj.
- Se frambufo mankas (Xen-PVH aŭ malnova firmvaro), la konzolo restas
  malŝaltita kaj la eligo okazas nur sur COM1.

---

## Uzantspaco

### Komuna kontrakto

`libmikerno` fiksas la komunan kontrakton:

- `Mesago` (`#[repr(C)]`, 64-bajta datumaro, `tipo`, `sendinto`, `ricevonto`),
- `FinaPunkto`, `Kapablo`, `Permesoj`, `Tenilo`, `KOMENCA_TENILO`,
- la dudek sistemvokaj numeroj de ABI 2 (vidu la referencan tabelon sube),
- `Tenilo` — 32-bita kapabla tenilo (8 bitoj da indekso + 24 bitoj da
  generacio, generacio ekde `1`); la novaj erar-kodoj estas `-9`, `-11`, `-28`,
  `-32`, `-90` kaj `-110`,
- registra ABI: `rax` = numero, `rdi`/`rsi`/`rdx`/`r10` = argumentoj, `rax` =
  rezulto; la voko estas `int 0x80`;
- starta ABI: `horizonto_komenco(reĝimo: u64, identigilo: u64)` — la kerno
  transdonas la reĝimon en `RDI` (`REĜIMO_SEKRETO = 2`, `REĜIMO_SPIONO = 3`,
  `REĜIMO_EXEC = 4`, `REĜIMO_FONO = 5`, `REĜIMO_ŜELO = 6`,
  `REĜIMO_NEVALIDA = 7`) kaj la procezan identigilon en `RSI`.

#### Referenca tabelo de ABI 2

La dudek voknumeroj, iliaj argumentoj, rezultoj kaj erar-kodoj. Ĉiuj numeroj
venas el `libmikerno::syscall` kaj estas kontrolataj kontraŭ la konstantoj per
`tools/dokumentaro.sh`. La kvar argumentoj `a0`–`a3` mapiĝas al
`rdi`/`rsi`/`rdx`/`r10` sur `x86_64` kaj al `x0`–`x3` sur `arm64`.

**ABI 2 estas frostigita ekde 0.9.1 (M19 Fazo 2):** post 1.0 ajna nekongrua
ŝanĝo — nova numero, argumento aŭ erar-kodo — postulas `ABI_VERSIO = 3`.

| N-ro | Konstanto | Argumentoj (`a0`–`a3`) | Rezulto | Eraroj |
| --- | --- | --- | --- | --- |
| 1 | `SYS_SENDI` = 1 | `tenilo`, `mesaĝo` | `0` | `-1`, `-3`, `-9`, `-32`, `-90` |
| 2 | `SYS_RICEVI` = 2 | `tenilo`, `celo` | `0` | `-1`, `-9`, `-32` |
| 3 | `SYS_SKRIBI` = 3 | `bufro` (UTF-8), `longo` | `0` | `-1` |
| 4 | `SYS_FINI` = 4 | `elirkodo` | neniam revenas | — |
| 5 | `SYS_CEDI` = 5 | — | `0` | — |
| 6 | `SYS_KREI_FINPUNKTON` = 6 | `permesoj` | tenilo (> 0) | `-1`, `-28` |
| 7 | `SYS_DONI` = 7 | `fonto`, `celo` (procezo), `permesoj` | tenilo (> 0) | `-1`, `-3`, `-9`, `-28` |
| 8 | `SYS_REVOKI` = 8 | `tenilo`, `0` (rezervita) | `0` | `-1`, `-9` |
| 9 | `SYS_RICEVI_TEMPO` = 9 | `tenilo`, `celo`, `fonto` (`0` = ajna), `tempolimo` (ms; `SENFINA_TEMPOLIMO`) | `0` | `-1`, `-9`, `-11`, `-32`, `-110` |
| 10 | `SYS_SENDI_TEMPO` = 10 | `tenilo`, `mesaĝo`, `0` (rezervita), `tempolimo` (ms) | `0` | `-1`, `-9`, `-11`, `-32`, `-90`, `-110` |
| 11 | `SYS_FERMI` = 11 | `tenilo` | `0` | `-9` |
| 12 | `SYS_DORMI` = 12 | `daŭro` (ms) | `0` | `-1` |
| 13 | `SYS_BRK` = 13 | `nova limo` (`0` = demandi) | kupo-limo | `-1` |
| 14 | `SYS_EXEC` = 14 | `vojo`, `longo` | neniam revenas (sukceso) | `-1` |
| 15 | `SYS_ENIGI` = 15 | — | bajto (0–255) | `-1` |
| 16 | `SYS_LISTI` = 16 | `vojo`, `longo`, `celo`, `grando` | nombro da `Dosierero`-j | `-1`, `-17`, `-28` |
| 17 | `SYS_LEGI` = 17 | `vojo`, `longo`, `celo`, `grando` | nombro da bajtoj | `-1`, `-17`, `-28` |
| 18 | `SYS_PROCEZOJ` = 18 | `celo`, `grando` | nombro da `ProcezoRekordo`-j | `-1` |
| 19 | `SYS_MORTIGI` = 19 | `pid` | `0` | `-1`, `-13`, `-17` |
| 20 | `SYS_LANĜI` = 20 | `vojo`, `longo` | nova `pid` (> 0) | `-1`, `-17`, `-28` |

| Kodo | Konstanto | Signifo |
| --- | --- | --- |
| `-1` | — | malĝusta argumento, mankanta permeso aŭ malsukcesa operacio |
| `-2` | — | nekonata voknumero |
| `-3` | — | nevalida celloko: la celo de mesaĝo aŭ kapablo ne estas valida fadeno aŭ procezo |
| `-4` | — | la mesaĝujo plenas sur la neblokebla kerna vojo |
| `-9` | `NEVALIDA_TENILO` | la tenilo ne validas, la generacio malakordas aŭ la kapablo revokiĝis |
| `-11` | `PROVU_DENOVE` | nebloka voko trovis malplenan vicon aŭ plenan mesaĝujon |
| `-13` | `NEPERMITITA` | la operacio estas malpermesata; la nukleo kaj la ŝelo ne mortigeblas |
| `-17` | `NE_TROVITA` | la petita dosiero aŭ procezo ne ekzistas |
| `-28` | `TABELO_PLENA` | la finpunkta aŭ kapabla tabelo plenas; ankaŭ la cel-bufro estas tro malgranda |
| `-32` | `FINPUNKTO_REVOKITA` | la finpunkto revokiĝis aŭ ĝia posedanto finiĝis |
| `-90` | `TRO_LONGA_MESAGO` | la mesaĝa longo superas `MAKSA_MESAGA_LONGO` |
| `-110` | `TEMPOLIMO` | la tempolimo pasis antaŭ ol la operacio eblis |

La kvar unuaj kodoj havas konstanton nur en la kerna dispetĉo
(`MALĜUSTA_ARG`, `NEKONATA_VOKO`, `NEVALIDA_CELLOKO`, `MESAGUJO_PLENA`); la
ceteraj estas re-eksportitaj de `libmikerno::syscall`. La kompila tempo fiksas
ankaŭ la aranĝon: `size_of::<Tenilo>() == 4`, `size_of::<Mesago>() == 96` kaj
`ABI_VERSIO == 2`, kaj `kernologiko` ripetas tiun kontrolon per unu-testoj.

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
`limine.conf`, aŭ la PVH-modlisto el `-initrd` (``just ruli`` aldonas ĝin
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
privatigis (kampo `privataj_pdptoj`, trajta aliro `privataj_subtabeloj`). La
indekso `0` ĉiam estas privata: ĝi portas
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
`extern "C" fn horizonto_komenco(reĝimo: u64, identigilo: u64)` (`REĜIMO_SEKRETO`
kaj `REĜIMO_SPIONO` el `libmikerno::starto`). La `int 0x80`-pordego havas DPL3,
do Ring 3 rajtas voki `SKRIBI`, `CEDI` kaj `FINI`; la procesoro ŝaltas al `TSS.RSP0`
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
spiono kaj la procesoro liveras `#PF` kun `erarkodo=0x4` (`U=1`, `P=0`). La kerno
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
`kerno/sistemvoko.rs` efektivigas ĉiujn numerojn `1`–`20` el
`libmikerno::syscall`: la mesaĝaj kaj kapablaj vokoj (`SENDI`, `RICEVI`,
`KREI_FINPUNKTON`, `DONI`, `REVOKI`, `FERMI`), la tempaj vokoj (`RICEVI_TEMPO`,
`SENDI_TEMPO`, `DORMI`), la procezaj (`FINI`, `CEDI`, `MORTIGI`, `LANĜI`,
`EXEC`), la konsolaj (`SKRIBI`, `ENIGI`) kaj la SNDS/procezaj legiloj
(`LISTI`, `LEGI`, `PROCEZOJ`), plus `BRK` por la kupa limo. Ring 3-bufroj estas
ne-nulaj, limigitaj kaj ene de la anoncitaj bilda kaj stakaj regionoj (la
limoj venas el la nuna procezo mem, `Procezo::bildaj_limoj()`); la tenilaj
vokoj validigas la kapablon en du niveloj (fendo + finpunkta generacio);
`SYS_FINI` vokas `planisto::morti()` kaj neniam revenas.
Ring 0-memtesto pruvas la vojon per `int 0x80` en la unua fadeno (`SYS_CEDI`
tie vere cedas la procesoron).

### Ligo

`horizonto` fariĝas la objekta ŝelo Ligo, kiam la kerno lanĉas ĝin per
`REĜIMO_ŜELO`. La ŝelo startas en `/hejmo`; la invito montras `~` anstataŭ tiu
prefikso (`ligo:~>`, `ligo:~/notoj>` aŭ `ligo:/…>`). La komandoj estas
Esperantaj kaj puraj ASCII. La deviga priskribo, la gramatiko de la dukto kaj
ekzemploj estas en `docs/FLUOU.md` §8.

| Komando | Signifo |
| --- | --- |
| `helpu` | Montri la komandtabelon |
| `listi [vojo]` | Listigi dosierojn (`SYS_LISTI`) |
| `legi <dosiero>` | Legi dosieron (`SYS_LEGI`) |
| `iri [vojo]` | Ŝanĝi la nunan dosierujon (sen argumento: `/hejmo`) |
| `kie` | Montri la plenan vojon de la nuna dosierujo |
| `procezoj` | Montri la procezojn (`SYS_PROCEZOJ`) |
| `mortigi <pid>` | Mortigi procezon (`SYS_MORTIGI`; `pid ≤ 1` estas protektita) |
| `ruli <dosiero>` | Lanĉi novan ELF el SNDS (`SYS_LANĜI`) |
| `fini` | Fini la ŝelon (`SYS_FINI`); `Ctrl+D` en malplena linio same |
| `eldiri <teksto>` | Krei mesaĝan rekordon |
| `pri` | Montri sistemajn informojn |
| `temo <malluma\|hela>` | Ŝanĝi la temon |
| `akcento <nomo>` | Ŝanĝi la akcenton |
| `eniro` | Malfermi Eniron |
| `filtri`, `ordigi`, `unua`, `sumi`, `nombri`, `montri` | Stadioj de la rekorda dukto |
| `teksto` | Transiri al la eksplicita teksta limo |
| `purigi` | Malplenigi la terminalon |

Vojoj estas komprenataj kiel en Linux: `~` estas `/hejmo`, relativaj vojoj
komenciĝas en la nuna dosierujo, kaj `..` neniam eliras la radikon. La solvilo
loĝas en `kernologiko::vojo`, do la ŝelo kaj ĝiaj testoj uzas la samajn
regulojn.

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
| PVH (`-kernel` + `-initrd`) | 7 regionoj, 255 MiB uzeblaj, sen frambufo, `KASLR: kerna bazo 0x…` (malsama ĉiu starto), reuzoprovo `10 cikloj, 64823 kadroj antaŭ, 64823 post`, procezoj `4` kaj `5` samtempe pretaj, sama sekreta adreso `0x200000000`, id `4`/`5`, spiono `#PF` ĉe `0x200001000` (`erarkodo=0x4`), viktimo pluvivas kaj finas per `SYS_FINI`, `[ik] purigo de procezo 5/4`, `34`/`35` kadroj liberigitaj, `izolado (bone): 64823 kadroj antaŭ, 64823 post`, 1000 tikoj, 5 fadenoj mortintaj |
| Limine/Multiboot2 (ISO, BIOS) | frambufo `1280x800, 32 bitoj/pikselo`, `KASLR: kerna bazo 0x…`, reuzoprovo `10 cikloj, 64821 kadroj antaŭ, 64821 post`, procezoj `4` kaj `5` samtempe pretaj, sama sekreta adreso `0x200000000`, id `4`/`5`, spiono `#PF` ĉe `0x200001000` (`erarkodo=0x4`), viktimo pluvivas kaj finas per `SYS_FINI`, `[ik] purigo de procezo 5/4`, `34`/`35` kadroj liberigitaj, `izolado (bone): 64821 kadroj antaŭ, 64821 post`, 1000 tikoj, 5 fadenoj mortintaj |
| Limine (ISO, UEFI/OVMF) | sama scenaro; `KASLR: kerna bazo 0x…`, liberaj kadroj `63367` antaŭ/post, frambufo `1280x800, 32 bitoj/pikselo`, 1000 tikoj, 5 fadenoj mortintaj |

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

`just kontroli` unue kontrolas la dokumentan version per `tools/versio.sh
--kontroli` kaj poste vokas
`cargo kani -p mikerno -Z unstable-options --ignore-global-asm`; la 29 pruvoj
(6 pri la kadro-alokilo, 20 pri la IK) pruviĝis verde per Kani 0.68.0.

### Kion vi devus vidi

Ĉe sukcesa starto la seriala konzolo montras (mallongigite):

```text
[MiKerno] Saluton, mi estas MiKerno v0.9.3!
[kerno] KASLR: kerna bazo 0x… (deŝovo 0x…); la bildo venis de 0x100000
[kerno] SMEP subtenata: jes
[kerno] SMAP subtenata: jes
[kerno] SMEP ŝaltita; Ring 0 ne povas plenumi uzantajn paĝojn.
[kerno] SMAP ŝaltita; Ring 0 legas uzantajn bufrojn nur en STAC-fenestroj.
[memoro] AC (bone): fermita=true, malfermita=true, interna=true, post_interna=true, fermita_post=true, ŝtelita=true, purigita=true
[kerno] gardoj (bone): malŝalto=true, interna=true, restarigo=true (IF antaŭe false)
[sistemvoko] pordego 0x80 (DPL3) instalita
[kerno] memoro: 7 regionoj, 255 MiB uzeblaj
[memoro] liberigo (bone): 64956 kadroj antaŭ, 64956 post
[memoro] W^X (bone): .text RX, .rodata R NX, .data/.bss RW NX, modulo R NX
[memoro] CR3 (bone): PCID=malŝaltita, aktiva=0x196000, kongrua=true, aliniita=true
[kerno] frambufo ne havebla; nur COM1    (PVH; ISO montras 1280x800)
[tss] TR ŝargita; kernostako pinto 0x123ce10
[temporizilo] kalibrado: ... kalkuloj/s, komenca ...
[kerno] APIC 0 kaj paĝado pretaj; temporizilo 100 Hz
[procezo] 6 detruita: 1 privataj PDPT-oj, PML4 0x213000, 27 kadroj liberigitaj
... (10 reuzaj cikloj)
[memoro] reuzo (bone): 10 cikloj, 64823 kadroj antaŭ, 64823 post
[ik] tabeloj (bone): kreitaj=13, plena=true, fermitaj=13, duobla_fermo=true, nova_generacio=true, malnova_tenilo=true, kerno=true
[ik] rajtoj (bone): sendi per RICEVI-nura → -1, ricevi per SENDI-nura → -1
[ik] izolado de kapabloj (bone): procezo 7 uzas tenilon de procezo 6 → -9
[ik] dono (bone): rajtoj 0b011 → 0b010, eskalado 0b100 → -1
[ik] revoko (bone): sen_revoki=-1, revoko=0, donita=-9, fermo=-9, duobla=-9
[kerno] IK: mesaĝo sendita al fadeno 2
[kerno] planisto ekas: 4 fadenoj, antaŭokupo ĉiu 10 ms
saluton el Ring 0 tra int 0x80
[sistemvoko] Ring 0-memtesto: skribo=0, nekonata=-2, nula=-1, troa=-1, cedo=0, krei=-28, doni=-9, fermi=-9, revoki=-9, revok_argumento=-1, tro_longa=-90, sen_eoi=true
...
[fadeno 1] premrego (bone): 17 mesaĝoj senditaj; la lasta atendis liberan lokon
[fadeno 1] kuranta (fojo 1)
[fadeno 2] premrego (bone): 17 mesaĝoj, idento sendinto=1, ricevonto=2
...
[fadeno 2] sendblokada revoko (bone): enmetitaj=16, kodo=-32
[fadeno 4] tempolimoj (bone): nebloka=-11 (0 tikoj), tempolimo=-110 (5 tikoj), mesaĝo=0/tipo 90, senfina=0/tipo 91
...
[fadeno 1] revokis la defaŭltan finpunkton de fadeno 3
[fadeno 3] revoka veko (bone): kodo -32
[fadeno 3] revokis finpunkton 1; vekas la blokitan sendanton
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
[ring3] procezo 5: tempolimoj (bone): nebloka=-11, tempolimo=-110, sendo=0, senfina=0, troa=-1
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
[ik] purigo de procezo 5: 1 kapabloj liberigitaj, 1 finpunktoj revokitaj
[planisto] fadeno 5 finiĝis
[procezo] 5 detruita: 1 privataj PDPT-oj, PML4 0x236000, 34 kadroj liberigitaj
[ring3] procezo 4: laboras (fojo 2)
...
[ring3] procezo 4: laboras (fojo 6)
[ring3] procezo 4: pluvivas post la spiono; finas per SYS_FINI
[ik] purigo de procezo 4: 1 kapabloj liberigitaj, 1 finpunktoj revokitaj
[planisto] fadeno 4 finiĝis
[procezo] 4 detruita: 1 privataj PDPT-oj, PML4 0x213000, 35 kadroj liberigitaj
[apic] EOI (bone): 1000 konfirmoj, 1000 tikoj
[memoro] izolado (bone): 64809 kadroj antaŭ, 64809 post
[kerno] planisto revenis al la ripoza fadeno
[ik] statistiko (bone): senditaj=41, ricevitaj=25, premregoj=2, revokoj=7, tempolimoj=3
...
[kerno] fina stato: 6 mortintaj, 0 pretaj, 0 blokitaj; 1000 tikoj
[ekrano] piksela fiksaĵo 1280x800, 32 bitoj/pikselo, tiparo 12x24; haketo 0xecf1f6dbe1421575    (ISO; PVH diras «preterlasita»)
[memoro] SMAP-provo: Ring 0 provos legi la uzantan folion ĉe 0x100000000
[eraro] procezo 0: SMAP-malobservo ĉe 0x0000000100000000 (erarkodo=0x1)
```

La piksela fiksaĵo estas la sama sur `arm64`: `just ruli-arm` kaj
`just ruli-iso-arm` presas `[ekrano] piksela fiksaĵo 1280x800, 32 bitoj/pikselo, tiparo
12x24; haketo 0xecf1f6dbe1421575`, kaj ankaŭ `[memoro] PAN-provo: EL1 provos
legi la uzantan folion ĉe 0x100000000` sekvata de `[eraro] PAN-malobservo`.
Ambaŭ arkoj uzas la saman desegnaĵon (gradiento kaj teksto), la saman tiparon
kaj la saman FNV-1a-haketon, do la bildo estas komparebla bajto post bajto.

---

## Testa scenaro

`kerno::ĉefa` kreas tri fadenojn kun statikaj 16 KiB-stakoj:

1. **Fadeno 2** tuj blokiĝas en `ricevi_mesagon(0)`; la praŝarĝo sendas al ĝi
   mesaĝon `tipo 1` («saluton el la praŝarĝo»), kiu vekas ĝin.
2. **Fadeno 1** rulas la Ring 0-memteston (inkluzive `tro_longa=-90`) kaj plenigas
   la vicon de fadeno 2 per 16 mesaĝoj; la 17-a **blokas** ĝis fadeno 2 liberigas
   lokon. Fadeno 2 malplenigas la vicon kaj kontrolas la identecon: `sendinto=1`
   (ne la falsa `999`) kaj `ricevonto=2`.
3. **Fadeno 2** plenigas la vicon de fadeno 1 kaj blokas sur la 17-a sendo;
   fadeno 3 blokiĝas sur sia propra defaŭlta finpunkto.
4. En la tria fojo **fadeno 1** revokas la defaŭltan finpunkton de fadeno 3 per
   `revoki_defaŭltan`; fadeno 3 vekiĝas kun `-32` (`Revokita`) kaj revokas
   finpunkton 1, kio vekas la blokitan sendanton (fadeno 2) ankaŭ per `-32`.
5. Ĉiuj tri fadenoj dek fojojn printas sian progreson kaj cedas la procesoron;
   post la lasta fojo ili finiĝas. La kerno printas `[ik] statistiko` kaj la
   planisto revenas al la ripoza kunteksto.
6. Antaŭ la uzantaj procezoj la praŝarĝo rulas la reuzoprovo: ĝi kreas la
   procezan spacon kun la sama bildo kaj stako, detruas ĝin kaj ripetas dek
   fojojn. La nombro de liberaj kadroj devas esti egala antaŭ kaj post.
7. La praŝarĝo preparas **du** procezojn antaŭ iu ajn planista voko, dum
   interrompoj restas malŝaltitaj: procezo `4` (`REĜIMO_SEKRETO`, la viktimo)
   kaj procezo `5` (`REĜIMO_SPIONO`). Ĉiu ricevas sekretan paĝon ĉe la sama
   virtuala adreso `0x2_0000_0000`, sed malsaman kadron enhavantan sian propran
   identigilon; la viktimo krome mapas duan paĝon ĉe `0x2_0000_1000`. Nur poste
   `planisto::lanĉi()` ekas; la du procezoj tiel interplektiĝas.
8. Ĉiu procezo ekas en Ring 3 per `iretq` (`RDI`=reĝimo, `RSI`=identigilo) kaj
   legis sian sekretan paĝon: `4` vidas `id 4`, `5` vidas `id 5`. Tio pruvas, ke
   la sama virtuala adreso montras malsamajn fizikajn kadrojn.
9. Procezo `5` legas la duan paĝon `0x2_0000_1000`, kiun ĝia spaco ne havas. La
   `#PF` el Ring 3 kun `erarkodo=0x4` (`U=1`, `P=0`) estas kaptita; la kerno
   printas `[eraro] procezo 5 …`, mortigas la procezon kaj `detruu` liberigas
   ĝian tutan spacon. La planisto elektas procezon `4`; neniu `halti()` okazas.
10. Procezo `4` pluvivas la spionon, printas la rondojn `2`–`6` kaj eliras per
    `fini(0)`; `SYS_FINI` detruas ankaŭ ĝian spacon. Antaŭ la detruo
    `ik::purigi_procezon` liberigas ĝiajn kapablojn kaj revokas ĝian
    finpunkton (`[ik] purigo de procezo 4 …`); la sama okazas por la spiono.
    La praŝarĝo logas `[memoro] izolado (bone)` kaj la liberaj kadroj egalas
    antaŭ kaj post.
11. La praŝarĝo observas la ceterajn 1000 tikojn (≈10 s ĉe 100 Hz), printas la
    finan staton `5 mortintaj` kaj haltas per `hlt`.

---

## Strukturo de la dosieroj

```text
.
├── Cargo.toml                  # laborloko (membroj, versio, statika analizo)
├── deny.toml                   # permesilaj reguloj de `cargo-deny`
├── justfile                    # konstrui / ruli / iso / kontroli / permesiloj
├── .cargo/config.toml          # cela triopo, rust-lld, QEMU-lanĉilo
├── mikerno/
│   ├── Cargo.toml
│   ├── build.rs                # liveras boot/linker.ld nur al la kerno
│   ├── snds/                   # bilda enhavo: /bonveno.txt, /hejmo, /notoj
│   ├── boot/
│   │   ├── linker.ld           # 1 MiB, ET_DYN, RELATIVE-relokoj
│   │   └── limine.conf         # ISO-menuo (Multiboot2)
│   └── src/
│       ├── main.rs             # enirpunkto, `skribu!`, paniko
│       ├── kunhavigita.rs      # UnsafeCell + Sync (unuprocesora)
│       ├── arkitekturo/
│       │   ├── mod.rs          # platforma fasado
│       │   ├── komuna.rs       # SistemvokaKadro, EsceptoTipo, FirmigaRaporto
│       │   └── x86_64/
│       │       ├── mod.rs      # pravalorizi (seria, gdt, idt, nxe)
│       │       ├── eniro.rs    # 32→64-bita ŝelko, MB2-kapo, PVH-noto
│       │       ├── mb2.rs      # Multiboot2-analizo
│       │       ├── pvh.rs      # Xen-PVH-analizo
│       │       ├── gdt.rs      # GDT + TSS-priskribo
│       │       ├── idt.rs      # IDT kaj interrompaj ŝablonoj
│       │       ├── interrompo.rs # InterrompaKadro, escepta klasifiko
│       │       ├── tss.rs      # RSP0, 16 KiB kerna stako
│       │       ├── apic.rs     # xAPIC MMIO
│       │       ├── temporizilo.rs # PIT-kalibrita 100 Hz
│       │       ├── pagado.rs  # PML4/PDPT/PD/PT, CR3
│       │       ├── planado.rs  # kuntekstŝanĝo (asm)
│       │       ├── sistemvoko.rs # 0x80-dispetĉilo el la kadro
│       │       ├── procesoro.rs# hlt, cli/sti, CR2
│       │       ├── uzanta.rs   # `iretq` al Ring 3
│       │       └── seria.rs    # COM1
│       ├── memoro/
│       │   ├── mod.rs          # regionoj, mapo, EniraInformo, Modulo
│       │   ├── kadro.rs        # bitmapa alokilo + Kani-pruvoj
│       │   └── virtuala.rs     # adresoj, permesoj, uzanta fenestro
│       ├── interrompo/mod.rs   # traktiloj, RAII-gardo
│       ├── kerno/
│       │   ├── mod.rs          # ĉefa sekvenco, testaj fadenoj
│       │   ├── objekto.rs      # Fadeno, stato, konteksta posedo
│       │   ├── procezo.rs      # Procezo, proceza stato, privata spaco
│       │   ├── planisto.rs    # rondiga planisto
│       │   ├── sistemvoko.rs   # ĉiuj `SYS_*` (1–20): mesaĝoj, kapabloj, procezoj, SNDS
│       │   ├── uzantspaco.rs   # ELF64-ŝargo, stako, U-kontrolo
│       │   └── ik.rs          # mesaĝoj, kapabloj + Kani-pruvoj
│       └── ekrano/
│           ├── mod.rs          # frambufa konzolo kaj elekto de streko
│           ├── fonto.rs        # Tiparo, TIPAROJ, glifo()
│           └── fonto/*.bin     # kvar rasterigitaj strekoj (DejaVu)
├── libmikerno/
│   └── src/{lib,kapablo,mesago,syscall}.rs
├── kernologiko/
│   └── src/{lib,adreso,memoro,mesagujo,preta_vico,snds,vekotemporizilo,vojo}.rs
├── horizonto/
│   ├── build.rs, linker.ld     # 0x1_0000_0000 (4 GiB)
│   └── src/{main,biblioteko,ligo}.rs
├── tools/
│   ├── run-qemu.sh             # QEMU-lanĉilo por WSL2/Linux
│   ├── dokumentaro.sh          # sistemvokaj numeroj kontraŭ la dokumentaro
│   ├── font/generi.py          # DejaVu → bitmapaj strekoj
│   └── limine/                 # praŝarĝilo (binaraj dosieroj)
├── third_party/dejavu/         # permesilo de la fonto
└── docs/
    ├── ANONCO-1.0.md           # preparita anonco por la eldono 1.0
    ├── ARM.md                  # plano por la arm64-ŝpuro
    ├── FLUOU.md                # deviga GU-gvidilo (FluoU)
    ├── FLUOU-montro.html       # vida montro de la gvidilo
    ├── MEJLOSTONOJ.md          # mejloŝtonoj kaj pruvaro
    └── VORTARO.md              # Esperanta terminologio
```

---

## Limigoj kaj sekva laboro

- **Neniu `fork`**: Ring 3 ne povas kopii adresan spacon. Novaj procezoj
  kreiĝas per `SYS_LANĜI` (ELF el SNDS), kaj `SYS_EXEC` anstataŭigas la bildon
  de la nuna procezo; la praŝarĝo decidas la komencan aron antaŭ la unua
  planista voko.
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
  IOMMU-izolado venas en M20 (post 1.0).
- **Unuprocesora**: neniu SMP aŭ CPU-afineco.
- **Fiksaj limoj**: 8 fadenoj, 16 mesaĝoj po fadeno, 64 bajtoj po mesaĝo,
  4 GiB fizika memoro.
- **IK**: la finpunktaj objektoj havas generaciojn kaj referenckalkulon, kaj
  `SENDI`/`RICEVI` uzas kapablo-tenilojn kun permesa kaj du-nivela generacia
  kontrolo; `REVOKI` estas tutfinpunkta kaj proceza detruo purigas la
  kapablojn; plena vico premegas la sendanton anstataŭ perdi mesaĝojn; la
  tempaj vokoj ofertas senfinan, neblokan kaj milisekundan atendon kun absoluta
  limdato; mankas nur `fork`/`exec`-distribuo de kapabloj (M17).
- **Ring 3-esceptoj nur `#PF`/`#GP`**: tiuj du mortigas la procezon; aliaj
  esceptoj — ankaŭ el Ring 3 — restas fatalaj, kaj demando-paĝado ne
  ekzistas.
- **Frambufo**: nur 32 bitoj/pikselo, kvar fiksaj strekoj (8×16 … 32×64), 111 glifoj
  (95 ASCII + Esperanto + citiloj), malplenigo anstataŭ rulumo.
- **Pruvoj**: 29 Kani-pruvoj spegulas la bitan logikon sur 64 kadroj kaj
  malgrandaj lokaj kapablotabeloj; la produkta kodo mem ne estas pruvita.
  `just kontroli` postulas apartan Kani-instalon.
- **Neniu DI nek testa pruvaro**: la kontrolo estas `konstrui` + `clippy` + `cargo fmt` +
  QEMU-smoka testo.
- **Versio**: la sola fonto estas `Cargo.toml`; la saluta linio legas
  `env!("CARGO_PKG_VERSION")` dum kompilado, do `just ruli` kaj la ISO montras
  la saman version, kaj `limine.conf` ne portas version. La dokumentojn
  kontrolas `tools/versio.sh --kontroli` (parto de `just kontroli`), kaj
  `--ĝisdatigi` ilin reskribas.
- **ISO povas malaktualiĝi** (vidita dum la kontrolo); ĉiam `just iso` post
  kerna ŝanĝo.

La plena, numerita vojkarto kun akceptkriterioj estas en
[docs/MEJLOSTONOJ.md](docs/MEJLOSTONOJ.md).

---

## Kontribuado

- La kodo, komentoj, identigiloj kaj dokumentaro estas en **Esperanto**.
- Uzu `rustfmt` kaj `cargo clippy --workspace --release` antaŭ ol sendi ŝanĝojn.
- Ne aldonu eksterajn kestojn sen klara kialo; la kerno celas resti sen
  dependaĵoj.
- Ĉiu `unsafe` bloko dokumentu sian sekurecan kontrakton.
- Por novaj arĥitekturaj malantaŭoj, respektu la fasadon
  `arkitekturo/mod.rs`; la nukleo ne rajtas uzi `x86_64`-specifan kodon rekte.
- Pruvu novan logikon per Kani, se ĝi estas sendependa de aparataro.

### Subskriboj

Ekde la 2026-11-22 la komitoj kaj Eldonoj de `makoto-lernanto` estas
subskribitaj per GPG-ŝlosilo (identigilo `A61A98D06B0B8E8F`; fingrospuro
`DD00 4F35 AC37 9928 DA48 9884 A61A 98D0 6B0B 8E8F`). GitHub montras la
subskribitajn komitojn kiel «Verified», do la deveno de la kodo estas
kontrolebla per `git log --show-signature`.

---

## Licenco kaj triaj komponantoj

La originala kodo de MiKerno estas publikigita sub la permesilo **MIT** (vidu
[`LICENSE`](LICENSE)). Vi rajtas uzi, modifi, kunfandi, publikigi, disdoni kaj
vendi la programaron, kondiĉe ke la kopirajta avizo kaj la permesila teksto
restu kunigitaj kun la disdonataĵo. La programaro estas provizita «tia, kia ĝi
estas», sen ia ajn garantio.

MiKerno elektis MIT anstataŭ Apache-2.0 ĉar la kerno estas malgranda kaj sen
eksteraj dependaĵoj, kaj ĉar MIT restas kongrua kun GPLv2 — tio faciligas reuzon
de la koderoj en aliaj kernaj projektoj.

La triaj komponantoj listigitaj sube havas siajn proprajn permesilojn; la MIT
permesilo ne anstataŭas ilin.

La dependaĵa kaj permesila kontrolo estas aŭtomatigita per `deny.toml`:
permesataj estas nur MIT, Apache-2.0, BSD-3-Clause kaj Unicode-DFS-2016; ĉio
alia — interalie GPL, AGPL kaj LGPL — estas rifuzata. `just permesiloj` (kaj la
DI-tasko `permesiloj-kontroli`) rulas `cargo deny check`; la laborloko nun havas
neniun eksteran dependaĵon, do la kontrolo estas precipe gardo por estontaj
aldonoj.

Enigitaj triaj komponantoj:

| Komponanto | Loko | Noto |
| --- | --- | --- |
| Limine | `tools/limine/` | Praŝarĝilo: Multiboot2 (`x86_64`) kaj Linux-protokolo (`arm64`) |
| DejaVu Sans Mono (bitmapaj strekoj) | `mikerno/src/ekrano/fonto/` | Rasterigita el DejaVu Sans Mono; permesilo Bitstream Vera + Arev en `third_party/dejavu/LICENSE.txt` |
| `tools/font/generi.py` | `tools/font/` | Generilo por la kvar strekoj; postulas Python 3 kaj Pillow |

`xorriso` ne estas enigita en la deponejon; `just iso` uzas la sisteman
pakaĵon (vidu [Postuloj](#postuloj)).

La bitaj tabeloj estas derivaĵoj de DejaVu Sans Mono. La permesilo postulas,
ke la kopirajtaj kaj permesaj tekstoj estu kunigitaj kun la fontprogramaro —
tial `third_party/dejavu/LICENSE.txt` estas parto de la deponejo. La nomoj
«Bitstream» kaj «Vera» ne estas uzataj por la derivaĵoj. Kontrolu la
koncernajn permesilojn antaŭ ol redistribui la ISO-on.

Fontdosiero uzita por generi la tabelojn: `DejaVuSansMono.ttf`, SHA-256
`602ec86b8948cfcd956482fe64f94c36c867770149ef2f791d4613f443bcecb3`.

---

*Ĉi tiu dokumento estas parto de la projekto kaj estas verkita en Esperanto,
same kiel la kodo.*
