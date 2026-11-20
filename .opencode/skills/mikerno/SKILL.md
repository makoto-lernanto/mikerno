---
name: mikerno
description: Projekta gvidilo por labori en la deponejo MiKerno — eta unuprocesora kerno en Rust por x86_64 kaj arm64, kun ABI 2, Ring 3, FluoU, SNDS, Kani-DI kaj Esperanta dokumentaro. Uzu ĉi tion antaŭ ol ŝanĝi kodon, dokumentojn, version, ISO-on aŭ Eldonon.
---

# MiKerno — projekta skill

Ĉi tiu skill resumas la regulojn kaj konojn necesajn por sekure labori en la
deponejo MiKerno. La devigaj fontoj restas `AGENTS.md`, `docs/FLUOU.md`,
`docs/VORTARO.md` kaj `docs/MEJLOSTONOJ.md`; ĉi tiu dosiero estas rapida
gvidilo, ne anstataŭo.

## 1. Identeco de la projekto

- **MiKerno** estas eta, memstara, unuprocesora kerno verkita tute en Rust por
  `x86_64` kaj `arm64`.
- `#![no_std]`, `#![no_main]`, sen eksteraj Rust-dependaĵoj, sen C en la kerno.
- La celo estas legebleco, hoblebleco kaj pruvebleco — ne maksimuma
  funkciaro.
- Permesilo: MIT. La bitmaphava tiparo venas el `third_party/dejavu/`.
- Cargo-laborloko kun tri kestoj:
  - `mikerno` — la kerno (Ring 0).
  - `libmikerno` — la komuna kontrakto inter kerno kaj uzantspaco (ABI 2).
  - `horizonto` — la unua uzanta procezo, ligita ĉe `0x1_0000_0000`.

## 2. Lingva regulo

- Ĉio verkata en la deponejo estas **Esperanto**: kodo, identigiloj, komentoj,
  protokolaj tekstoj, dokumentaro, mesaĝoj, dosiernomoj kaj komitmesaĝoj.
- **Konstitucia regulo — komitmesaĝoj tute Esperanto.** La tuta komitmesaĝo
  DEVI esti verkata en Esperanto: la titolo, la korpo kaj ĉiuj aldonitaj
  linioj (ekzemple piedlinioj, subskriboj kia `Co-authored-by:` kaj CI-notoj).
  Komito kun angla aŭ ĉina mesaĝo aŭ aldonaĵo estas nekonforma. Sama regulo
  validas por etikedmesaĝoj kaj eldonpriskriboj.
- Restas en la angla nur neŝanĝeblaj propraj aŭ fiksaj nomoj: `Rust`,
  `x86_64`, `arm64`, registraj kaj instrukciaj nomoj, normaj interfacaj kaj
  aparataraj nomoj (vidu `docs/VORTARO.md`).
- La ĉina lingvo estas malpermesita en la **fonta** tavolo. Aparta mapada
  tavolo rajtas traduki montratajn tekstojn (ekzemple
  `docs/tradukoj/angla.js` por la retejo); ĝi ne anstataŭas la Esperantajn
  fontajn tekstojn kaj ne rajtas ŝanĝi identigilojn, protokolajn nomojn aŭ
  komandnomojn.
- Ne uzu anglajn identigilojn kiel `cpu`, `ram`, `memory`, `thread`,
  `scheduler`, `page` aŭ `frame`. Uzu la vortaron: `procesoro`, `memoro`,
  `fadeno`, `planisto`, `paĝo`, `kadro`.
- Dosiernomoj de moduloj restas ASKIAJ (`pagado.rs`), dum la prozo uzas
  diakritojn («paĝado»). Ne-ASKIAj identigiloj kiel `PaĝaTablo` estas
  permesataj.

## 3. Versio kaj dokumentoj

- La sola fonto de la versio estas `[workspace.package] version` en
  `Cargo.toml`. La kerno legas ĝin per `env!("CARGO_PKG_VERSION")`;
  `limine.conf` ne portas version.
- Versia skemo: `0.x.y`.
  - Mejloŝtono **X** → `x = X − 10`.
  - Fazo **Y** → `y = Y − 1`.
  - Ekzemplo: M17 Fazo 3 → `0.7.2`.
- La versio devas kongrui en:
  - `README.md` — `**Versio:** 0.x.y (Cargo-laborloko)`
  - `docs/MEJLOSTONOJ.md` — kapo
  - `AGENTS.md` — «Nuna versio»
  - `docs/index.html` — `data-versio="0.x.y"`
- Kontrolu per `bash tools/versio.sh --kontroli`; ĝisdatigu per
  `bash tools/versio.sh --ĝisdatigi`.

### Rolo de la dokumentoj

| Dosiero | Rolo |
| --- | --- |
| `README.md` | Superrigardo kaj rapida starto; povas postresti malantaŭ la nuna stato. |
| `docs/MEJLOSTONOJ.md` | La inĝeniera taglibro: celoj, fazoj, akceptkriterioj, DI-runoj, Eldonoj, buĝetoj. La fidinda fonto por historiaj detaloj. |
| `docs/VORTARO.md` | Esperanta terminologio kaj listo de permesataj anglaj nomoj. Novaj identigiloj sekvu ĝin. |
| `docs/FLUOU.md` | La deviga normo por la grafika uzinterfaco FluoU. |
| `docs/FLUOU-montro.html` | Nur-Esperanta vida montro; ne normo. Ĝisdatigu ĝin kune kun `FLUOU.md`. |
| `docs/ARM.md` | La plano kaj historio de la arm64-ŝpuro. |
| `docs/index.html` | La retejo; uzas la samajn FluoU-ĵetonojn kiel la montro. |
| `AGENTS.md` | La laborfluo, DI-reguloj, komitaj kaj Eldonaj reguloj. |

Se `README.md` kaj la nuna stato malkongruas (ekzemple la nombro de
Kani-pruvoj aŭ la listo de limigoj), fidu al `docs/MEJLOSTONOJ.md`, al la
DI-matrico kaj al la efektiva kodo; ne blinde kopiu nombrojn el la README.

## 4. Strukturo

```text
Cargo.toml                  # laborloko, versio, lintoj
justfile                    # konstrui / ruli / iso / kontroli
.cargo/config.toml          # x86_64-unknown-none, rust-lld, QEMU-lanĉilo
mikerno/
  boot/                     # linker.ld, limine.conf
  src/
    main.rs                 # enirpunkto, skribu!, paniko
    arkitekturo/            # komuna fasado + x86_64 (GDT, IDT, APIC, pagado, ...)
    memoro/                 # kadra alokilo, virtuala mapado, Kani-pruvoj
    interrompo/             # traktiloj, InterrompaGardo
    kerno/                  # planisto, procezoj, IK, sistemvokoj, exec
    ekrano/                 # frambufa konzolo kaj bitmapa tiparo
libmikerno/src/             # kapablo.rs, mesago.rs, syscall.rs, starto.rs
horizonto/src/              # uzanta bildo kaj bump-alokilo
tools/                      # QEMU, ISO, SNDS, versio, kodlinioj
docs/                       # normoj, taglibro, retejo, tradukoj
```

## 5. Konstruo, rulado kaj kontrolo

```sh
just konstrui-ciujn     # la tuta laborloko (kerno, libmikerno, horizonto)
just konstrui           # nur la kerno
just snds               # determinisma SNDS-bildo el mikerno/snds/
just ruli               # QEMU -kernel, PVH (sen ISO)
just iso                # mikerno.iso (Limine + Multiboot2)
just ruli-iso           # la ISO en QEMU
just konstrui-arm       # arm64: plata Image + horizonto
just ruli-arm           # arm64 Image en QEMU virt
just iso-arm            # UEFI-bootebla mikerno-arm.iso
just ruli-iso-arm       # la arm64-ISO per AAVMF
just kontroli           # versio + fmt + Clippy (ambaŭ arkoj) + konstruo
```

Senkapa kontrolo de la praŝarĝoj:

```sh
QEMU_DISPLAY=none bash tools/run-qemu.sh iso
QEMU_DISPLAY=none bash tools/run-qemu-arm.sh --iso
```

- Postuloj: Rust 1.100.0 (minimuma), `clippy`, `rustfmt`, la celoj
  `x86_64-unknown-none` kaj `aarch64-unknown-none`, `qemu-system-x86_64`,
  `qemu-system-aarch64`, `xorriso`; por arm64-ISO ankaŭ `mkfs.fat`, `mmd`,
  `mcopy` kaj UEFI-firmware.
- `just kontroli` estas la **loka** pordego kaj **ne** enhavas Kani-on.
- `cargo kani` **neniam** ruliĝas loke: WSL2 elĉerpiĝas je memoro kaj
  diskonektiĝas. Kani ruliĝas nur en GitHub Actions.
- Rust 2024, MSRV 1.100. Ne aldonu eksterajn Rust-dependaĵojn sen klara
  bezono kaj sen demandi.

## 6. Arkitekturo — punktoj, kiujn oni devas scii

### Praŝarĝo

- Du protokoloj: Xen-PVH por rapida `cargo run`/`just ruli`, kaj
  Multiboot2/Limine por la ISO.
- La bildo estas `ET_DYN` kun relativaj relokoj; KASLR elektas novan bazan
  adreson ĉiustarte.
- x86_64: 32→64-bita ŝelko, GDT/TSS, IDT, `int 0x80`-pordego (DPL3).
- arm64: plata `Image`, `EL1` → `EL0` per `eret`, `svc #0`, `TTBR0_EL1` por
  ĉiu procezo.

### Memoro

- 4 KiB paĝoj kaj kadroj; bitmaps kadra alokilo.
- La kerno uzas identecan mapon por sia propra bildo kaj MMIO.
- W^X laŭ sekcioj; NX, SMEP kaj SMAP estas ŝaltataj se la procesoro subtenas
  ilin.
- Ring 0 legas uzantajn bufrojn nur en `STAC`-fenestro (`StacGardo`); ne
  aliru uzantan memoron sen tiu gardo.
- Uzanta bildo: `0x1_0000_0000`; stako: `0x1000_0000`–`0x1010_0000`;
  kupo: `0x1_0200_0000`–`0x2_0000_0000`.
- Sekreta paĝo: `0x2_0000_0000`; la izolada provo atendas `#PF` ĉe
  `0x200001000`.

### Interrompoj kaj planado

- x86_64: xAPIC (MMIO), PIT-kalibrita temporizilo je 100 Hz; EOI nur por
  APIC-vektoroj, neniam por `int 0x80`.
- arm64: GICv2/GICv3 kaj `CNTP_TVAL_EL0`; `DAIF`-maskado per
  `InterrompaGardo`.
- Antaŭokupa cikla planado kun prioritatoj: `Malalta` = 1, `Normala` = 2,
  `Alta` = 4 tikoj. Neniu malsatigo; preta fadeno atendas maksimume unu
  rondon.
- `Vekotemporizilo` estas min-kupo por `dormi(md)` kaj tempolimoj.
- Po-fadenaj plenumtikoj estas specimenoj (≈10 ms), ne preciza murhorloĝa
  mezuro.

### IK kaj ABI 2

- `Mesago` estas 96 bajtoj; datumaro maksimume 64 bajtoj.
- Mesaĝujo: 16 fendoj, FIFO; plena vico premegas la sendanton anstataŭ perdi
  mesaĝon.
- `Tenilo` = indekso + generacio; generacio ekas ĉe `1`; indekso `< 16`.
- Kapabloj: `SENDI`, `RICEVI`, `REVOKI`, `PLENAJ`, `ĈIUJ`; la kerna linio 0
  restas ĉiam malplena.
- `REVOKI` estas tutfinpunkta; `FERMI` fermas nur la lokan tenilon.
- Sistemvokoj (x86_64: `rax` + `rdi`/`rsi`/`rdx`/`r10`, `int 0x80`;
  arm64: `x8` + `x0`–`x3`, `svc #0`):

| Nr | Nomo | Signifo |
| --- | --- | --- |
| 1 | `SENDI` | Sendi mesaĝon |
| 2 | `RICEVI` | Ricevi mesaĝon |
| 3 | `SKRIBI` | Skribi al konzolo |
| 4 | `FINI` | Finigi la nunan fadenon/procezon |
| 5 | `CEDI` | Cedi la procesoron |
| 6 | `KREI_FINPUNKTON` | Krei finpunkton |
| 7 | `DONI` | Transdoni kapablon |
| 8 | `REVOKI` | Revoki finpunkton |
| 9 | `RICEVI_TEMPO` | Ricevi kun tempolimo |
| 10 | `SENDI_TEMPO` | Sendi kun tempolimo |
| 11 | `FERMI` | Fermi tenilon |
| 12 | `DORMI` | Dormi (ms) |
| 13 | `SYS_BRK` | Etendi la uzantan datumsegmenton |
| 14 | `SYS_EXEC` | Anstataŭigi la nunan procezon per ELF el SNDS |

- Tempolimo: `u32::MAX` = senfina; `0` = nebloka; alie `ceil(ms/10)` tikoj.
- Gravaj erar-kodoj: `-1` malĝusta argumento, `-2` nekonata voko, `-3`
  neekzistanta celo, `-4` ripoza kunteksto, `-9` nevalida/eksdata tenilo,
  `-11` provu denove (nebloka), `-28` tabelo plena, `-32` finpunkto revokita,
  `-90` tro longa mesaĝo, `-110` tempolimo. La konstantoj estas en
  `libmikerno/src/syscall.rs`; ne inventu novajn kodojn.

### Uzantspaco

- `horizonto` estas la unua Ring 3-procezo; ĝi uzas `libmikerno`.
- Eniro al Ring 3: x86_64 per `iretq` (`CS=0x23`, `SS=0x1B`,
  `RFLAGS=0x202`); arm64 per `eret` (`SPSR_EL1=EL0t`).
- `SYS_BRK` nur pligrandigas la kupolimon; neniam malmapas.
- La kupalokilo en `horizonto/src/biblioteko.rs` estas `bump`-alokilo;
  `liberigi` estas malplena.
- `SYS_EXEC` legas ELF-dosieron el la radiko de SNDS, validigas ĝin, kreas
  novan adresan spacon kaj anstataŭigas la nunan procezon (kiel `execve`, ne
  `fork`).
- `fork`, ĝenerala `wait`, kaj plena ŝelo ankoraŭ mankas aŭ estas teknika
  ŝuldo; ne prezenti ilin kiel pretajn.

### Konservado

- `virtio-blk` moderna (virtio 1.0) nur: PCI sur x86_64, MMIO sur arm64.
- Unu vico, polusa atendo, po unu sektoro (512 bajtoj) per peto; sen IOMMU.
- Antaŭ sciigo kaj antaŭ legado de la uzita ringo staras
  `fence(Ordering::SeqCst)`.
- SNDS estas propra, nurlega dosiersistemo: superbloko, inoda tabelo,
  dosierujaj eroj, kontiguaj datumblokoj.
- Limigoj: maksimuma dosiero 64 KiB, nomoj ĝis 59 bajtoj, unu ebeno da
  dosierujoj, neniu kaŝmemoro, bildo fiksita dum konstruo.
- `tools/krei_snds_img.sh` devas esti determinisma (ordigo laŭ UTF-8-bajtoj,
  nulplenigo); `just snds` kreas la bildon sen QEMU.

## 7. FluoU — la deviga grafika lingvo

FluoU estas la sola permesata grafika kadro de Horizonto. Moto: **«Ĉio
fluas.»** La norma teksto estas `docs/FLUOU.md`; la montro estas nur vida.

Plej gravaj reguloj:

- **G01** — ĉiu grafika aplikaĵo deklaru `gu = "fluoU"` en sia manifesto.
- **G02** — sen `FENESTRO`-tenilo ne ekzistas pikseloj.
- **G05** — neniu aplikaĵo desegnu rekte en la frambufon; nur la kompozitoro
  rajtas.
- **G06** — la kompozitoro akceptas nur fenestrojn kun valida `FENESTRO`-tenilo
  ricevita per `DONI`.
- **G07** — la gvidilo venkas super prototipoj, ekrankopioj kaj historia kodo.

Fenestroj sekvas la **Windows-modelon**, ne macOS:

- titolstrio 32 DIP alta; aplika ikono 16 × 16 DIP maldekstre; titolo 12 DIP
  duondika, horizontale centrita;
- dekstre, sen interspaco: `Malgrandigi`, `Pligrandigi`/`Restarigi`, `Fermi`;
- regiloj 46 × 32 DIP, ikonoj 10 × 10 DIP, ortangulaj, senkoloraj en la normala
  stato; `Fermi` ruĝiĝas nur ĉe ŝvebo (`#C42B1C`);
- fenestra radiuso 8 DIP; la regiloj mem ne havas radiuson;
- neniuj trikoloraj rondaj «trafiklumoj»;
- `Alt+F4`, `Alt+Space`, duobla klako sur la titolstrio kaj klavara atingeblo
  de `Fermi` estas postulataj.
- `Ligo` estas vera fenestro kaj montras la tri regilojn; `Eniro` estas paletro
  sen fenestraj regiloj kaj fermiĝas per `Esc` aŭ klako sur la kurteno.

Ĵetonoj kaj materialoj:

- Ĉiuj koloroj, mezuroj, tipografio, movo kaj denseco venas el la ĵetonoj de
  FluoU. Ne fiksu ilin rekte en komponantoj.
- Materialoj: nur `Mika`, `Akrila` kaj `Revelo`.
- Ambaŭ temoj (`malluma`, `hela`) devas difini ĉiujn ĵetonojn; mankanta ĵetono
  estas konstru-eraro.
- Kontrasto kaj alirebleco estas devigaj; movo respektu la sisteman preferon
  por malpli da movado.

Eniro kaj Ligo:

- `Ctrl+E` malfermas Eniron (provizore; estonte `Mito`, la tradicia `Win`-klavo).
- Tri reĝimoj: serĉo, kalkulo post `=`, delegado post `>`.
- La kalkulilo estas sekura analizilo sen `eval`, sen dosieroj kaj sen reto.
- `Ligo` estas PowerShell 7-simila objekta ŝelo: `|` konektas tipizitajn
  stadiojn kaj transdonas rekordojn, ne tekstajn liniojn
  (`filtri`, `ordigi`, `unua`, `sumi`, `nombri`, `montri`, `teksto`).
- `listi` eligas `Dosiero`-rekordojn (`nomo`, `tipo`, `grandeco`, `tenilo`) kaj
  en interago montras koloran tabelon laŭ la ĵetonoj kaj semantikaj roloj;
  koloroj nur akompanas la signifon — tipo, grandeco, tenilo kaj eraroj ne
  dependas nur de koloro.
- Ĉiuj komandnomoj estas Esperantaj kaj sen diakritaj signoj; anglaj aliasoj
  (`ls`, `cat`, `ps`, `kill`, `echo`, `help`, `clear`) estas malpermesitaj, ankaŭ
  kaŝe. Prompto: `ligo:~>`.
- Erarmesaĝo: `komando: kaŭzo (kodo)`, ekzemple
  `legi: dosiero ne ekzistas (-3)`.
- `teksto` estas la sola eksplicita limo inter objektoj kaj teksto.
- `mortigi` rifuzas la protektajn procezojn `pid ≤ 3`.

Konformkontrolo P01–P20 en `docs/FLUOU.md` estas la akcepta listo; ne
enkonduku novan stilon nur por unu ekrano.

## 8. Laborfluo por tasko aŭ Fazo

1. **Efektivigu nur tion, kion la uzanto petis** — ne komencu la sekvan Fazon
   kaj ne kunfandu plurajn Fazojn sen klara postulo.
2. Legu la koncernan sekcion de `docs/MEJLOSTONOJ.md` kaj la regulojn de
   `docs/VORTARO.md` antaŭ ol skribi kodon.
3. Efektivigu, poste rulu loke:
   ```sh
   just kontroli
   ```
   kaj la kromajn kontrolojn, kiujn la uzanto aŭ la Fazo postulas.
4. Se la ŝanĝo tuŝas la kernon, la lanĉilojn aŭ la SNDS-bildon, rekonstruu kaj
   praŝarĝu ambaŭ ISO-ojn:
   ```sh
   just iso
   just iso-arm
   QEMU_DISPLAY=none bash tools/run-qemu.sh iso
   QEMU_DISPLAY=none bash tools/run-qemu-arm.sh --iso
   ```
5. Ĝisdatigu la version laŭ la mapado, `docs/MEJLOSTONOJ.md` kaj
   `docs/VORTARO.md`. Uzu `bash tools/versio.sh --ĝisdatigi` por la dokumenta
   versio.
6. **Haltu kaj raportu** la rezulton; atendu pluajn instrukciojn.

### Antaŭ komito

- Nur kun klara permeso de la uzanto rulu `git commit` aŭ `git push`.
- Antaŭ ĉiu komito mezuru la Ring 0-kodon:
  ```sh
  bash tools/kodlinioj.sh --linio
  ```
  kaj aldonu la ricevitan vicon al la tabelo «Grando de la Ring 0-kodo» en
  `docs/MEJLOSTONOJ.md`.
- Komitu ĉiujn ŝanĝojn de la nuna Fazo per Esperanta mesaĝo laŭ la stilo de la
  deponejo. Ne kunfandu Fazojn.
- Post puŝo atendu la DI-runon kaj raportu ĝian konkludon. Ruĝa
  `kani-kontroli` blokas pluan laboron.

### DI kaj Kani

- La sola formala pordego estas GitHub Actions
  (`.github/workflows/di.yml`), ĉe `push` al `main` kaj `pull_request`.
- Taskoj: `kontroli-kaj-konstrui` (`just kontroli` + `just konstrui-arm`),
  `kani-pretigi` (eltiras la pruvliston) kaj la matrico `kani-kontroli` — unu
  tasko por ĉiu pruvo:
  `timeout -k 10 600 cargo kani … --exact --harness <pruvo>`.
- Ne malstreĉu aŭ malŝaltu pruvojn por igi la pordon verda. Riparu la kaŭzon
  (ofte mempremo de CBMC: malplenigu datumaron, malgrandigu `unwind`, aŭ
  anstataŭigu tut-tabelan egalecon per skalaraj komparoj).
- La nombro de pruvoj ŝanĝiĝas laŭ la Fazo; fidu al la matrico, ne al fiksita
  nombro.

### ISO kaj Eldono

- Post ĉiu finita tasko, kiu ŝanĝas la kernon, la lanĉilojn aŭ la SNDS-bildon,
  kaj post verda DI, konstruu `just iso` kaj `just iso-arm` el la **puŝita**
  kodo, kontrolu la praŝarĝojn kaj alŝutu ilin al la GitHub-Eldono de la nuna
  versio.
- Etikedo `v0.x.y` montru al la **koda** komito de la Fazo, ne al posta
  dokumenta komito. La Eldono estu la plej nova, ne antaŭeldono, krom se la
  uzanto diras alie.
- Se la Eldono jam ekzistas, uzu
  `gh release upload v0.x.y mikerno.iso mikerno-arm.iso --clobber` kaj
  ĝisdatigu la SHA-256 en la priskribo kaj en `docs/MEJLOSTONOJ.md`.
- La priskribo enhavu la SHA-256 de ambaŭ ISO-oj kaj la ĉefajn akceptajn
  liniojn.
- **Neniam** alŝutu ISO-ojn konstruitajn el nepuŝita aŭ malverda kodo.
- Nur-dokumenta tasko ne ŝanĝas la ISO-enhavon: ne rekonstruu, sed kontrolu,
  ke la lasta Eldono ankoraŭ kongruas kun la nuna kodo.

## 9. Kaptiloj kaj kontrol-listoj

### Oftaj kaptiloj

- **Malnova ISO.** `mikerno.iso` kaj `iso_root/` estas ignorataj de git; post
  kerna ŝanĝo rerulu `just iso`. Okazis, ke ISO montris malnovan version.
- **README povas postresti.** La nombro de pruvoj kaj la listo de limigoj en
  `README.md` ne ĉiam spegulas la nunan kodon; kontrolu `MEJLOSTONOJ.md` kaj
  la DI-matricon.
- **Retejaj fiksaĵoj.** `docs/index.html` portas fiksajn SHA-256, ekzemplan
  konzolan eligon kaj `data-versio`. Post nova Eldono kontrolu, ke la haketoj,
  la loga versio kaj la elŝutaj ligiloj kongruas; ili emas postresti.
- **SMAP.** La kerno ne rajtas rekte legi aŭ skribi uzantan bufron sen
  `StacGardo`/`STAC`; uzu kopion aŭ la taŭgan gardon.
- **arm64-konzolo.** `skribi_formaton` devas malŝalti interrompojn ankaŭ sur
  arm64; alie la temporizila traktilo povas skribi meze de `novlinio` kaj
  difekti `TTBR0`.
- **arm64-ELF.** `horizonto/build.rs` devas pasigi `-z max-page-size=4096`;
  alie la bildo havas 64 KiB-truojn kaj ne taŭgas por la 64 KiB-limigo de
  SNDS.
- **virtio.** La PCI-kapablolisto ne estas sinsekva; ne postulu kreskantan
  montrilon. La disponebla indekso devas progresi post ĉiu peto. La MMIO-statuso
  estas 32-bita registro — legu/skribu 32 bitojn, ne bajtojn.
- **QEMU-arm64.** Por la ISO uzu `cortex-a76` (AAVMF 2025.11 blokiĝas sub TCG
  kun `-cpu max`); `QEMU_ARM_CPU` povas anstataŭigi la elekton. Por MMIO
  necesas `-global virtio-mmio.force-legacy=false`.
- **Diskaj provoj.** La lanĉiloj uzas `-snapshot`, do memprovo ne ŝanĝas la
  diskbildon; ne forigu tiun flagon.
- **PCID.** La kerno malŝaltas `CR4.PCIDE` kaj postulas alinitan `CR3`-radikon;
  ne reŝaltu PCID sen revizii la TLB-modelon.
- **Kani-memoro.** Grandaj simbolaj datumoj kaŭzas `shutdown signal` aŭ
  `out of memory`; uzu malplenajn mesaĝojn, taŭgajn `unwind`-limojn kaj
  skalarajn komparojn anstataŭ `memcmp` de tutaj tabeloj.

### Sukcesa starto — kion atendi

- Linioj `[MiKerno] Saluton, mi estas MiKerno v0.x.y!`, KASLR, SMEP/SMAP,
  `[memoro] W^X (bone)`, `[memoro] CR3 (bone)`, konzolo/frambufo.
- Fine: `[kerno] fina stato: 6 mortintaj, 0 pretaj, 0 blokitaj` kaj 1000
  tikoj (laŭ la nuna scenaro).
- Neniu linio kun `MALBONE`.
- La `[eraro]`-linioj de la negativaj provoj (montrilo, sekreta paĝo, kapablo,
  SMAP) estas **intencaj**; ne forigu ilin.
- Por arm64 ankaŭ kontrolu `[ring3] kupo: alokita …` kaj
  `[ring3] exec: nova bildo funkcias`.

### Rapida antaŭ-raporta kontrolo

```sh
bash tools/versio.sh --kontroli
bash tools/kodlinioj.sh --linio
cargo fmt --all -- --check
cargo clippy --workspace --release -- -D warnings
cargo clippy -p mikerno --release --target aarch64-unknown-none -- -D warnings
cargo build --workspace --release
```

Se vi tuŝis la kernon aŭ la bildon, aldonu la du ISO-praŝarĝojn. Se vi tuŝis
`docs/index.html` aŭ `docs/FLUOU.md`, kontrolu ankaŭ la retejan version kaj la
montron.

## 10. Raportado

En la fina raporto resumu:

- kion vi ŝanĝis kaj kial;
- kiujn komandojn vi rulis kaj ĉu ili sukcesis;
- la rezulton de `just kontroli` kaj de la QEMU-praŝarĝoj;
- la nunan version kaj la ĝisdatigitajn dokumentojn;
- ĉu DI aŭ Eldono ankoraŭ bezonas agon;
- ĉiujn restantajn limigojn aŭ teknikajn ŝuldojn.

Ne diru, ke la formala pordego estas verda, ĝis la DI-matrico `kani-kontroli`
vere sukcesis.
