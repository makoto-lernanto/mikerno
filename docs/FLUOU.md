# FluoU — deviga gvidilo pri la grafika uzinterfaco de Horizonto

> Stato: 2026-11-19 · FluoU 0.1 · ABI 2 · MIT
>
> Ĉi tiu dokumento estas la norma gvidilo por ĉiu grafika programo sur
> Horizonto. Ĝi anstataŭas la interagan prototipon «FluoU.html»: la prototipo
> montris la direkton, sed nur ĉi tiu teksto estas deviga. Ĉio, kio ne estas
> permesita ĉi tie, ne apartenas al FluoU.
>
> Vida montro: [FLUOU-montro.html](FLUOU-montro.html) — nur demonstra; ĝi ne
> anstataŭas la normajn regulojn.

---

## 0. Normaj vortoj

| Vorto | Signifo |
| --- | --- |
| `DEVI` | absoluta postulo; malobservo malakceptas la aplikaĵon |
| `DEVI NE`, `MALPERMESITE` | absoluta malpermeso |
| `DEVUS` | forta rekomendo; deflankiĝo postulas skriban pravigon |
| `POVAS` | permeso |
| `KONFORMA` | plenumas ĉiujn `DEVI`-rulojn de ĉi tiu dokumento |

Ruloj havas identigilojn (`G`, `L`, `F`, `J`, `M`, `K`, `E`, `O`, `A`, `P`).
Oni ne renumeru ilin sen ĝisdatigi la konformkontrolon.

---

## 1. Celaro kaj la kvar kolonoj

FluoU (Fluo + Uzinterfaco) estas la sola permesita grafika kadro de Horizonto.
Ĝia moto estas **«Ĉio fluas.»** Ĉiu fenestro, ĉiu komponanto kaj ĉiu pikselo
fluas tra la sama kadro.

| ID | Kolono | Deviga signifo |
| --- | --- | --- |
| G01 | Unueco | Ĉiu grafika aplikaĵo `DEVI` esti konstruita per FluoU kaj deklari `gu = "fluoU"` en sia manifesto. |
| G02 | Kapablo | Renderado estas kapablo: sen `FENESTRO`-tenilo, ne ekzistas pikseloj. |
| G03 | Paritateco | La sama kompona arbo `DEVI` doni la saman rezulton sur `x86_64` kaj `arm64`. |
| G04 | Lingva unueco | Ĉiuj identigiloj, fontaj interfacaj tekstoj, dokumentaro kaj komandnomoj estas en Esperanto. |

- **G05.** Neniu aplikaĵo — ankaŭ sistema — rajtas desegni rekte en la
  frambufon. La sola escepto estas la kompozitoro mem.
- **G06.** La kompozitoro estas izolita servo kaj akceptas nur fenestrojn, kiuj
  posedas validan `FENESTRO`-tenilon ricevitan per `DONI`.
- **G07.** Ĉi tiu gvidilo venkas super prototipoj, ekrankopioj, kutimoj kaj
  historia kodo. Se fonto kaj gvidilo malkongruas, la gvidilo validas.
- **G08.** Programo, kiu aspektas kiel FluoU sed malobservas ĝian konduton,
  estas nekonforma eĉ se ĝi uzas la samajn kolorojn.

---

## 2. Lingva regulo

- **L01.** La fonta interfaco, la dokumentaro, la erarmesaĝoj, la komentoj kaj
  la identigiloj `DEVI` esti en Esperanto (vidu §2.2).
- **L02.** Restas en la angla nur neŝanĝeblaj propraj aŭ fiksaĵaj nomoj:
  `Rust`, `x86_64`, `arm64`, registraj nomoj, instrukcioj kaj normaj interfacaj
  kaj aparataraj nomoj.
- **L03.** Dum evoluigo oni verkas en Esperanto: kodo, fontaj interfacaj
  tekstoj, dokumentaro, mesaĝoj kaj dosiernomoj. Aldona plurlingva mapada
  tavolo rajtas provizi tradukojn de la montrataj tekstoj al aliaj lingvoj,
  inkluzive la ĉinan; vidu §2.2. La mapada tavolo estas aparta tavolo — ĝi ne
  anstataŭas la Esperantajn fontajn tekstojn.
- **L04.** Ĉiuj komandnomoj de Ligo `DEVI` esti Esperantaj. Anglaj aliasoj
  (`ls`, `cat`, `ps`, `kill`, `echo`, `help`, `clear` kaj similaj) estas
  malpermesitaj, ankaŭ kaŝe. Komandnomoj `DEVUS` esti sen diakritaj signoj,
  por ke ili restu tajpeblaj per simpla klavaro.
- **L05.** La interfaco ne rajtas miksi lingvojn en unu frazo aŭ unu etikedo.
  Traduku la tutan tekston, ne nur la videblan parton.
- **L06.** Erarmesaĝoj sekvas la formon
  `<komando>: <kaŭzo> (<kodo>)`, ekzemple
  `legi: dosiero ne ekzistas (-3)`.

### 2.1. Devigaj komandnomoj

| Malpermesita angla nomo | Deviga Esperanta nomo |
| --- | --- |
| `ls` | `listi` |
| `cat` | `legi` |
| `ps` | `procezoj` |
| `kill` | `mortigi` |
| `echo` | `eldiri` |
| `help` | `helpu` |
| `clear` | `purigi` |
| `grep` | `filtri` |
| `sort` | `ordigi` |
| `head` | `unua` |
| `wc` | `nombri` |
| `sum` | `sumi` |
| `theme` | `temo` |
| `accent` | `akcento` |

- **L07.** La prompto de Ligo enhavas la nunan dosierujon: `ligo:~>` en la
  hejmo, `ligo:~/notoj>` sub ĝi, alie `ligo:/…>` (ekzemple `ligo:/>`). La
  signo `~` estas `/hejmo` mem kaj malaperas, kiam `iri` forlasas ĝin. La
  helpa tabelo, la kompletigo kaj la historio uzas nur la Esperantajn nomojn.
- **L08.** La sistema tiparo `DEVI` kovri la Esperantajn literojn
  `ĉ ĝ ĥ ĵ ŝ ŭ` kaj la citilojn `« »` en ĉiu streko, kiun FluoU uzas.

### 2.2. Lingvaj tavoloj

- **Fonta tavolo (deviga).** Ĉio, kio estas verkata dum evoluigo — identigiloj,
  fontaj interfacaj tekstoj, komentoj, dokumentaro, mesaĝoj kaj dosiernomoj —
  estas en Esperanto.
- **Mapada tavolo (nedeviga).** Aparta traduka dosierujo, ekzemple `tradukoj/`,
  rajtas enhavi tradukojn de la montrataj tekstoj al aliaj lingvoj, inkluzive
  la ĉinan. Ĝi estas aldonita tavolo, ne anstataŭo de la fonta tavolo.
- La mapada tavolo ne rajtas ŝanĝi identigilojn, protokolajn nomojn,
  komandnomojn aŭ la strukturon de la interfaco; ĝi tradukas nur la montratajn
  tekstojn.
- Se la mapada tavolo mankas aŭ ne kovras iun tekston, la interfaco montras la
  Esperantan fontan tekston.

---

## 3. Fenestra kadro

La fenestra modelo sekvas **Windows**, ne macOS. Tio estas konscia decido: la
regiloj estas ortangulaj, senkoloraj kaj dekstre aliniiĝintaj, do uzantoj
rekonas ilin sen lerni novan simbolan lingvon. Malkiel en Windows, la titolo
estas horizontale centrita. La tri koloraj rondaj regiloj de macOS (la
«trafiklumoj») estas malpermesitaj.

### 3.1. Devigaj mezuroj

DIP signifas aparat-sendependan pikselon.

| Trajto | Valoro |
| --- | --- |
| Titolstria alto | 32 DIP |
| Aplika ikono | 16 × 16 DIP, maldekstre |
| Titola pozicio | horizontale centrita |
| Titola tiparo | 12 DIP, duondika |
| Regila grupo | dekstre aliniiĝinta, sen interspaco |
| Unu regilo | 46 × 32 DIP |
| Regila ikono | 10 × 10 DIP |
| Regila fono | travidebla en la normala stato |
| Fenestra radiuso | 8 DIP |
| Regrandiga rando | 4 DIP da nevidebla kapta zono |

### 3.2. Ordo kaj funkcio

De maldekstre al dekstre la grupo `DEVI` enhavi ekzakte tri regilojn:

1. **Malgrandigi** — malgrandigas la fenestron.
2. **Pligrandigi** aŭ **Restarigi** — ŝanĝas inter pligrandigita kaj antaŭa
   grando; la ikono kaj la konsileto ŝanĝiĝas kun la stato.
3. **Fermi** — fermas la fenestron.

- **F01.** Ĉiu fenestro `DEVI` havi titolstrian areon 32 DIP altan kun aplika
  ikono, titolo kaj la tri regiloj. La aplika ikono staru maldekstre, la
  titolo `DEVI` esti horizontale centrita, kaj la regiloj staru dekstre.
- **F02.** La regiloj `DEVI` stari dekstre, en la supra ordo, sen interspaco kaj
  sen propra radiuso.
- **F03.** La regiloj `DEVI` esti ortangulaj (sen rondaj anguloj) kaj
  senkoloraj en la normala stato. Koloraj rondoj, apartaj koloroj por
  malgrandigi kaj pligrandigi, aŭ regiloj maldekstre estas malpermesitaj.
- **F04.** La ikonoj estu: unu horizontala streko (malgrandigi), unu kvadrato
  (pligrandigi), du interkovritaj kvadratoj (restarigi) kaj unu ikso (fermi).
  La ikona streko estu 1,2 DIP dika.
- **F05.** En la ŝveba stato la neŭtralaj regiloj ricevu fonon de `--teksto` ĉe
  6% da opakeco; la premata stato uzu 3% da opakeco.
- **F06.** La regilo **Fermi** `DEVI` ricevi ruĝan ŝveban fonon `#C42B1C` kun
  blanka ikono; la premata fono estu `#B02A22`. Ruĝo aperas nur ĉe tiu regilo.
- **F07.** La titolstrio estu trenareo. Duobla klako `DEVI` ŝanĝi inter
  pligrandigita kaj antaŭa grando. La aplika ikono `POVAS` malfermi la sisteman
  menuon.
- **F08.** La sisteman menuon oni malfermu per `Alt+Space` aŭ per dekstra klako
  sur la titolstrio. Ĝi `DEVI` enhavi la agojn `Restarigi`, `Malgrandigi`,
  `Pligrandigi` kaj `Fermi`, kaj ankaŭ la kutimajn fenestrajn agojn. La menuo
  `DEVI` montri la `FENESTRO`-tenilon de la fenestro (A09): jen la vojo por
  kontroli la tenilon, ĉar la titolstrio ne portas `TENILO`-ŝildeton.
- **F09.** `Alt+F4` `DEVI` fermi la aktivan fenestron. `Win+←/→/↑/↓` `DEVUS`
  uzi la fenestran alkroĉadon de Windows; la pligrandiga regilo `DEVUS` montri
  la alkroĉajn aranĝojn en la ŝveba stato.
- **F10.** Fenestro sen fokuso montru la titolon per `--teksto3`, sed la
  regiloj restu videblaj kaj funkciantaj. La fenestro ne rajtas kaŝi sian
  ferman regilon.
- **F11.** `Fermi` `DEVI` esti atingebla ankaŭ per klavaro kaj per la sistema
  menuo; ĝi `DEVI` havi alireblan nomon `Fermi`.
- **F12.** En la alta kontrasta reĝimo la regiloj uzu la sistemajn kolorojn,
  unu-pikselan limon kaj neniun travideblon.
- **F13.** Propra titolstrio estas permesita nur se ĝi kopias ekzakte la
  mezurojn, la ordon, la statojn kaj la klavarajn agojn de ĉi tiu sekcio. Oni
  ne rajtas forigi, reordigi aŭ kaŝi la tri regilojn.
- **F14.** La konsiletoj estu Esperantaj: `Malgrandigi`, `Pligrandigi`,
  `Restarigi`, `Fermi`.

### 3.3. Skemo

```text
┌──────────────────────────────────────────────────────────────┬────┬────┬────┐
│ [ikono]                Titolo — FluoU                        │ —  │ □  │ ✕  │
└──────────────────────────────────────────────────────────────┴────┴────┴────┘
                                ↑ centro de la titolstrio
                                                                 malgrandigi
                                                                 pligrandigi
                                                                 fermi
```

La titolo estas horizontale centre de la tuta strio — ne apud la ikono; la
maldekstra areo estas rezervita por la ikono kaj eventualaj etikedoj. La
regiloj estas unu kontinua grupo sen videblaj apartigiloj. La fenestra
radiuso (8 DIP) tranĉas la supran dekstran angulon; la regiloj mem ne havas
propran radiuson.

### 3.4. Malpermesitaj modeloj

- tri koloraj rondaj «trafiklumoj» maldekstre;
- regiloj kun malsamaj koloroj aŭ formoj en la normala stato;
- kaŝita aŭ malŝaltita ferma regilo;
- propra titolstrio sen la normaj tri regiloj;
- movo de la regiloj al la maldekstra flanko;
- fenestra fono, kiu ŝanĝas la kontraston de la titolo.

### 3.5. Helpaj surfacoj

La komanda paletro `Eniro` kaj la terminala fenestro `Ligo` sekvas la saman
titolstrian skemon kiel la fenestroj.

- **F15.** La titolstrio de helpa surfaco estu 32 DIP alta: aplikaja aŭ rola
  ikono 16 × 16 DIP maldekstre, titolo 12 DIP duondika horizontale centre, kaj
  stato aŭ reĝimo dekstre. La titolo ne rajtas esti maldekstre aliniiĝinta.
- **F16.** `Ligo` estas fenestro: ĝi `DEVI` montri la tri regilojn de §3.2,
  inkluzive de `Fermi`, kaj la stato staru maldekstre de ili. Oni ne rajtas
  kaŝi aŭ forigi la regilojn (F13).
- **F17.** La komanda paletro `Eniro` ne estas memstara fenestro kaj ne montru
  la tri regilojn; ĝi fermiĝu per `Esc` (E06) kaj per klako sur la malantaŭa
  kurteno.

---

## 4. Ĵetonoj

Ĵetonoj estas la sola fonto de koloroj, tipografio, geometrio, movo kaj
denseco. Aplikaĵo `DEVI NE` fiksi kolorojn aŭ mezurojn rekte, krom en la
difino de la ĵetonoj mem.

### 4.1. Koloroj

| Ĵetono | Malluma | Hela | Rolo |
| --- | --- | --- | --- |
| `--fondo` | `#0A0D1C` | `#F6F3EC` | Ĝenerala fono |
| `--fondo2` | `#10142A` | `#EFE9DF` | Profunda fono |
| `--surfaco` | `#171C36` | `#FFFFFF` | Karto, panelo |
| `--surfaco2` | `#1F2547` | `#F4EFE6` | Dueca surfaco |
| `--teksto` | `#EEF0FF` | `#20223A` | Ĉefa teksto |
| `--teksto2` | `#A7ADCC` | `#565B7C` | Dueca teksto |
| `--teksto3` | `#6D7396` | `#8B8FA8` | Helpa teksto |
| `--limo` | `rgba(148,163,255,.16)` | `rgba(60,60,110,.18)` | Videbla limo |
| `--limo2` | `rgba(148,163,255,.09)` | `rgba(60,60,110,.10)` | Subtila limo |
| `--oro` | `#FFB454` | `#FFB454` | Suna akcento |
| `--koralo` | `#FF5C8A` | `#FF5C8A` | Krepuska akcento |
| `--ĉielo` | `#4CC9FF` | `#4CC9FF` | Ĉiela akcento |
| `--verdo` | `#57E389` | `#57E389` | Natura akcento |
| `--viola` | `#A78BFA` | `#A78BFA` | Dua akcento |
| `--bluo` | `#5B7CFA` | `#5B7CFA` | Profunda bluo |
| `--sukceso` | `#57E389` | `#57E389` | Sukceso |
| `--averto` | `#FFB454` | `#FFB454` | Averto |
| `--danĝero` | `#FF6B6B` | `#FF6B6B` | Danĝero |
| `--informo` | `#4CC9FF` | `#4CC9FF` | Informo |

La ses akcentaj koloroj estas la permesitaj antaŭagordoj: `oro`, `koralo`,
`ĉielo`, `verdo`, `viola` kaj `bluo`. Propra akcento `POVAS` ekzisti, sed ĝi
`DEVI` derivi kongruan duan koloron kaj plenumi la kontrastajn postulojn.

### 4.2. Geometrio kaj denseco

| Ĵetono | Defaŭlto | Permesita intervalo |
| --- | --- | --- |
| `--radiuso` | 8 DIP | 0–16 DIP |
| `--radiuso-granda` | 12 DIP | derivita |
| `--radiuso-pilolo` | 99 DIP | fiksa |
| `--densa-pad` | 14 DIP | 10 / 14 / 18 DIP |
| `--elevacio` | 0,6 | 0–1 |
| `--tipara-skalo` | 1,0 | 0,85–1,25 |
| `--akra-nebulo` | 24 DIP | 0–40 DIP |
| `--akra-opakeco` | 50% | 20–92% |
| `--revelo` | 55% | 0–100% |
| `--grajno` | 1,0 | 0–1 |

Denseco havas tri nivelojn: `kompakta`, `norma` kaj `komforta`. Ĉiuj komponantoj
`DEVI` sekvi la aktivan densecon sen apartaj esceptoj.

### 4.3. Tipografio

| Rolo | Grando / pezo | Uzo |
| --- | --- | --- |
| `MONTRO` | 64 / 850 | Heroa titolo |
| `TITOLO` | 32 / 800 | Sekcia titolo |
| `SUBTITOLO` | 20 / 700 | Karta titolo |
| `KORPO` | 16 / 400 | Ĉefa teksto |
| `ETIKEDO` | 13 / 600 | Kampaj etikedoj |
| `PRISKRIBO` | 12 / 500 | Helpa teksto |
| `MONO` | 12 / 400 | Kodaj kaj teknikaj valoroj |

- **J01.** La tipara skalo `DEVI` esti uzata kiel tuto; oni ne rajtas elekti
  hazardajn grandojn.
- **J02.** La minimuma videbla teksto estas 12 DIP; la ĉefa teksto estas 16 DIP.
- **J03.** La tipara stako `DEVI` enhavi la Esperantajn literojn kaj la
  citilojn; se la unua tiparo mankas, la alternativa tiparo `DEVI` aspekti
  egale.

### 4.4. Movo

| Ĵetono | Valoro |
| --- | --- |
| `--mov-daŭro` | 1,0 (0,2–2,0) |
| `--mov-kurbo` | `cubic-bezier(.16, 1, .3, 1)` |
| Mallonga movo | 120 ms × `--mov-daŭro` |
| Meza movo | 200 ms × `--mov-daŭro` |
| Longa movo | 320 ms × `--mov-daŭro` |

- **J04.** Kiam la sistemo postulas malpli da movado, ĉiuj transformoj kaj
  animacioj `DEVI` malŝaltiĝi aŭ daŭri plej multe 1 ms.
- **J05.** Movo `DEVI NE` esti la sola transportilo de informo; stato ĉiam havu
  ankaŭ tekston, ikonon aŭ koloron.

### 4.5. Kontrasto kaj alirebleco

- **J06.** Ĉefa teksto `DEVI` atingi kontrastan rilatumon de almenaŭ 4,5:1;
  granda teksto almenaŭ 3:1.
- **J07.** Fokuso `DEVI` esti videbla per 2 DIP dika ringo kaj neniam per koloro
  sola.
- **J08.** Ĉiuj interagaj celoj `DEVI` havi alton de almenaŭ 32 DIP; sur
  tuŝekranoj la celo `DEVUS` esti almenaŭ 40 × 40 DIP.
- **J09.** La interfaco `DEVI` resti uzebla ĉe skalo de 200% kaj en la alta
  kontrasta reĝimo.
- **J10.** La akcento `DEVI NE` rompi la kontraston de teksto aŭ de fokusa
  ringo.

### 4.6. Grajno kaj elevacio

La grajno estas subtila brua tavolo (defaŭlte 5,5% da opakeco). Ĝi `DEVUS`
malŝaltiĝi en la alta kontrasta reĝimo kaj sur malpotencaj aparatoj. Elevacio
uzas unu el kvin niveloj (`0`–`4`); ombroj `DEVI` veni el `--elevacio`, ne el
lokaj valoroj.

---

## 5. Materialoj

Estas ekzakte tri materialoj. Iliaj nomoj estas propraj kaj restas neŝanĝitaj.

| Materialo | Difino | Uzo |
| --- | --- | --- |
| **Mika** | Netravidebla bazo kun 7% da akcenta nuanco el la fonbildo | Defaŭlta fenestra materialo |
| **Akrila** | Fona nebulo (`--akra-nebulo`, 24 DIP) + saturiĝo 1,5 + grajno | Transiraj surfacoj: Eniro, dialogoj, ŝprucfenestroj |
| **Revelo** | Montrilsekva randobrilo (`--revelo`, 55%) | Interagaj kartoj kaj randaj emfazoj |

- **M01.** Ĉiu fenestro `DEVI` uzi `Mika` kiel bazon, krom se ĝi estas tute
  transira surfaco.
- **M02.** `Akrila` `DEVI NE` kovri la tutan ekranon aŭ longan tekston; ĝi
  taŭgas nur por mallongdaŭraj surfacoj.
- **M03.** `--akra-nebulo` estas 24 DIP (0–40); `--akra-opakeco` estas 62%
  (20–92%). Valoroj ekster tiuj intervaloj estas malpermesitaj.
- **M04.** `--revelo` estas 55% (0–100). Revelo `DEVI NE` malhelpi tekston,
  fokuson aŭ kontraston.
- **M05.** Se la platformo ne subtenas fonan nebulon, `Akrila` `DEVI` uzi
  anstataŭe netravideblan `--surfaco2`.
- **M06.** Oni ne rajtas elpensi novan materialon aŭ uzi hazardan nebulon.
  Nur `Mika`, `Akrila` kaj `Revelo` ekzistas.

---

## 6. Komponantoj

La komponantoj estas la publika interfaco de FluoU. Aplikaĵo `DEVI` uzi ilin anstataŭ
desegni proprajn regilojn.

### 6.1. Ĝeneralaj reguloj

- **K01.** Ĉiu interaga komponanto `DEVI` havi la statojn `normala`, `ŝveba`,
  `premata`, `fokusa`, `malŝaltita` kaj, se koncerne, `elektita`.
- **K02.** Ĉiu komponanto `DEVI` esti plene uzebla per klavaro: `Tab` por
  navigado, `Space`/`Enter` por agado, sagoj por listoj, langetoj kaj glitiloj.
- **K03.** Ĉiu komponanto `DEVI` havi alireblan nomon, rolon kaj staton por
  ekranlegiloj. Ikonaj butonoj `DEVI` havi tekstan alternativon.
- **K04.** Fokuso `DEVI` resti videbla kaj ne rajtas esti forigita per
  `outline: none` sen anstataŭo.
- **K05.** La komponantoj `DEVI` sekvi la ĵetonojn de temo, akcento, denseco,
  tipara skalo kaj movado.

### 6.2. Devigaj komponantoj

| Komponanto | Devigaj trajtoj |
| --- | --- |
| `Buto` | Kvar stiloj: `Akcenta`, `Norma`, `Subtila`, `Danĝera`; alto ≥ 32 DIP; granda varianto 40 DIP. |
| `Ŝaltilo` | Trako 42 × 23 DIP; kubuto 13 DIP; ŝveba stretiĝo; teksta stato `ŝaltita`/`malŝaltita`. |
| `Glitilo` | Trako 5 DIP; kubuto 19 DIP; klavaraj sagoj; viva valoro. |
| `Tekstkampo` | Etikedo ĉiam videbla; fokusa indiko 2 DIP ĉe la suba rando; erarmesaĝo sub la kampo. |
| `Elektilo` | Etikedo, elektita valoro kaj klavara navigado; malfermo per `Space`/`Enter`. |
| `Langetoj` | Sagoj maldekstre/dekstre; elektita indikilo 2,5 DIP; panelo ŝanĝiĝas sen repaĝigo. |
| `ProgresStrio` kaj `ProgresRingeto` | Sama stata fonto; difinita kaj nedifinita reĝimoj; paŭzo kaj rekomenco. |
| `InformaStrio` | Kvar semantikoj: `Sukceso`, `Averto`, `Danĝero`, `Informo`; fermebla; restaurebla. |
| `Ŝildeto` | Montras tenilon, permesojn aŭ ABI-version; teksto ne estas tranĉita sen konsileto. |
| `Listo` | Unuopa elekto; klavara navigado; meta-informo dekstre. |
| `Dialogo` | Modala; fokuso kaptita; `Esc` fermas; klako sur la fono fermas; fokuso revenas al la antaŭa elemento. |
| `Fulmomesaĝo` | Maksimume kvar samtempe; 3,4 s; `aria-live="polite"`; neniam la sola kanalo por kritika eraro. |

- **K06.** Detrua ago `DEVI` uzi la stilon `Danĝera` kaj postuli konfirmon, se
  ĝi ne estas malfarebla.
- **K07.** `Dialogo` `DEVI NE` fermiĝi per la fono aŭ per `Esc`, se ĝi enhavas
  nekonservitajn datumojn; ĝi `DEVI` peti konfirmon.
- **K08.** `Fulmomesaĝo` `DEVI NE` anstataŭi eraran staton, kiu restas videbla
  en la koncerna komponanto.
- **K09.** La komponantoj `DEVI` esti funkcie identaj en ambaŭ temoj kaj en
  ĉiuj densecoj.
- **K10.** La interfaco de la komponantoj `DEVI` esti Esperanta: `Buto`, `Ŝaltilo`,
  `Glitilo`, `Tekstkampo`, `Elektilo`, `Langetoj`, `ProgresStrio`,
  `ProgresRingeto`, `InformaStrio`, `Ŝildeto`, `Listo`, `Dialogo`,
  `Fulmomesaĝo`.

---

## 7. Eniro

Eniro estas la tutsistema komanda paletro. Ĝi estas FluoU-aplikaĵo kun
`Akrila` materialo kaj `Revelo` emfazo.

- **E01.** `Ctrl+E` `DEVI` malfermi Eniron el ĉiu aplikaĵo; `Esc` fermas ĝin.
  **Noto.** Tiu ĉi klavo estas nur provizora solvo: la efektiva klavkombino
  de Horizonto estos la tradicia `Win`-klavo, kiun ni nomas **Mito**.
- **E02.** Eniro `DEVI` subteni tri reĝimojn:
  1. ordinara serĉo laŭ Esperantaj nomoj, identigiloj kaj priskriboj;
  2. kalkulo post la prefikso `=`;
  3. delegado al Ligo post la prefikso `>`.
- **E03.** La kalkulilo `DEVI` uzi sekuran espriman analizilon sen `eval` kaj
  sen aliro al la dosiersistemo aŭ la reto.
- **E04.** La serĉo `DEVI` funkcii senrete, prioritatigi laste uzitajn erojn kaj
  ne sendi demandojn eksteren.
- **E05.** La komanda registro `DEVI` esti komuna kun Ligo: unu cerbo, du
  surfacoj. Komando aldonita al Ligo `DEVI` aperi ankaŭ en Eniro.
- **E06.** La klavaraj agoj estu: `↑↓` navigi, `Enter` ruli, `Tab` kompletigi,
  `Esc` fermi.
- **E07.** Aplikaĵoj lanĉitaj el Eniro `DEVI` ricevi nur la minimumajn
  kapablojn per `DONI`; kolapso de unu aplikaĵo `DEVI NE` haltigi Eniron aŭ la
  kompozitoron.
- **E08.** Ĉiu rezulto `DEVI` havi Esperantan fontan nomon kaj alireblan
  priskribon. Komandnomoj kaj identigiloj restas Esperantaj; la mapada tavolo
  (§2.2) tradukas nur la montratajn tekstojn.
- **E09.** La piedlinio `DEVI` montri la klavarajn helpojn en Esperanto kaj la
  version de FluoU.

---

## 8. Ligo

Ligo estas la objekta ŝelo de Horizonto: ĝi transdonas **tipizitajn
rekordojn**, ne tekston. La terminalo estas FluoU-aplikaĵo.

La filozofio similas PowerShell 7-on: la dukto portas objektojn, ne tekston;
ĉiu stadio ricevas kaj redonas rekordojn. En la interaga reĝimo la rezultoj
(ekzemple `listi`) estas redonitaj kiel **koloraj tabeloj** laŭ la ĵetonoj kaj
la semantikaj roloj de la kampoj; la koloroj nur akompanas la signifon —
tipo, grandeco, tenilo kaj eraroj distingiĝu ankaŭ sen koloro.

### 8.1. Komandaro

Ĉiuj komandnomoj estas Esperantaj kaj devigaj. Komandnomoj kaj argumentoj
konsistas nur el puraj ASCII-literoj: oni uzu Esperantajn vortojn, kiuj jam
havas neniun ĉapelan literon (ĉ, ĝ, ĥ, ĵ, ŝ, ŭ), kaj neniam la x-sistemon
(`cx`, `gx`, …). La tabelo listigas nur la Esperantajn nomojn; la anglaj formoj
ne ekzistas kaj ne estas tajpeblaj.

| Komando | Signifo | Rezulto |
| --- | --- | --- |
| `helpu` | Montri la komandtabelon kaj la helpon | Tabelo |
| `listi [vojo]` | Listigi dosierojn (defaŭlte la nuna dosierujo) | `nomo`, `tipo`, `grandeco`, `tenilo`; en interago kolora tabelo |
| `legi <dosiero>` | Legi dosieron | Dosiera karto kun `tenilo` |
| `iri [vojo]` | Ŝanĝi la nunan dosierujon (defaŭlte `~`) | Sukceso aŭ eraro |
| `kie` | Montri la nunan dosierujon | Skalaro |
| `procezoj` | Montri la procezan tabelon | `pid`, `nomo`, `stato`, `memoro`, `permesoj` |
| `mortigi <pid>` | Mortigi procezon | Sukceso aŭ eraro |
| `ruli <dosiero>` | Ruli ELF-bildon el SNDS kiel novan procezon | Sukceso aŭ eraro |
| `eldiri <teksto>` | Krei mesaĝan rekordon | `noto` |
| `pri` | Montri sistemajn informojn | `kampo`, `valoro` |
| `temo <malluma\|hela>` | Ŝanĝi la temon | Sukceso |
| `akcento <oro\|koralo\|lazuro\|verdo\|viola\|bluo>` | Ŝanĝi la akcenton | Sukceso |
| `eniro` | Malfermi Eniron | Sukceso |
| `filtri <kampo> <operacio> <valoro>` | Filtri rekordojn | Rekordoj |
| `ordigi <kampo> [malkreske]` | Ordigi rekordojn | Rekordoj |
| `unua <n>` | Preni la unuajn `n` rekordojn | Rekordoj |
| `sumi <kampo>` | Sumi nombran kampon | Skalaro |
| `nombri` | Nombri rekordojn | Skalaro |
| `montri [kampoj…]` | Projekcii kolumnojn | Rekordoj |
| `teksto` | Transiri al la eksplicita teksta limo | Linioj |
| `purigi` | Malplenigi la terminalon | — |

`lazuro` estas la Ligo-nomo de la helblua FluoU-akcento (`#4CC9FF`). Mesaĝa
rekordo havas unu kampon, `noto`.

La ŝelo startas en `/hejmo`. Ĉiu voja argumento estas solvata antaŭ la
sistemvoko: `~` kaj `~/…` signifas `/hejmo`, relativaj vojoj komenciĝas en la
nuna dosierujo, `.` malaperas kaj `..` forprenas komponanton sed neniam
eliras `/`. Nur la normaligita absoluta vojo atingas SNDS.

La malpermesitaj anglaj nomoj (`ls`, `cat`, `ps`, `kill`, `echo`, `help`,
`clear`) ne ekzistas, eĉ ne kiel kaŝitaj aliasoj. Provon uzi ilin oni raportu
per `Nekonata komando: «ls» (-2) — tajpu helpu`.

### 8.2. Gramatiko

```text
komando [argumentoj] ( | stadio [argumentoj] )*
stadio  := filtri | ordigi | unua | sumi | nombri | montri | teksto
```

- **O01.** La dukto `DEVI` transdoni rekordojn, ne tekstajn liniojn. Ĉiu
  stadio ricevas kaj redonas kompletan aron da rekordoj.
- **O02.** La komandoj `DEVI NE` ruli ĝeneralan ŝelan interpretilon nek
  `eval`-funkcion. Argumentoj estas analizataj laŭ la fiksa gramatiko.
- **O03.** Ĉiu rekordo el `listi` `DEVI` porti nurlegan `tenilo`-kampon; la
  terminalo montras ĝin per la ŝildeto `TENILO`.
- **O04.** La kampo `tenilo` `DEVI` esti transdonita laŭ la kapabla modelo de
  `DONI`; rekordo ne rajtas elmontri fremdan aŭ revokitan tenilon.
- **O05.** `teksto` estas la sola eksplicita limo inter objektoj kaj teksto.
  Sen ĝi la eligo restas tabelo aŭ skalaro.
- **O06.** La rega kanalo uzas `Mesaĝo` de 64 bajtoj por skemo, fino kaj
  eraro; grandaj objektoj iras per kunhava memora ringo. La du vojoj ne
  miksiĝas.
- **O07.** `mortigi` `DEVI` rifuzi la procezojn `pid ≤ 3` per la mesaĝo
  `procezo <pid> estas protekta sistema procezo (-1)`.
- **O08.** Ĉiu eraro `DEVI` havi Esperantan mesaĝon kaj kodan numeron; la
  helpo estu atingebla per `helpu`.
- **O09.** La historio (`↑↓`), la kompletigo (`Tab`) kaj la malplenigo
  (`Ctrl+L`) `DEVI` funkcii sen reto kaj sen konfuziĝi pro nekompletaj komandoj.
- **O10.** `temo` kaj `akcento` `DEVI` ŝanĝi la tutan FluoU-interfacon tuj kaj
  memori la elekton por la uzanto.
- **O11.** `pri` `DEVI` raporti la version de FluoU, la ABI-version, la kernon
  kaj la arkitekturojn.
- **O12.** Skalaraj stadioj (`sumi`, `nombri`) `DEVI` montri unu klaran valoron
  kun etikedo; ili ne ŝajnigu tabelon.
- **O13.** En la interaga reĝimo `listi` `DEVI` montri koloran tabelon laŭ la
  ĵetonoj kaj semantikaj roloj (ekzemple `nomo` en la ĉefa teksta rolo,
  `tipo`/`grandeco` en la kaska rolo, `tenilo` kiel ŝildeto); koloroj nur
  akompanas la signifon kaj neniam estas la sola portanto de informo.

### 8.3. Ekzemploj

Ĉiu komando havas ekzemplon; la stadioj funkcias nur post `listi` aŭ
`procezoj`.

| Komando | Ekzemplo |
| --- | --- |
| `helpu` | `helpu` |
| `listi [vojo]` | `listi /notoj` |
| `legi <dosiero>` | `legi bonveno.txt` |
| `iri [vojo]` | `iri ../notoj` |
| `kie` | `kie` |
| `procezoj` | `procezoj` |
| `mortigi <pid>` | `mortigi 2` |
| `ruli <dosiero>` | `ruli /saluto.elf` |
| `eldiri <teksto>` | `eldiri saluton` |
| `pri` | `pri` |
| `temo <malluma\|hela>` | `temo hela` |
| `akcento <nomo>` | `akcento verdo` |
| `eniro` | `eniro` |
| `filtri <kampo> <operacio> <valoro>` | `listi \| filtri tipo == dosiero` |
| `ordigi <kampo> [malkreske]` | `listi \| ordigi grandeco malkreske` |
| `unua <n>` | `procezoj \| unua 3` |
| `sumi <kampo>` | `listi \| sumi grandeco` |
| `nombri` | `listi \| nombri` |
| `montri [kampoj…]` | `procezoj \| montri pid nomo memoro` |
| `teksto` | `procezoj \| teksto` |
| `purigi` | `purigi` |
| `fini [kodo]` | `fini` |

```text
listi | filtri grandeco > 100 | ordigi grandeco malkreske | montri
procezoj | filtri stato == Kuranta | montri pid nomo memoro
listi | sumi grandeco
listi | nombri
```

La unua dukto legiĝas: «Listigu la dosierojn, konservu tiujn pli grandajn
ol 100 KiB, ordigu ilin malkreske laŭ grandeco kaj montru la rezulton.»
`saluto.elf` venas de la lanĉilo (`just ruli` aŭ `just ruli-iso`) kaj kuŝas
en la radiko, do la ekzemplo uzas `ruli /saluto.elf`.

---

## 9. Arkitekturo kaj devigo

La devigo venas el la kapabla modelo de la kerno, ne el konvencio.

```text
Aplikaĵo
   │  FluoU-interfaco
   ▼
FluoU 0.1
   │  komponantoj · materialoj · movo
   ▼
libobjekto · libmikerno
   │  ABI 2 · Tenilo · Mesaĝo
   ▼
MiKerno ⇄ MiKerno-ARM
   │  int 0x80 / svc #0
   ▼
Kompozitoro (izolita servo)
   │  nur FENESTRO-teniloj
   ▼
Ekrano
```

- **A01.** La ŝargilo `DEVI` malakcepti aplikaĵon, kies manifesto ne enhavas
  `gu = "fluoU"`.
- **A02.** La manifesto `DEVI` deklari la kadran version (`fluo`) kaj la
  subtenatajn arkitekturojn. Nekongrua ĉefversio `DEVI` esti malakceptita.
- **A03.** La kompozitoro `DEVI NE` akcepti fenestron sen valida
  `FENESTRO`-tenilo. Sen tenilo ne ekzistas pikseloj.
- **A04.** `DONI` `DEVI` nur malpliigi permesojn; `REVOKI` `DEVI` revoki la
  tutan finpunkton; `FERMI` `DEVI` fermi nur la lokan tenilon. Generacia
  kontrolo `DEVI` malhelpi la reviviĝon de malnova tenilo.
- **A05.** Ĉiu aplikaĵo `DEVI` kuri izolite. Kolapso de unu procezo `DEVI NE`
  detrui la kompozitoron aŭ aliajn fenestrojn.
- **A06.** La sama kompona arbo `DEVI` produkti piksele saman rezulton sur
  `x86_64` kaj `arm64`. La DI `DEVI` kompari fiksaĵojn de la frambufo kaj
  malsukcesi ĉe iu ajn diferenco, krom dokumentita platforma escepto.
- **A07.** La komuna ABI `DEVI` resti kongrua inter `libmikerno`, la kerno kaj
  FluoU: `Tenilo`, `Mesaĝo` kaj la voknumeroj de ABI 2. La registra aranĝo
  estas platforma (`rax`/`rdi`/`rsi`/`rdx`/`r10` aŭ `x8`/`x0`–`x3`), la
  semantiko komuna.
- **A08.** La manifesto `DEVUS` esti subskribita kaj kontrolita antaŭ ŝargo;
  la kapabloj `DEVI` esti la minimuma aro, kiun la aplikaĵo bezonas.
- **A09.** Ĉiu fenestro `DEVI` esti spurebla en la protokolo: aplikaĵo, tenilo,
  generacio, permesoj kaj vivdaŭro. La uzanto `POVAS` vidi kaj revoki ilin.
- **A10.** Neniu aplikaĵo havas implicitajn permesojn al la reto, la
  dosiersistemo aŭ la ekrano. Ĉiu aliro `DEVI` esti deklarita kaj akirita per
  kapablo.

### 9.1. Manifesto

```toml
# aplikaĵa manifesto — la ŝargilo devigas la kampojn
[aplikaĵo]
nomo    = "Notilo"
versio  = "1.0.0"
gu      = "fluoU"                    # deviga
fluo    = "0.1"                      # kadra versio
arkoj   = ["x86_64", "arm64"]        # tago unu

[permesoj]
fenestro = true                      # FENESTRO-tenilo per DONI ↓
dosieroj = ["~/dokumentoj:legi"]
reto     = false                     # ne petita → ne ekzistas
```

### 9.2. Rust-interfaco

```rust
use fluou::{Fenestro, Mika, Buto, Ŝaltilo, Stilo};

Fenestro::nova("Agordoj")
    .materialo(Mika)
    .grandeco(520, 380)
    .enhavo((
        Ŝaltilo::nova("Aŭtomata ĝisdatigo")
            .ŝaltita(true),
        Buto::nova("Konservi")
            .stilo(Stilo::Akcenta)
            .klake(|_| agordoj.konservi()),
    ))
    .montri()   // → FENESTRO-tenilo → Kompozitoro
```

- **A11.** La interfaco `DEVI` esti Esperanta kaj ne elmeti krudajn fenestrajn
  identigilojn; la programisto ricevas `Fenestro`-objekton, ne frambufon.
- **A12.** La interfaco `DEVI` havi stabilan version kaj dokumentitajn rompajn
  ŝanĝojn nur ĉe pliigo de la ĉefversio.

---

## 10. Konformkontrolo

Aplikaĵo estas `KONFORMA` nur se ĉiuj jenaj kontroloj estas verdaj. Malsukceso
ĉe iu ajn `P`-ero blokas la eldonon.

| ID | Kontrolo |
| --- | --- |
| P01 | Manifesto sen `gu = "fluoU"` estas malakceptita. |
| P02 | Aplikaĵo sen `FENESTRO`-tenilo ne desegnas pikselojn. |
| P03 | La fenestra titolstrio havas la mezurojn, la ordon kaj la tri regilojn de §3; la helpaj surfacoj sekvas §3.5. |
| P04 | Neniu trikolora ronda «trafiklumo» aperas ie ajn. |
| P05 | `Fermi` havas ruĝan ŝveban staton kaj estas atingebla per klavaro. |
| P06 | Ambaŭ temoj difinas ĉiujn ĵetonojn; mankanta ĵetono estas konstru-eraro. |
| P07 | Teksto plenumas la kontrastojn de §4.5. |
| P08 | Movado malŝaltiĝas aŭ mallongiĝas laŭ la sistema prefero. |
| P09 | Ĉiu komponanto estas plene uzebla per klavaro. |
| P10 | Fokuso estas ĉiam videbla kaj ne estas forigita sen anstataŭo. |
| P11 | Ĉiuj komandnomoj de Ligo estas Esperantaj; `ls` kaj la aliaj anglaj nomoj estas rifuzitaj. |
| P12 | La dukto transdonas tipizitajn rekordojn kaj montras `TENILO`-ŝildetojn. |
| P13 | `teksto` estas la sola eksplicita teksta limo. |
| P14 | `mortigi` rifuzas la protektajn procezojn `pid ≤ 3`. |
| P15 | Ĉiu erarmesaĝo estas Esperanta kaj havas kodan numeron. |
| P16 | `Dialogo` kaptas fokuson, fermeblas per `Esc` kaj redonas fokuson. |
| P17 | La piksela fiksaĵo kongruas inter `x86_64` kaj `arm64`. |
| P18 | Neniu aplikaĵo aliras la frambufon rekte. |
| P19 | Ĉiuj fontaj interfacaj tekstoj estas Esperantaj; aliaj lingvoj aperas nur per la aparta mapada tavolo de §2.2. |
| P20 | La eksporto de ĵetonoj al JSON, CSS kaj Rust rekreas la saman arbon. |

---

## 11. Vojmapo

| Versio | Enhavo |
| --- | --- |
| **FluoU 0.1 · nun** | Komponantoj, ĵetonoj kaj materialoj; Eniro kaj Ligo kiel la unuaj FluoU-aplikaĵoj; ĉi tiu gvidilo validas. |
| **FluoU 0.2** | Kompozitoro kaj kunhava memora kapablo; Eniro kaj Ligo kunhavigas la saman objektan fluon. |
| **FluoU 1.0** | FluoU eldoniĝas kun Horizonto; piksela akcepto sur ambaŭ arkitekturoj; SDK kaj dokumentaro frostigitaj. |

---

## Aldono A. Listo de malpermesaĵoj

La jenaj aferoj estas malpermesitaj sendepende de tio, ĉu ili aspektas bone:

- la macOS-trikoloraj trafiklumoj aŭ iu ajn kolorigita ronda regilo;
- forigita, kaŝita aŭ reordigita ferma regilo;
- propra titolstrio sen la normaj tri regiloj;
- anglaj komandnomoj aŭ kaŝaj aliasoj en Ligo;
- miksado de pluraj lingvoj en unu fonta teksto anstataŭ aparta mapada tavolo (§2.2);
- rekta frambufa aliro, krudaj pikseloj aŭ neadministritaj fenestroj;
- koloroj, mezuroj kaj daŭroj ekster la ĵetonoj;
- `eval` aŭ ĝenerala ŝela interpretilo en Eniro aŭ Ligo;
- modala fenestro sen `Esc`, sen fokusa kapto aŭ sen fokusa reveno;
- kritika eraro nur kiel `Fulmomesaĝo`;
- akcento, kiu rompas kontraston aŭ fokuson;
- `Akrila` nebulo sub longa teksto;
- materialo, kiu ne nomiĝas `Mika`, `Akrila` aŭ `Revelo`.

---

## Aldono B. Historio de la dokumento

- **2026-11-19.** Ligo estas eksplicite PowerShell 7-simila objekta dukto: la
  objektoj fluas anstataŭ teksto; `listi` en interago devige montras koloran
  tabelon laŭ la ĵetonoj, kaj la prompto de Ligo renomiĝis al `ligo:~>`.
- **2026-11-14.** Unua norma versio. Derivita el la interaga prototipo
  «FluoU.html»; la fenestraj regiloj transiris al la Windows-modelo, kaj la
  tuta komandaro de Ligo estas Esperanta sen anglaj aliasoj.
- **2026-11-14.** Titoloj centritaj (ikono maldekstre, regiloj dekstre); §3.5
  por la helpaj surfacoj `Eniro` kaj `Ligo`; la komando por krei mesaĝan
  rekordon anstataŭiĝis per `eldiri`, kaj komandnomoj evitu diakritajn signojn.
- **2026-11-14.** Lingva regulo klare disigita en du tavolojn (§2.2): evoluigo
  devige en Esperanto; nedeviga mapada tavolo rajtas traduki la montratajn
  tekstojn, inkluzive al la ĉina. La absoluta malpermeso de la ĉina lingvo
  forigita el §2 kaj Aldono A.
