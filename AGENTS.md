# AGENTS.md — Laborreguloj

Ĉi tiu dosiero difinas la laborfluon de la AI-asistanto en la deponejo MiKerno.
Krom la klaraj instrukcioj de la uzanto por la nuna tasko, ĉiam sekvu ĉi tiun
dosieron.

## Lingva regulo

- Ĉio — kodo, identigiloj, komentoj, dokumentaro, commit-mesaĝoj, nomoj de
  protokoloj kaj testoj — estas verkata en Esperanto.
- Restas en la angla nur anglaj terminoj, kiuj ne estas ŝanĝeblaj: `Rust`,
  `x86_64`, nomoj de registroj kaj instrukcioj, normaj API- kaj aparataraj
  nomoj.
- La ĉina lingvo ne estas permesita en la fonta tavolo. Aldona plurlingva
  mapada tavolo tamen estas permesata: apartaj tradukaj dosieroj (ekzemple
  `tradukoj/`) rajtas traduki la montratajn tekstojn al aliaj lingvoj,
  inkluzive la ĉinan (vidu `docs/FLUOU.md` §2.2). La mapada tavolo ne
  anstataŭas la Esperantajn fontajn tekstojn kaj ne rajtas ŝanĝi identigilojn,
  protokolajn nomojn aŭ komandnomojn; se ĝi mankas, la fonta teksto montriĝas.

## Versioj

La versio havas la formon `0.x.y` kaj rilatas al la mejloŝtonoj:

- Mejloŝtono **X** → `x = X − 10`
- **Fazo Y** en tiu mejloŝtono → `y = Y − 1`

Ekzemple: la Fazoj 1/2/3 de M13 respondas al `0.3.0` / `0.3.1` / `0.3.2`.
La formo `0.x.y` validas ekde M10; M0–M9 apartenas al la historio `0.0.x`.

Nuna versio: `0.4.5` (M14, Fazo 6).

La sola fonto de la versio estas `[workspace.package] version` en `Cargo.toml`.
La kerno legas ĝin per `env!("CARGO_PKG_VERSION")`, do `just run` kaj la ISO
montras ĝin aŭtomate; `limine.conf` ne portas version.

Lokoj, kie la versio devas resti kongrua (kontrolataj de `tools/versio.sh`):

- `README.md` — `**Versio:** 0.x.y (workspace)`
- `docs/MEJLOSTONOJ.md` — `Versio de la workspace` en la kapo
- `AGENTS.md` — `Nuna versio`

Post versio-ŝanĝo rulu `bash tools/versio.sh --ĝisdatigi`; `just verify`
rulas `--kontroli`.

## Laborfluo por unu Fazo

1. Efektivigu nur la Fazon indikitan de la uzanto; ne komencu la sekvan.
2. Post la efektivigo rulu la taŭgajn kontrolojn (`just verify` aŭ tiujn
   indikitajn de la uzanto) kaj certigu, ke ili pasas.
3. Ĝisdatigu la version (laŭ la supra mapado; `bash tools/versio.sh
   --ĝisdatigi` por la dokumentoj), `docs/MEJLOSTONOJ.md` kaj
   `docs/VORTARO.md`.
4. **Haltu tuj**, raportu la rezulton al la uzanto kaj atendu pluajn
   instrukciojn.

## Commit-reguloj

- Sen klara permeso de la uzanto («vi povas commit»), **neniam** rulu
  `git commit` aŭ `git push`.
- Post la permeso, commitu ĉiujn ŝanĝojn de la nuna Fazo kaj puŝu ilin; la
  mesaĝo estu en Esperanto laŭ la stilo de la deponejo.
- Ne kunfandu plurajn Fazojn en unu commit, krom se la uzanto klare postulas
  tion.

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
  `docs/FLUOU.md`: fenestra modelo, Eniro (`Ctrl+K`) kaj Ligo.
- Ĝi ne estas normo; la deviga teksto restas `docs/FLUOU.md`. Se la gvidilo
  ŝanĝiĝas, ĝisdatigu la montron samtempe.
- La montro uzas la samajn ĵetonojn, materialojn kaj regilojn; ne enkonduku
  novan stilon nur por la montro.
