# AGENTS.md — Laborreguloj

Ĉi tiu dosiero difinas la laborfluon de la AI-asistanto en la deponejo MiKerno.
Krom la klaraj instrukcioj de la uzanto por la nuna tasko, ĉiam sekvu ĉi tiun
dosieron.

## Lingva regulo

- Ĉio — kodo, identigiloj, komentoj, dokumentaro, komitaj mesaĝoj, nomoj de
  protokoloj kaj testoj — estas verkata en Esperanto.
- Restas en la angla nur anglaj terminoj, kiuj ne estas ŝanĝeblaj: `Rust`,
  `x86_64`, nomoj de registroj kaj instrukcioj, normaj interfacaj kaj aparataraj
  nomoj.
- Ligo-komandnomoj kaj iliaj argumentoj estas puraj ASCII: oni uzu Esperantajn
  vortojn, kiuj jam havas neniun ĉapelan literon (ĉ, ĝ, ĥ, ĵ, ŝ, ŭ), kaj
  neniam la x-sistemon. Ekzemplo: `ruli`, ne `lanĉi` aŭ `lancxi`.
- La ĉina lingvo ne estas permesita en la fonta tavolo. Aldona plurlingva
  mapada tavolo tamen estas permesata: apartaj tradukaj dosieroj (ekzemple
  `tradukoj/`) rajtas traduki la montratajn tekstojn al aliaj lingvoj,
  inkluzive la ĉinan (vidu `docs/FLUOU.md` §2.2). La mapada tavolo ne
  anstataŭas la Esperantajn fontajn tekstojn kaj ne rajtas ŝanĝi identigilojn,
  protokolajn nomojn aŭ komandnomojn; se ĝi mankas, la fonta teksto montriĝas.
  La retejo havas Esperantan fonton kaj apartan anglan mapadon en
  `docs/tradukoj/angla.js`; la ĉina restas ekster la reteja fonta tavolo.

## Versioj

La versio havas la formon `0.x.y` kaj rilatas al la mejloŝtonoj:

- Mejloŝtono **X** → `x = X − 10`
- **Fazo Y** en tiu mejloŝtono → `y = Y − 1`

Ekzemple: la Fazoj 1/2/3 de M13 respondas al `0.3.0` / `0.3.1` / `0.3.2`.
La formo `0.x.y` validas ekde M10; M0–M9 apartenas al la historio `0.0.x`.

Nuna versio: `0.8.3` (M18, Fazo 4: purigado de `#[allow]` kaj Ring 3-escepta
izolado).

La sola fonto de la versio estas `[workspace.package] version` en `Cargo.toml`.
La kerno legas ĝin per `env!("CARGO_PKG_VERSION")`, do `just ruli` kaj la ISO
montras ĝin aŭtomate; `limine.conf` ne portas version.

Lokoj, kie la versio devas resti kongrua (kontrolataj de `tools/versio.sh`):

- `README.md` — `**Versio:** 0.x.y (Cargo-laborloko)`
- `docs/MEJLOSTONOJ.md` — `Versio de la Cargo-laborloko` en la kapo
- `AGENTS.md` — `Nuna versio`
- `docs/index.html` — `data-versio="0.x.y"` en la `<html>`-etikedo

Post versio-ŝanĝo rulu `bash tools/versio.sh --ĝisdatigi`; `just kontroli`
rulas `--kontroli`.

## DI kaj formala pordego

- La sola loko por Kani estas GitHub Actions (`.github/workflows/di.yml`):
  `push` al `main` kaj `pull_request` rulas `kontroli-kaj-konstrui` (`just kontroli`,
  sen Kani, plus `just konstrui-arm` por la plata arm64-`Image`), poste
  `kani-pretigi` kaj la matricon `kani-kontroli`. La matrico
  havas unu taskon por ĉiu pruvo; ĉiu tasko rulas
  `timeout -k 10 600 cargo kani … --exact --harness <pruvo>` (10 min) sub
  `timeout-minutes: 30`, kaj suprenigas sian protokolon kiel artefakton eĉ
  ĉe malsukceso. Kani instaliĝas per
  `model-checking/kani-github-action@v1` sur Rust 1.100.0.
- La DI ankaŭ rulas `miri-kontroli`: nokta Rust kun Miri super la unu-testoj
  de `kernologiko` (`just miri`). Miri kaptas nedifinitan konduton, kiun la
  ordinara testa rultempo ne montras. Loke `just miri` estas la sama kontrolo.
- La tasko `permesiloj-kontroli` rulas `cargo deny check` laŭ la radika
  `deny.toml`. Nur MIT, Apache-2.0, BSD-3-Clause kaj Unicode-DFS-2016 estas
  permesataj; GPL, AGPL kaj LGPL estas rifuzataj. Loke la saman kontrolon
  ebligas `just permesiloj` (post `cargo install cargo-deny --locked`); ĝi ne
  estas parto de `just kontroli`, por ke tiu resta loke rulebla sen la ilo.
- Ĉiu `unsafe fn` portu sekurec-kontrakton (la antaŭkondiĉojn, kiujn la
  alvokanto devas garantii) kaj ĉiu `unsafe impl` portu `// SAFETY:`
  komenton; ambaŭ tekstoj estas en Esperanto. En la publika API de
  `libmikerno` la titolo restas `# Safety`, ĉar Clippy postulas ĝin.
- Loke oni **ne** rulas `cargo kani`; WSL2 elĉerpiĝas je memoro kaj
  diskonektiĝas. `just kontroli` nun enhavas nur la sen-Kani-ajn kontrolojn kaj
  estas loke rulebla: `bash tools/versio.sh --kontroli`, `just testoj`
  (`cargo test -p kernologiko --target <gastiganto>`),
  `cargo fmt --all -- --check`,
  `cargo clippy --workspace --release -- -D warnings`,
  `cargo clippy -p mikerno --release --target aarch64-unknown-none -- -D
  warnings`, `cargo clippy -p horizonto -p saluto --release --target
  aarch64-unknown-none -- -D warnings` kaj `cargo build --workspace
  --release`.
- La arkitekture sendependa logiko loĝas en la laborloka membro `kernologiko`;
  ĝiaj unu-testoj rulas sur la gastiganto kaj ne bezonas QEMU. Novaj puraj
  reguloj apartenu al `kernologiko` kaj ricevu `#[cfg(test)]`-kazojn; la kerno
  ilin re-eksportu prefere ol duobligi la kodon.
- Post puŝo kontrolu la runon en la agoj-paĝo de la deponejo (aŭ per
  `gh run watch`); la pordego estas verda nur kiam la tuta matrico
  `kani-kontroli` kaj la taskoj `miri-kontroli` kaj `permesiloj-kontroli`
  sukcesas. Detaloj estas en
  `docs/MEJLOSTONOJ.md` §4.7–§4.8.

## Laborfluo por unu Fazo

1. Efektivigu nur la Fazon indikitan de la uzanto; ne komencu la sekvan.
2. Post la efektivigo rulu loke `just kontroli` (sen Kani) kaj la kontrolojn
   indikitajn de la uzanto, kaj certigu, ke ili pasas. La formalan pordegon
   (la DI-matricon `kani-kontroli`) plenumas la DI post la puŝo. Rekonstruu la
   ISO-ojn (`just iso` kaj `just iso-arm`), kontrolu iliajn praŝarĝojn kaj,
   post verda DI, alŝutu ilin al la Eldono de la nuna versio (vidu «Eldono
   de ISO») antaŭ la raporto.
3. Ĝisdatigu la version (laŭ la supra mapado; `bash tools/versio.sh
   --ĝisdatigi` por la dokumentoj), `docs/MEJLOSTONOJ.md` kaj
   `docs/VORTARO.md`.
4. **Haltu tuj**, raportu la rezulton al la uzanto kaj atendu pluajn
   instrukciojn.

## Komitaj reguloj

- Antaŭ ĉiu komito mezuru la Ring 0-kodon per `bash tools/kodlinioj.sh
  --linio` kaj aldonu la ricevitan vicon al la tabelo «Grando de la Ring
  0-kodo» en `docs/MEJLOSTONOJ.md`.
- Sen klara permeso de la uzanto («vi povas komiti»), **neniam** rulu
  `git commit` aŭ `git push`.
- Post la permeso, komitu ĉiujn ŝanĝojn de la nuna Fazo kaj puŝu ilin; la
  mesaĝo estu en Esperanto laŭ la stilo de la deponejo.
- Post la puŝo atendu la DI-runon kaj raportu ĝian konkludon; ruĝa
  `kani-kontroli` blokas pluan laboron ĝis la kaŭzo estas riparita.
- Ne kunfandu plurajn Fazojn en unu komito, krom se la uzanto klare postulas
  tion.

## Eldono de ISO

- Post ĉiu finita tasko (ne nur Fazo), kiu ŝanĝas la kernon, la lanĉilojn aŭ
  la enhavon de la SNDS-bildo, kaj post verda DI, konstruu `just iso` kaj
  `just iso-arm` el la puŝita kodo, kontrolu la praŝarĝojn de ambaŭ
  (`QEMU_DISPLAY=none bash tools/run-qemu.sh iso` kaj
  `QEMU_DISPLAY=none bash tools/run-qemu-arm.sh --iso`) kaj alŝutu ilin al la
  GitHub-Eldono de la nuna versio. Nur-dokumenta tasko ne ŝanĝas la
  ISO-enhavon; tiam ne rekonstruu, sed kontrolu, ke la lasta Eldono ankoraŭ
  kongruas kun la nuna kodo.
- Se la Eldono por la nuna versio ankoraŭ ne ekzistas, uzu `gh release
  create v0.x.y mikerno.iso mikerno-arm.iso --target <plena koda SHA>
  --title … --notes-file …`. La etikedo montru al la koda komito de la Fazo,
  ne al posta dokumenta komito; la Eldono estu la plej nova, ne antaŭeldono,
  krom se la uzanto diras alie.
- Se ĝi jam ekzistas (ankaŭ kiam la versio ne ŝanĝiĝis), anstataŭigu la
  aktivaĵojn per `gh release upload v0.x.y mikerno.iso mikerno-arm.iso
  --clobber` kaj ĝisdatigu la SHA-256 en la priskribo kaj en
  `docs/MEJLOSTONOJ.md`.
- La priskribo enhavu la SHA-256 de ambaŭ ISO-oj kaj la ĉefajn akceptajn
  liniojn; la sama informo iru en `docs/MEJLOSTONOJ.md`.
- Neniam alŝutu ISO-ojn konstruitajn el nepuŝita aŭ malverda kodo.

## Vortaro

- Post ĉiu Fazo ĝisdatigu `docs/VORTARO.md`.
- Registru novajn identigilojn, tipojn, protokolajn nomojn kaj aparatarajn
  terminojn (Esperanto / angla / rimarko).
- Laŭbezone plenigu la sekciojn «Protokolaj kaj aparataraj nomoj» kaj
  «Reguloj».
- Identigiloj en la kodo sekvu la vortaron; ne enkonduku anglajn identigilojn
  ekster ĝi.

## Vida montro

- `docs/FLUOU-montro.html` estas memstara, nur-Esperanta vidmontraĵo de
  `docs/FLUOU.md`: fenestra modelo, Eniro (`Ctrl+E`, provizore; estonte
  `Mito`) kaj Ligo.
- Ĝi ne estas normo; la deviga teksto restas `docs/FLUOU.md`. Se la gvidilo
  ŝanĝiĝas, ĝisdatigu la montron samtempe.
- La montro uzas la samajn ĵetonojn, materialojn kaj regilojn; ne enkonduku
  novan stilon nur por la montro.
