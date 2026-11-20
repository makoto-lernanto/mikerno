"use strict";

// Konduto de la retejo: versio, temo, akcentoj, lingvoj, fenestroj, eligo,
// Eniro kaj la FluoU-movoj. La fonta lingvo estas Esperanto; la angla venas
// nur el `tradukoj/angla.js` kiel aparta mapada tavolo.

(() => {
  const radiko = document.documentElement;
  const VERSIO = radiko.dataset.versio || "0.7.3";
  const ANGLA = window.TRADUKOJ_ANGLA || {};
  const reduktitaMovo = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  const AKCENTOJ = {
    oro: "#FFB454",
    koralo: "#FF5C8A",
    ĉielo: "#4CC9FF",
    verdo: "#57E389",
    viola: "#A78BFA",
    bluo: "#5B7CFA",
  };
  const AKCENTAJ_NOMOJ = {
    oro: "oro",
    koralo: "koralo",
    ĉielo: "ĉielo",
    verdo: "verdo",
    viola: "viola",
    bluo: "bluo",
  };
  const GRUPOJ = {
    sekcioj: ["eniro.grupo.sekcioj", "Sekcioj"],
    agordoj: ["eniro.grupo.agordoj", "Agordoj"],
    ligoj: ["eniro.grupo.ligoj", "Ligoj"],
  };

  const $ = (selektilo, patro = document) => patro.querySelector(selektilo);
  const $$ = (selektilo, patro = document) => Array.from(patro.querySelectorAll(selektilo));

  let lingvo = "eo";
  let originalaTitolo = document.title;
  const originaloj = new Map();

  function traduki(ŝlosilo, originalo) {
    if (lingvo === "en" && typeof ANGLA[ŝlosilo] === "string") {
      return ANGLA[ŝlosilo];
    }
    return originalo;
  }

  function t(ŝlosilo, defaŭlto) {
    return traduki(ŝlosilo, defaŭlto ?? ŝlosilo);
  }

  function legi(ŝlosilo, defaŭlto) {
    try {
      return localStorage.getItem(ŝlosilo) ?? defaŭlto;
    } catch {
      return defaŭlto;
    }
  }

  function skribi(ŝlosilo, valoro) {
    try {
      localStorage.setItem(ŝlosilo, valoro);
    } catch {
      // La retejo restas funkcianta ankaŭ sen konservado.
    }
  }

  function konserviOriginalojn() {
    $$(
      "[data-t], [data-t-html], [data-t-aria], [data-t-meta], [data-t-placeholder], [data-t-title]"
    ).forEach((elemento) => {
      originaloj.set(elemento, {
        teksto: elemento.textContent,
        html: elemento.innerHTML,
        aria: elemento.getAttribute("aria-label"),
        meta: elemento.getAttribute("content"),
        placeholder: elemento.getAttribute("placeholder"),
        titolo: elemento.getAttribute("title"),
      });
    });
  }

  function aplikiLingvon(nova, konservu = true) {
    lingvo = nova === "en" ? "en" : "eo";
    radiko.lang = lingvo;

    originaloj.forEach((originalo, elemento) => {
      const datumoj = elemento.dataset;
      if (datumoj.t !== undefined) {
        elemento.textContent = traduki(datumoj.t, originalo.teksto);
      }
      if (datumoj.tHtml !== undefined) {
        elemento.innerHTML = traduki(datumoj.tHtml, originalo.html);
      }
      if (datumoj.tAria !== undefined) {
        elemento.setAttribute("aria-label", traduki(datumoj.tAria, originalo.aria));
      }
      if (datumoj.tMeta !== undefined) {
        elemento.setAttribute("content", traduki(datumoj.tMeta, originalo.meta));
      }
      if (datumoj.tPlaceholder !== undefined) {
        elemento.setAttribute("placeholder", traduki(datumoj.tPlaceholder, originalo.placeholder));
      }
      if (datumoj.tTitle !== undefined) {
        elemento.setAttribute("title", traduki(datumoj.tTitle, originalo.titolo));
      }
    });

    document.title = traduki("meta.titolo", originalaTitolo);
    $$("[data-lingvo]").forEach((butono) => {
      butono.setAttribute("aria-pressed", butono.dataset.lingvo === lingvo ? "true" : "false");
    });

    agordiTemon(radiko.dataset.temo || "malluma", false);
    agordiAkcenton(radiko.dataset.akcento || "verdo", false);
    ĝisdatigiFenestrojn();
    if (eniroAktiva) {
      prezentiEniron();
    }
    if (konservu) {
      skribi("mikerno-lingvo", lingvo);
    }
  }

  function agordiTemon(temo, konservu = true) {
    const valoro = temo === "hela" ? "hela" : "malluma";
    radiko.dataset.temo = valoro;

    const ŝaltilo = $("#temoSxaltilo");
    if (ŝaltilo) {
      ŝaltilo.checked = valoro === "malluma";
      ŝaltilo.setAttribute(
        "aria-label",
        t(
          valoro === "malluma" ? "temo.aria.malluma" : "temo.aria.hela",
          valoro === "malluma" ? "Malluma temo" : "Hela temo"
        )
      );
    }

    const etikedo = $("#temoEtikedo");
    if (etikedo) {
      etikedo.textContent = t(
        valoro === "malluma" ? "temo.malluma" : "temo.hela",
        valoro === "malluma" ? "Malluma" : "Hela"
      );
    }

    const koloro = valoro === "malluma" ? "#0A0D1C" : "#F6F3EC";
    $('meta[name="theme-color"]')?.setAttribute("content", koloro);

    if (konservu) {
      skribi("mikerno-temo", valoro);
    }
  }

  function agordiAkcenton(nomo, konservu = true) {
    const valoro = Object.prototype.hasOwnProperty.call(AKCENTOJ, nomo) ? nomo : "verdo";
    radiko.dataset.akcento = valoro;
    radiko.style.setProperty("--akcento", AKCENTOJ[valoro]);

    $$("[data-akcento]").forEach((butono) => {
      const akcento = butono.dataset.akcento;
      const aktiva = akcento === valoro;
      const nomoEnLingvo = AKCENTAJ_NOMOJ[akcento] || akcento;
      const etikedo = t(`akcento.${akcento}`, `Akcento: ${nomoEnLingvo}`);

      butono.setAttribute("aria-pressed", aktiva ? "true" : "false");
      butono.setAttribute("aria-label", etikedo);
      butono.setAttribute("title", etikedo);
    });

    if (konservu) {
      skribi("mikerno-akcento", valoro);
    }
  }

  function ĝisdatigiVersion() {
    $$("[data-versio-montri]").forEach((elemento) => {
      elemento.textContent = VERSIO;
    });
    $$("[data-versio-ligo]").forEach((ligilo) => {
      const href = ligilo.getAttribute("href") || "";
      ligilo.setAttribute(
        "href",
        href.replace(/\/v\d+\.\d+\.\d+(?:-arm\.\d+)?(?=\/|$)/, `/v${VERSIO}`)
      );
    });
  }

  function ŝaltiRevelon() {
    if (reduktitaMovo) {
      return;
    }
    $$(".revelo").forEach((elemento) => {
      elemento.addEventListener("pointermove", (evento) => {
        const kadro = elemento.getBoundingClientRect();
        if (!kadro.width || !kadro.height) {
          return;
        }
        const x = ((evento.clientX - kadro.left) / kadro.width) * 100;
        const y = ((evento.clientY - kadro.top) / kadro.height) * 100;
        elemento.style.setProperty("--mx", `${x.toFixed(1)}%`);
        elemento.style.setProperty("--my", `${y.toFixed(1)}%`);
      });
    });
  }

  function ŝaltiAperon() {
    const eroj = $$("[data-aperu]");
    if (reduktitaMovo || !("IntersectionObserver" in window)) {
      eroj.forEach((elemento) => elemento.classList.add("videbla"));
      return;
    }

    eroj.forEach((elemento) => elemento.classList.add("aperu"));
    const observanto = new IntersectionObserver(
      (eniroj, observilo) => {
        eniroj.forEach((eniro) => {
          if (eniro.isIntersecting) {
            eniro.target.classList.add("videbla");
            observilo.unobserve(eniro.target);
          }
        });
      },
      { rootMargin: "0px 0px -6% 0px", threshold: 0.08 }
    );
    eroj.forEach((elemento) => observanto.observe(elemento));
  }

  function ĝisdatigiFenestrojn() {
    $$("[data-fenestro]").forEach((fenestro) => {
      const maksimuma = $('[data-ago="pligrandigi"]', fenestro);
      if (!maksimuma) {
        return;
      }
      const pligrandigita = fenestro.classList.contains("pligrandigita");
      const etikedo = t(
        pligrandigita ? "fenestro.restarigi" : "fenestro.pligrandigi",
        pligrandigita ? "Restarigi" : "Pligrandigi"
      );
      maksimuma.setAttribute("aria-label", etikedo);
      maksimuma.setAttribute("title", etikedo);
      maksimuma.setAttribute("aria-pressed", pligrandigita ? "true" : "false");
    });
  }

  function ŝaltiFenestrojn() {
    $$("[data-fenestro]").forEach((fenestro) => {
      const identigilo = fenestro.dataset.fenestro;
      const sceno = fenestro.closest("[data-scene]");
      const restarigo = sceno
        ? $(`[data-restarigo="${identigilo}"]`, sceno)
        : null;

      const kaŝi = (klaso) => {
        fenestro.classList.remove("pligrandigita");
        fenestro.classList.add(klaso);
        fenestro.inert = true;
        fenestro.setAttribute("aria-hidden", "true");
        if (restarigo) {
          restarigo.hidden = false;
          $("[data-restarigi]", restarigo)?.focus();
        }
      };

      $$("[data-ago]", fenestro).forEach((butono) => {
        butono.addEventListener("click", () => {
          const ago = butono.dataset.ago;
          if (ago === "pligrandigi") {
            fenestro.classList.toggle("pligrandigita");
            ĝisdatigiFenestrojn();
          } else if (ago === "malgrandigi") {
            kaŝi("malgrandigita");
          } else if (ago === "fermi") {
            kaŝi("fermita");
          }
        });
      });

      const titolstrio = $("[data-treni]", fenestro);
      if (titolstrio) {
        titolstrio.addEventListener("dblclick", (evento) => {
          if (evento.target.closest("button")) {
            return;
          }
          fenestro.classList.toggle("pligrandigita");
          ĝisdatigiFenestrojn();
        });
      }

      if (restarigo) {
        $("[data-restarigi]", restarigo)?.addEventListener("click", () => {
          fenestro.classList.remove("malgrandigita", "fermita", "pligrandigita");
          fenestro.inert = false;
          fenestro.removeAttribute("aria-hidden");
          restarigo.hidden = true;
          ĝisdatigiFenestrojn();
          $('[data-ago="malgrandigi"]', fenestro)?.focus();
        });
      }
    });
  }

  function ŝaltiLogojn() {
    const butonoj = $$("[data-logbutono]");
    const legaĵoj = $$("[data-log]");
    const kopiiButono = $('[data-kopii^="eligo"]');

    const elekti = (nomo) => {
      butonoj.forEach((butono) => {
        const aktiva = butono.dataset.logbutono === nomo;
        butono.classList.toggle("aktiva", aktiva);
        butono.setAttribute("aria-selected", aktiva ? "true" : "false");
        butono.tabIndex = aktiva ? 0 : -1;
      });
      legaĵoj.forEach((legaĵo) => {
        legaĵo.hidden = legaĵo.dataset.log !== nomo;
      });
      if (kopiiButono) {
        kopiiButono.dataset.kopii = nomo === "arm" ? "eligoArm" : "eligoX86";
      }
    };

    butonoj.forEach((butono, indekso) => {
      butono.addEventListener("click", () => elekti(butono.dataset.logbutono));
      butono.addEventListener("keydown", (evento) => {
        if (evento.key !== "ArrowLeft" && evento.key !== "ArrowRight") {
          return;
        }
        evento.preventDefault();
        const direkto = evento.key === "ArrowRight" ? 1 : -1;
        const celo = (indekso + direkto + butonoj.length) % butonoj.length;
        butonoj[celo].focus();
        butonoj[celo].click();
      });
    });

    if (butonoj.length) {
      elekti(butonoj.find((butono) => butono.getAttribute("aria-selected") === "true")?.dataset.logbutono || "x86");
    }
  }

  async function kopiiTekston(teksto) {
    try {
      await navigator.clipboard.writeText(teksto);
      return true;
    } catch {
      const helpo = document.createElement("textarea");
      helpo.value = teksto;
      helpo.setAttribute("readonly", "");
      helpo.style.position = "fixed";
      helpo.style.opacity = "0";
      document.body.appendChild(helpo);
      helpo.select();
      let sukceso = false;
      try {
        sukceso = document.execCommand("copy");
      } catch {
        sukceso = false;
      }
      helpo.remove();
      return sukceso;
    }
  }

  function fulmomesaĝo(titolo, teksto, semantiko = "sukceso") {
    const ujo = $("#fulmomesaĝoj");
    if (!ujo) {
      return;
    }
    const mesaĝo = document.createElement("div");
    mesaĝo.className = `fulmomesaĝo ${semantiko}`;

    const titola = document.createElement("div");
    titola.className = "fulma-titolo";
    titola.textContent = titolo;

    const teksta = document.createElement("div");
    teksta.className = "fulma-teksto";
    teksta.textContent = teksto;

    mesaĝo.append(titola, teksta);
    ujo.appendChild(mesaĝo);
    while (ujo.children.length > 4) {
      ujo.firstElementChild.remove();
    }
    window.setTimeout(() => {
      mesaĝo.classList.add("foriri");
      window.setTimeout(() => mesaĝo.remove(), 240);
    }, 3400);
  }

  function ŝaltiKopiojn() {
    $$("[data-kopii]").forEach((butono) => {
      butono.addEventListener("click", async () => {
        const celo = document.getElementById(butono.dataset.kopii);
        if (!celo) {
          return;
        }
        const sukceso = await kopiiTekston(celo.textContent.trim());
        fulmomesaĝo(
          sukceso ? t("fulmo.kopiita", "Kopiita") : t("fulmo.eraro", "Kopio malsukcesis"),
          sukceso
            ? t("fulmo.kopiita.teksto", "La teksto estas en la tondujo.")
            : t("fulmo.eraro.teksto", "Vi povas elekti la tekston permane."),
          sukceso ? "sukceso" : "danĝero"
        );
      });
    });
  }

  function iriAl(selektilo) {
    const celo = $(selektilo);
    if (!celo) {
      return;
    }
    celo.scrollIntoView({
      behavior: reduktitaMovo ? "auto" : "smooth",
      block: "start",
    });
  }

  function malfermiLigon(url) {
    window.open(url, "_blank", "noopener,noreferrer");
  }

  function normaligi(teksto) {
    return teksto
      .toLocaleLowerCase()
      .replace(/cx/g, "c")
      .replace(/gx/g, "g")
      .replace(/hx/g, "h")
      .replace(/jx/g, "j")
      .replace(/sx/g, "s")
      .replace(/ux/g, "u")
      .normalize("NFD")
      .replace(/[\u0300-\u036f]/g, "");
  }

  function kalkuli(teksto) {
    const esprimo = teksto.replace(/,/g, ".").replace(/\s+/g, "");
    let pozicio = 0;

    function ĉefa() {
      const valoro = sumo();
      if (pozicio < esprimo.length) {
        throw new Error(t("kalkulo.nevalida", "Nevalida esprimo"));
      }
      return valoro;
    }

    function sumo() {
      let valoro = produto();
      while (esprimo[pozicio] === "+" || esprimo[pozicio] === "-") {
        const operacio = esprimo[pozicio++];
        const dekstra = produto();
        valoro = operacio === "+" ? valoro + dekstra : valoro - dekstra;
      }
      return valoro;
    }

    function produto() {
      let valoro = faktoro();
      while (
        esprimo[pozicio] === "*" ||
        esprimo[pozicio] === "/" ||
        esprimo[pozicio] === "%"
      ) {
        const operacio = esprimo[pozicio++];
        const dekstra = faktoro();
        if ((operacio === "/" || operacio === "%") && dekstra === 0) {
          throw new Error(
            operacio === "/"
              ? t("kalkulo.divido", "Divido per nulo")
              : t("kalkulo.modulo", "Modulo per nulo")
          );
        }
        valoro =
          operacio === "*" ? valoro * dekstra : operacio === "/" ? valoro / dekstra : valoro % dekstra;
      }
      return valoro;
    }

    function faktoro() {
      let signo = 1;
      while (esprimo[pozicio] === "+" || esprimo[pozicio] === "-") {
        if (esprimo[pozicio++] === "-") {
          signo = -signo;
        }
      }

      let valoro;
      if (esprimo[pozicio] === "(") {
        pozicio++;
        valoro = sumo();
        if (esprimo[pozicio] !== ")") {
          throw new Error(t("kalkulo.parentezo", "Mankas «)»"));
        }
        pozicio++;
      } else {
        const komenco = pozicio;
        while (pozicio < esprimo.length && /[0-9.]/.test(esprimo[pozicio])) {
          pozicio++;
        }
        if (komenco === pozicio) {
          throw new Error(t("kalkulo.mankas", "Mankas esprimo"));
        }
        valoro = Number(esprimo.slice(komenco, pozicio));
        if (!Number.isFinite(valoro)) {
          throw new Error(t("kalkulo.nombro", "Nevalida nombro"));
        }
      }

      if (esprimo[pozicio] === "^") {
        pozicio++;
        valoro = Math.pow(valoro, faktoro());
      }
      return signo * valoro;
    }

    const rezulto = ĉefa();
    if (!Number.isFinite(rezulto)) {
      throw new Error(t("kalkulo.nefinia", "La rezulto ne estas finia"));
    }
    return rezulto;
  }

  function formatu(nombro) {
    if (Number.isInteger(nombro)) {
      const teksto = String(nombro);
      return lingvo === "en"
        ? teksto
        : teksto.replace(/\B(?=(\d{3})+(?!\d))/g, "\u00A0");
    }
    const teksto = nombro.toFixed(6).replace(/0+$/, "").replace(/\.$/, "");
    return lingvo === "en" ? teksto : teksto.replace(".", ",");
  }

  let enirajEroj = [];
  let eniraElektita = 0;
  let eniraKalkulo = null;
  let eniraReĝimo = "sercho";
  let eniroAktiva = false;
  let antaŭaFokuso = null;

  function enirajDefinoj() {
    const nun = VERSIO;
    const difinoj = [
      {
        nomo: t("eniro.kapabloj", "Kapabloj"),
        priskribo: t("eniro.kapabloj.teksto", "Kerno, memoro, IK kaj konservado"),
        grupo: "sekcioj",
        ikono: "01",
        tipo: "sekcio",
        plenumu: () => iriAl("#kapabloj"),
      },
      {
        nomo: t("eniro.fluou", "FluoU"),
        priskribo: t("eniro.fluou.teksto", "Komponantoj, materialoj kaj fenestra kadro"),
        grupo: "sekcioj",
        ikono: "02",
        tipo: "sekcio",
        plenumu: () => iriAl("#fluou"),
      },
      {
        nomo: t("eniro.arkitekturo", "Arĥitekturo"),
        priskribo: t("eniro.arkitekturo.teksto", "Ses tavoloj de la aplikaĵo ĝis la ekrano"),
        grupo: "sekcioj",
        ikono: "03",
        tipo: "sekcio",
        plenumu: () => iriAl("#arkitekturo"),
      },
      {
        nomo: t("eniro.mejloŝtonoj", "Mejloŝtonoj"),
        priskribo: t("eniro.mejloŝtonoj.teksto", "M0 ĝis M18 kun pruvoj"),
        grupo: "sekcioj",
        ikono: "04",
        tipo: "sekcio",
        plenumu: () => iriAl("#mejloŝtonoj"),
      },
      {
        nomo: t("eniro.eligo", "Vera eligo"),
        priskribo: t("eniro.eligo.teksto", "Seria protokolo el QEMU"),
        grupo: "sekcioj",
        ikono: "05",
        tipo: "sekcio",
        plenumu: () => iriAl("#eligo"),
      },
      {
        nomo: t("eniro.elŝuti", "Elŝuti"),
        priskribo: t("eniro.elŝuti.teksto", "ISO-bildoj por x86_64 kaj arm64"),
        grupo: "sekcioj",
        ikono: "06",
        tipo: "sekcio",
        plenumu: () => iriAl("#elŝuti"),
      },
      {
        nomo: t("eniro.temo.malluma", "Temo: malluma"),
        priskribo: t("eniro.temo.teksto", "Ŝanĝi la aspekton de la retejo"),
        grupo: "agordoj",
        ikono: "T",
        tipo: "agordo",
        plenumu: () => {
          agordiTemon("malluma");
          fulmomesaĝo(t("fulmo.temo", "Temo ŝanĝita"), t("fulmo.temo.teksto", "La nova temo estas aktiva."));
        },
      },
      {
        nomo: t("eniro.temo.hela", "Temo: hela"),
        priskribo: t("eniro.temo.teksto", "Ŝanĝi la aspekton de la retejo"),
        grupo: "agordoj",
        ikono: "T",
        tipo: "agordo",
        plenumu: () => {
          agordiTemon("hela");
          fulmomesaĝo(t("fulmo.temo", "Temo ŝanĝita"), t("fulmo.temo.teksto", "La nova temo estas aktiva."));
        },
      },
      {
        nomo: t("eniro.lingvo.angla", "Lingvo: English"),
        priskribo: t("eniro.lingvo.teksto", "Montri la anglan mapadan tavolon"),
        grupo: "agordoj",
        ikono: "EN",
        tipo: "agordo",
        plenumu: () => aplikiLingvon("en"),
      },
      {
        nomo: t("eniro.lingvo.esperanto", "Lingvo: Esperanto"),
        priskribo: t("eniro.lingvo.teksto", "Montri la fontan Esperantan tekston"),
        grupo: "agordoj",
        ikono: "EO",
        tipo: "agordo",
        plenumu: () => aplikiLingvon("eo"),
      },
    ];

    Object.keys(AKCENTOJ).forEach((akcento) => {
      difinoj.push({
        nomo: t(`akcento.${akcento}`, `Akcento: ${AKCENTAJ_NOMOJ[akcento]}`),
        priskribo: t("eniro.akcento.teksto", "Ŝanĝi la akcentan koloron"),
        grupo: "agordoj",
        ikono: "A",
        tipo: "agordo",
        plenumu: () => {
          agordiAkcenton(akcento);
          fulmomesaĝo(
            t("fulmo.akcento", "Akcento ŝanĝita"),
            t("fulmo.akcento.teksto", "La nova akcento estas aktiva.")
          );
        },
      });
    });

    difinoj.push(
      {
        nomo: t("eniro.github", "Deponejo en GitHub"),
        priskribo: t("eniro.github.teksto", "Fonto, numeroj kaj diskutoj"),
        grupo: "ligoj",
        ikono: "↗",
        tipo: "ligo",
        plenumu: () => malfermiLigon("https://github.com/makoto-lernanto/mikerno"),
      },
      {
        nomo: `${t("eniro.eldono.prefikso", "Eldono")} ${nun}`,
        priskribo: t("eniro.eldono.teksto", "ISO-bildoj kaj rimarkoj"),
        grupo: "ligoj",
        ikono: "↗",
        tipo: "ligo",
        plenumu: () =>
          malfermiLigon(`https://github.com/makoto-lernanto/mikerno/releases/tag/v${nun}`),
      },
      {
        nomo: "README",
        priskribo: t("eniro.readme.teksto", "Rapida starto kaj konstruaj komandoj"),
        grupo: "ligoj",
        ikono: "↗",
        tipo: "ligo",
        plenumu: () =>
          malfermiLigon("https://github.com/makoto-lernanto/mikerno/blob/main/README.md"),
      },
      {
        nomo: t("eniro.dokumento.mejloŝtonoj", "Mejloŝtonoj (dokumento)"),
        priskribo: t("eniro.dokumento.mejloŝtonoj.teksto", "La plena historio kaj la DI-pordego"),
        grupo: "ligoj",
        ikono: "↗",
        tipo: "ligo",
        plenumu: () =>
          malfermiLigon(
            "https://github.com/makoto-lernanto/mikerno/blob/main/docs/MEJLOSTONOJ.md"
          ),
      },
      {
        nomo: "FLUOU.md",
        priskribo: t("eniro.fluou.dokumento.teksto", "La deviga vida lingvo"),
        grupo: "ligoj",
        ikono: "↗",
        tipo: "ligo",
        plenumu: () =>
          malfermiLigon("https://github.com/makoto-lernanto/mikerno/blob/main/docs/FLUOU.md"),
      },
      {
        nomo: t("eniro.montro", "Vida montro"),
        priskribo: t("eniro.montro.teksto", "La memstara FluoU-montraĵo"),
        grupo: "ligoj",
        ikono: "↗",
        tipo: "ligo",
        plenumu: () => malfermiLigon("FLUOU-montro.html"),
      }
    );

    return difinoj;
  }

  function kongruas(ero, serĉo) {
    const normala = normaligi(serĉo);
    if (!normala) {
      return true;
    }
    return normaligi(`${ero.nomo} ${ero.priskribo}`).includes(normala);
  }

  function eriElemento(ero, indekso) {
    const butono = document.createElement("button");
    butono.type = "button";
    butono.className = `enira-ero${indekso === eniraElektita ? " elektita" : ""}`;
    butono.setAttribute("role", "option");
    butono.setAttribute("aria-selected", indekso === eniraElektita ? "true" : "false");
    butono.dataset.indekso = String(indekso);

    const ikono = document.createElement("span");
    ikono.className = "era-ikono";
    ikono.setAttribute("aria-hidden", "true");
    ikono.textContent = ero.ikono || "·";

    const teksto = document.createElement("span");
    teksto.className = "era-teksto";
    const nomo = document.createElement("span");
    nomo.className = "era-nomo";
    nomo.textContent = ero.nomo;
    const priskribo = document.createElement("span");
    priskribo.className = "era-priskribo";
    priskribo.textContent = ero.priskribo;
    teksto.append(nomo, priskribo);
    butono.append(ikono, teksto);

    butono.addEventListener("click", () => {
      eniraElektita = indekso;
      plenumuElektitan();
    });

    return butono;
  }

  function montriErojn(eroj, senrezultaTeksto) {
    const rezultoj = $("#enirajRezultoj");
    if (!rezultoj) {
      return;
    }

    enirajEroj = eroj;
    if (eniraElektita >= eroj.length) {
      eniraElektita = 0;
    }

    const fragmento = document.createDocumentFragment();
    let lastaGrupo = "";

    eroj.forEach((ero, indekso) => {
      if (ero.grupo !== lastaGrupo) {
        const grupo = document.createElement("div");
        grupo.className = "enira-grupo";
        const [ŝlosilo, defaŭlto] = GRUPOJ[ero.grupo] || ["", ""];
        grupo.textContent = t(ŝlosilo, defaŭlto);
        fragmento.appendChild(grupo);
        lastaGrupo = ero.grupo;
      }
      fragmento.appendChild(eriElemento(ero, indekso));
    });

    if (!eroj.length) {
      const senrezulte = document.createElement("div");
      senrezulte.className = "enira-senrezulte";
      senrezulte.textContent = senrezultaTeksto;
      fragmento.appendChild(senrezulte);
    }

    rezultoj.replaceChildren(fragmento);
  }

  function prezentiKalkulon(esprimo) {
    eniraReĝimo = "kalkulo";
    enirajEroj = [];
    eniraKalkulo = null;
    const modo = $("#eniraModo");
    if (modo) {
      modo.textContent = t("eniro.modo.kalkulo", "Kalkulo");
    }
    const rezultoj = $("#enirajRezultoj");
    if (!rezultoj) {
      return;
    }

    try {
      const valoro = kalkuli(esprimo);
      eniraKalkulo = formatu(valoro);

      const skatolo = document.createElement("div");
      skatolo.className = "kalkula-rezulto";
      const etikedo = document.createElement("div");
      etikedo.className = "kalkula-etikedo";
      etikedo.textContent = t("kalkulo.rezulto", "Rezulto");
      const valora = document.createElement("div");
      valora.className = "kalkula-valoro";
      valora.textContent = eniraKalkulo;
      const esprima = document.createElement("div");
      esprima.className = "kalkula-esprimo";
      esprima.textContent = `= ${esprimo}`;
      const noto = document.createElement("div");
      noto.className = "kalkula-noto";
      noto.textContent = t(
        "kalkulo.noto",
        "Enter kopias la rezulton · sen eval · sen dosieroj · sen reto"
      );
      skatolo.append(etikedo, valora, esprima, noto);
      rezultoj.replaceChildren(skatolo);
    } catch (eraro) {
      const skatolo = document.createElement("div");
      skatolo.className = "enira-eraro";
      skatolo.textContent = eraro.message;
      rezultoj.replaceChildren(skatolo);
    }
  }

  function prezentiEniron() {
    const sercho = $("#eniraSercho");
    if (!sercho) {
      return;
    }
    const valoro = sercho.value;
    const modo = $("#eniraModo");

    if (valoro.startsWith("=")) {
      prezentiKalkulon(valoro.slice(1).trim());
      return;
    }

    if (valoro.startsWith(">")) {
      eniraReĝimo = "ligoj";
      eniraKalkulo = null;
      if (modo) {
        modo.textContent = t("eniro.modo.ligoj", "Ligoj");
      }
      const serĉo = valoro.slice(1).trim();
      const eroj = enirajDefinoj().filter((ero) => ero.tipo === "ligo" && kongruas(ero, serĉo));
      montriErojn(eroj, t("eniro.senligo", "Neniu kongrua ligo."));
      return;
    }

    eniraReĝimo = "sercho";
    eniraKalkulo = null;
    if (modo) {
      modo.textContent = t("eniro.modo.sercho", "Serĉo");
    }
    const serĉo = valoro.trim();
    const eroj = enirajDefinoj().filter((ero) => kongruas(ero, serĉo));
    montriErojn(eroj, t("eniro.senrezulte", "Neniu kongrua rezulto."));
  }

  function elekti(indekso) {
    if (!enirajEroj.length) {
      return;
    }
    eniraElektita = (indekso + enirajEroj.length) % enirajEroj.length;
    const rezultoj = $("#enirajRezultoj");
    if (!rezultoj) {
      return;
    }
    $$(".enira-ero", rezultoj).forEach((elemento, n) => {
      const aktiva = n === eniraElektita;
      elemento.classList.toggle("elektita", aktiva);
      elemento.setAttribute("aria-selected", aktiva ? "true" : "false");
    });
    $(`[data-indekso="${eniraElektita}"]`, rezultoj)?.scrollIntoView({ block: "nearest" });
  }

  async function kopiiKalkulon() {
    if (!eniraKalkulo) {
      return;
    }
    const sukceso = await kopiiTekston(eniraKalkulo);
    fulmomesaĝo(
      sukceso ? t("fulmo.kopiita", "Kopiita") : t("fulmo.eraro", "Kopio malsukcesis"),
      sukceso
        ? t("fulmo.kopiita.teksto", "La teksto estas en la tondujo.")
        : t("fulmo.eraro.teksto", "Vi povas elekti la tekston permane."),
      sukceso ? "sukceso" : "danĝero"
    );
  }

  function plenumuElektitan() {
    if (eniraReĝimo === "kalkulo") {
      kopiiKalkulon();
      return;
    }
    const ero = enirajEroj[eniraElektita];
    if (!ero) {
      return;
    }
    fermiEniron();
    ero.plenumu();
  }

  function kompletigiElektitan() {
    if (eniraReĝimo === "kalkulo") {
      return false;
    }
    const ero = enirajEroj[eniraElektita];
    const sercho = $("#eniraSercho");
    if (!ero || !sercho) {
      return false;
    }
    sercho.value = `${eniraReĝimo === "ligoj" ? "> " : ""}${ero.nomo}`;
    eniraElektita = 0;
    prezentiEniron();
    return true;
  }

  function malfermiEniron() {
    const tavolo = $("#eniraTavolo");
    const paletro = $("#eniroPaletro");
    const sercho = $("#eniraSercho");
    if (!tavolo || !paletro || !sercho) {
      return;
    }
    if (eniroAktiva) {
      sercho.focus();
      return;
    }

    antaŭaFokuso = document.activeElement;
    eniroAktiva = true;
    tavolo.hidden = false;
    paletro.hidden = false;
    document.body.style.overflow = "hidden";
    sercho.value = "";
    eniraElektita = 0;
    prezentiEniron();
    sercho.focus();
  }

  function fermiEniron() {
    const tavolo = $("#eniraTavolo");
    const paletro = $("#eniroPaletro");
    if (!eniroAktiva) {
      return;
    }
    eniroAktiva = false;
    if (tavolo) {
      tavolo.hidden = true;
    }
    if (paletro) {
      paletro.hidden = true;
    }
    document.body.style.overflow = "";
    if (antaŭaFokuso && typeof antaŭaFokuso.focus === "function") {
      antaŭaFokuso.focus();
    }
  }

  function ŝaltiEniron() {
    const sercho = $("#eniraSercho");
    const tavolo = $("#eniraTavolo");

    $("#eniroButono")?.addEventListener("click", malfermiEniron);
    $("#malfermiEniron")?.addEventListener("click", malfermiEniron);
    tavolo?.addEventListener("click", fermiEniron);

    if (sercho) {
      sercho.addEventListener("input", () => {
        eniraElektita = 0;
        prezentiEniron();
      });
      sercho.addEventListener("keydown", (evento) => {
        if (evento.key === "ArrowDown") {
          evento.preventDefault();
          elekti(eniraElektita + 1);
        } else if (evento.key === "ArrowUp") {
          evento.preventDefault();
          elekti(eniraElektita - 1);
        } else if (evento.key === "Enter") {
          evento.preventDefault();
          plenumuElektitan();
        } else if (evento.key === "Tab") {
          if (kompletigiElektitan()) {
            evento.preventDefault();
          }
        } else if (evento.key === "Escape") {
          evento.preventDefault();
          fermiEniron();
        }
      });
    }

    document.addEventListener("keydown", (evento) => {
      // Provizora klavo: Horizonto uzos la tradician Win-klavon (`Mito`),
      // kiun la foliumilo ne transdonas al la paĝo.
      if ((evento.ctrlKey || evento.metaKey) && evento.key.toLowerCase() === "e") {
        evento.preventDefault();
        malfermiEniron();
        return;
      }
      if (eniroAktiva && evento.key === "Escape") {
        evento.preventDefault();
        fermiEniron();
      }
    });
  }

  function ŝaltiAgordojn() {
    $("#temoSxaltilo")?.addEventListener("change", (evento) => {
      agordiTemon(evento.target.checked ? "malluma" : "hela");
    });
    $$("[data-akcento]").forEach((butono) => {
      butono.addEventListener("click", () => agordiAkcenton(butono.dataset.akcento));
    });
    $$("[data-lingvo]").forEach((butono) => {
      butono.addEventListener("click", () => aplikiLingvon(butono.dataset.lingvo));
    });
  }

  function iniciigi() {
    konserviOriginalojn();
    const konservitaLingvo = legi("mikerno-lingvo", "eo");
    lingvo = konservitaLingvo === "en" ? "en" : "eo";
    aplikiLingvon(lingvo, false);
    agordiTemon(legi("mikerno-temo", "malluma"), false);
    agordiAkcenton(legi("mikerno-akcento", "verdo"), false);

    ĝisdatigiVersion();
    ŝaltiAgordojn();
    ŝaltiFenestrojn();
    ŝaltiLogojn();
    ŝaltiKopiojn();
    ŝaltiEniron();
    ŝaltiRevelon();
    ŝaltiAperon();
  }

  iniciigi();
})();
