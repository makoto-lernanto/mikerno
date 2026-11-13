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
- La ĉina lingvo estas malpermesita ie ajn en la deponejo: kodo, dokumentaro,
  mesaĝoj, dosiernomoj.

## Versioj

La versio havas la formon `0.x.y` kaj rilatas al la mejloŝtonoj:

- Mejloŝtono **X** → `x = X − 10`
- **Fazo Y** en tiu mejloŝtono → `y = Y − 1`

Ekzemple: la Fazoj 1/2/3 de M13 respondas al `0.3.0` / `0.3.1` / `0.3.2`.
La formo `0.x.y` validas ekde M10; M0–M9 apartenas al la historio `0.0.x`.

Nuna versio: `0.4.2` (M14, Fazo 3).

Lokoj, kie la versio devas resti kongrua:

- `Cargo.toml` — `[workspace.package] version`
- `mikerno/src/kerno/mod.rs` — la saluta linio `Saluton, mi estas MiKerno v0.x.y`
- `README.md` — `**Versio:** 0.x.y (workspace)`
- `docs/MEJLOSTONOJ.md` — `Versio de la workspace` en la kapo kaj la koncerna
  mejloŝtona ero

## Laborfluo por unu Fazo

1. Efektivigu nur la Fazon indikitan de la uzanto; ne komencu la sekvan.
2. Post la efektivigo rulu la taŭgajn kontrolojn (`just verify` aŭ tiujn
   indikitajn de la uzanto) kaj certigu, ke ili pasas.
3. Ĝisdatigu la version (laŭ la supra mapado), `docs/MEJLOSTONOJ.md` kaj
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
