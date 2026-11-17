"use strict";

// Angla mapada tavolo por la retejo. La fontaj tekstoj restas Esperantaj en
// `docs/index.html`; ĉi tiu dosiero nur anstataŭigas montratajn tekstojn.
// Identigiloj, protokolaj nomoj kaj komandnomoj restas senŝanĝaj.

window.TRADUKOJ_ANGLA = {
  "meta.titolo": "MiKerno — a tiny Rust kernel for two architectures",
  "meta.priskribo":
    "MiKerno is a tiny, self-contained, verifiable uniprocessor kernel written in Rust, with identical behaviour on x86_64 and arm64.",

  salto: "Skip to content",
  "marko.sub": "tiny kernel · two arches",
  "marko.aria": "MiKerno — home",
  "nav.aria": "Main sections",
  "nav.kapabloj": "Capabilities",
  "nav.fluou": "FluoU",
  "nav.arkitekturo": "Architecture",
  "nav.mejlosxtonoj": "Milestones",
  "nav.eligo": "Output",
  "nav.elssxuti": "Download",
  "nav.eniro": "Eniro",
  "nav.fonto": "Source",
  "eniro.malfermi": "Open Eniro (Ctrl+E)",
  "eniro.provizora":
    "Stopgap shortcut: Ctrl+E. Later: Mito, the traditional Windows key.",
  "akcento.regiloj": "Accent",
  "temo.sxalti": "Switch between the dark and light themes",

  "hero.moto": "“The future is now.”",
  "hero.teksto":
    "A tiny, self-contained uniprocessor kernel written entirely in Rust — no external dependencies, no C in the kernel, and identical behaviour on x86_64 and arm64. Readable, hackable, provable.",
  "hero.principo1": "no dependencies",
  "hero.principo4": "41 proofs",
  "hero.butono.elssxuti": "Download",
  "hero.butono.fonto": "View the source",
  "hero.butono.gvidilo": "Read the guide",
  "hero.noto": "MIT · QEMU ISO · x86_64 and arm64",

  "pri.titolo": "About MiKerno",
  "pri.versio": "Version",
  "pri.arkitekturo": "Architecture",
  "pri.lingvo": "Language",
  "pri.planisto": "Scheduler",
  "pri.planisto.valoro": "preemptive · 100 Hz",
  "pri.konservado": "Storage",
  "pri.pruvoj": "Proofs",
  "pri.verda": "green",
  "pri.ci": "CI green: 43/43 jobs",
  "fenestro.regiloj": "Window controls",
  "fenestro.kasxita": "Window hidden",
  "fenestro.kasxita.teksto": "The window is waiting on the desktop.",
  "fenestro.remalfermi": "Reopen",

  "stat.aria": "Key numbers",
  "stat.arkoj": "architectures",
  "stat.pruvoj": "formal proofs",
  "stat.sistemvokoj": "syscalls",
  "stat.temporizilo": "timer",

  "kapabloj.titolo": "What is inside",
  "kapabloj.teksto":
    "From the first instruction to Ring 3: every layer is readable, hackable and backed by a real boot or a formal check.",
  "kapabloj.k1.titolo": "No dependencies",
  "kapabloj.k1.teksto":
    "#![no_std], #![no_main], no external Rust crates and no C in the kernel. Just the language, the hardware and the documentation.",
  "kapabloj.k2.titolo": "One codebase, two architectures",
  "kapabloj.k2.teksto":
    "The same IPC scenario and the same framebuffer hash on x86_64 and arm64. One backend, two platforms.",
  "kapabloj.k3.titolo": "Ring 3 and capabilities",
  "kapabloj.k3.teksto":
    "Eleven syscalls, ABI 2, Handle and Message. Every access to memory, files and the screen goes through an explicit capability.",
  "kapabloj.k4.titolo": "Formal verification",
  "kapabloj.k4.teksto":
    "Kani proves the frame allocator and the IPC core in CI: 41 proofs, one job per proof, with a 10-minute limit.",
  "kapabloj.k5.teksto":
    "The mandatory visual language of Horizonto: three materials, six accents, Windows-model windows and no accidental colours.",
  "kapabloj.k6.titolo": "Storage",
  "kapabloj.k6.teksto":
    "virtio-blk reads and writes 64 sectors over PCI on x86_64 and MMIO on arm64; the memory probe restores the contents.",

  "fluou.titolo": "A visual language, not an accidental style",
  "fluou.teksto":
    "This site itself uses the FluoU tokens: the same colours, materials, measurements and motion as the specified interface. The normative text remains FLUOU.md.",
  "fluou.fenestro.titolo": "FluoU — components",
  "fluou.dokumentoj": "Documents",
  "fluou.agordoj": "Settings",
  "fluou.akrila": "Acrylic blur",
  "fluou.butono.akcenta": "Accent",
  "fluou.butono.norma": "Standard",
  "fluou.butono.dangxera": "Danger",
  "fluou.butono.eniro": "Open Eniro",
  "fluou.demo.teksto":
    "Every window has a 32 DIP title bar, a centred title and the three controls on the right. The window radius is 8 DIP; the controls themselves are square.",
  "fluou.eniro.provizora":
    "The Ctrl+E shortcut is a stopgap. The real shortcut will be Mito, the traditional Windows key.",
  "fluou.informo.semantiko": "Success",
  "fluou.informo.teksto": "A view according to FluoU 0.1",
  "fluou.progreso": "Sync",
  "fluou.mika": "Opaque base with a 7% accent tint from the wallpaper.",
  "fluou.akrila.teksto":
    "24 DIP background blur, saturation 1.5 and grain for transitional surfaces.",
  "fluou.revelo": "Pointer-following edge glow at 55% for interactive cards and edges.",
  "fluou.montro": "Live view",

  "ark.titolo": "Six layers, one flow",
  "ark.teksto":
    "Capabilities travel down; messages travel up. No application touches the framebuffer directly.",
  "ark.t1.titolo": "Application",
  "ark.t1.teksto":
    "horizonto · Ring 3 / EL0 — a user process with its own address space and minimal capabilities.",
  "ark.t2.teksto":
    "Components, materials and motion; windows exist only with a FENESTRO handle.",
  "ark.t3.teksto":
    "The shared contract between the kernel and user space: ABI 2, Handle and Message.",
  "ark.t4.teksto":
    "Scheduler, IPC, memory, interrupts and virtio-blk on x86_64 and arm64.",
  "ark.t5.titolo": "Compositor",
  "ark.t5.teksto":
    "An isolated service that accepts only windows with a valid FENESTRO handle.",
  "ark.t6.titolo": "Screen",
  "ark.t6.teksto":
    "1280×800, 32 bpp; the same framebuffer hash on both architectures.",
  "ark.noto":
    "FluoU 0.1 describes the compositor as an isolated service; the current prototype already has the kernel and Ring 3 layers.",

  "mej.titolo": "From the first instruction to storage",
  "mej.teksto":
    "Every step has evidence: a boot, a hash, a sample or a green CI result.",
  "stato.plenumita": "Completed",
  "stato.parte": "Partial",
  "stato.nun": "Now",
  "mej.m0.titolo": "Workspace and pipeline",
  "mej.m0.teksto": "Three crates, one build.",
  "mej.m4.titolo": "GDT, IDT, TSS",
  "mej.m4.teksto": "256 vectors and an int 0x80 gate.",
  "mej.m7.titolo": "Preemptive scheduler",
  "mej.m7.teksto": "Round-robin scheduler with eight threads.",
  "mej.m8.titolo": "Message IPC",
  "mej.m8.teksto": "Endpoints, capabilities, blocking and wakeups.",
  "mej.m11.titolo": "Ring 3",
  "mej.m11.teksto": "int 0x80 from user space.",
  "mej.m12.titolo": "Per-process address spaces",
  "mej.m12.teksto": "A spy receives #PF; the victim survives.",
  "mej.m13.titolo": "Memory hardening",
  "mej.m13.teksto": "SMEP, SMAP and KASLR.",
  "mej.m15.titolo": "arm64",
  "mej.m15.teksto": "EL0, ABI 2, ramfb and PAN.",
  "mej.m17.titolo": "Storage",
  "mej.m17.teksto": "virtio-blk over PCI and MMIO.",
  "mej.m18.titolo": "CI and verification",
  "mej.m18.teksto": "43/43 jobs green; tests are still coming.",
  "mej.ligilo": "All milestones in the documentation",

  "eligo.titolo": "Not a mock-up — the console",
  "eligo.teksto":
    "The serial output of the current ISO under QEMU, including the virtio-blk memory probe. Both architectures produce the same framebuffer hash.",
  "eligo.fenestro.titolo": "Ligo — serial output",
  "eligo.stato": "Connected",
  "eligo.kopii": "Copy",
  "eligo.langetoj": "Output architecture",
  "eligo.x86.aria": "Serial output from x86_64",
  "eligo.arm.aria": "Serial output from arm64",
  "eligo.noto":
    "Selected lines from the real output; the full protocol is in MEJLOSTONOJ.md.",

  "elssxuti.titolo": "Download and boot",
  "elssxuti.teksto":
    "Two ISO images for QEMU. Both were booted headlessly before the release; the console output above comes from them.",
  "elssxuti.x86.teksto": "PCI · 1280×800 framebuffer · SMEP/SMAP",
  "elssxuti.arm.teksto": "MMIO · ramfb · PAN/PXN/UXN",
  "elssxuti.grando": "Size",
  "elssxuti.x86.grando": "4.44 MiB",
  "elssxuti.arm.grando": "4.95 MiB",
  "elssxuti.elssxuti": "Download",
  "elssxuti.kopii": "Copy SHA-256",
  "elssxuti.noto":
    "For the full memory probe, including virtio-blk, use the repository: <code>just run-iso</code> or <code>just run-iso-arm</code>. The ISO itself boots without the test disk.",
  "elssxuti.gvidilo": "Quick start in README",

  "piedo.teksto":
    "MIT · This site uses the FluoU visual language; the normative text remains FLUOU.md.",
  "piedo.mejlosxtonoj": "Milestones",
  "piedo.vortaro": "Glossary",
  "piedo.montro": "Live view",
  "piedo.autoro": "Idea: Makoto Lernanto · code and documentation: Qoder.",

  "temo.malluma": "Dark",
  "temo.hela": "Light",
  "temo.aria.malluma": "Dark theme",
  "temo.aria.hela": "Light theme",

  "akcento.oro": "Accent: gold",
  "akcento.koralo": "Accent: coral",
  "akcento.cxielo": "Accent: sky",
  "akcento.verdo": "Accent: green",
  "akcento.viola": "Accent: violet",
  "akcento.bluo": "Accent: blue",

  "fenestro.malgrandigi": "Minimize",
  "fenestro.pligrandigi": "Maximize",
  "fenestro.restarigi": "Restore",
  "fenestro.fermi": "Close",

  "fulmo.aria": "Notifications",
  "fulmo.kopiita": "Copied",
  "fulmo.kopiita.teksto": "The text is on the clipboard.",
  "fulmo.eraro": "Copy failed",
  "fulmo.eraro.teksto": "You can select the text manually.",
  "fulmo.temo": "Theme changed",
  "fulmo.temo.teksto": "The new theme is active.",
  "fulmo.akcento": "Accent changed",
  "fulmo.akcento.teksto": "The new accent is active.",

  "eniro.titolo": "Eniro",
  "eniro.modo.sercho": "Search",
  "eniro.modo.kalkulo": "Calculation",
  "eniro.modo.ligoj": "Links",
  "eniro.sercho": "Search sections, documents or links…",
  "eniro.navigi": "navigate",
  "eniro.ruli": "run",
  "eniro.kompletigi": "complete",
  "eniro.fermi": "close",
  "eniro.rezultoj": "Eniro results",
  "eniro.senrezulte": "No matching result.",
  "eniro.senligo": "No matching link.",
  "eniro.grupo.sekcioj": "Sections",
  "eniro.grupo.agordoj": "Settings",
  "eniro.grupo.ligoj": "Links",

  "eniro.kapabloj": "Capabilities",
  "eniro.kapabloj.teksto": "Kernel, memory, IPC and storage",
  "eniro.fluou": "FluoU",
  "eniro.fluou.teksto": "Components, materials and window chrome",
  "eniro.fluou.dokumento.teksto": "The mandatory visual language",
  "eniro.arkitekturo": "Architecture",
  "eniro.arkitekturo.teksto": "Six layers from the application to the screen",
  "eniro.mejlosxtonoj": "Milestones",
  "eniro.mejlosxtonoj.teksto": "M0 to M18 with evidence",
  "eniro.eligo": "Real output",
  "eniro.eligo.teksto": "Serial log from QEMU",
  "eniro.elssxuti": "Download",
  "eniro.elssxuti.teksto": "ISO images for x86_64 and arm64",

  "eniro.temo.malluma": "Theme: dark",
  "eniro.temo.hela": "Theme: light",
  "eniro.temo.teksto": "Change the look of the site",
  "eniro.lingvo.angla": "Language: English",
  "eniro.lingvo.esperanto": "Language: Esperanto",
  "eniro.lingvo.teksto": "Choose the language layer",
  "eniro.akcento.teksto": "Change the accent colour",

  "eniro.github": "GitHub repository",
  "eniro.github.teksto": "Source, issues and discussions",
  "eniro.eldono.prefikso": "Release",
  "eniro.eldono.teksto": "ISO images and release notes",
  "eniro.readme.teksto": "Quick start and build commands",
  "eniro.dokumento.mejlosxtonoj": "Milestones (document)",
  "eniro.dokumento.mejlosxtonoj.teksto": "The full history and the CI gate",
  "eniro.montro": "Live view",
  "eniro.montro.teksto": "The standalone FluoU showcase",

  "kalkulo.rezulto": "Result",
  "kalkulo.noto": "Enter copies the result · no eval · no files · no network",
  "kalkulo.mankas": "Missing expression",
  "kalkulo.parentezo": "Missing “)”",
  "kalkulo.nevalida": "Invalid expression",
  "kalkulo.nombro": "Invalid number",
  "kalkulo.divido": "Division by zero",
  "kalkulo.modulo": "Modulo by zero",
  "kalkulo.nefinia": "The result is not finite",
};
