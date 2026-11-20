//! Minimuma FDT-analizilo por la `arm64`-skeleto.
//!
//! La analizilo legas nur tion, kion A1 bezonas: la `/memory`-regionojn, la
//! adreson de la PL011 kaj la `initrd`-limojn el `/chosen`. Se `/memory`
//! mankas (Limine sub UEFI), la memoro venas el la UEFI-memormapo en
//! `/chosen`. Ĝi estas singarda kontraŭ difektita aŭ tro granda DTB: ĉiu
//! legado estas limigita al la deklarita grandeco.

use core::slice;

const MAGIO: u32 = 0xd00d_feed;
const MAKSIMUMA_GRANDO: usize = 16 * 1024 * 1024;
pub const MAKSAJ_MEMORAJ: usize = 16;
const MAKSAJ_NODOJ: usize = 16;
const UEFI_KONVENA_MEMORO: u32 = 7;
const UEFI_DESKRIPTO_BAJTOJ: usize = 40;
const UEFI_MAKSIMUMA_MAPO: usize = 64 * 1024;

#[derive(Clone, Copy)]
pub struct MemoraRegiono {
    pub komenco: u64,
    pub longo: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Speco {
    Alia,
    Memoro,
    Elektita,
    Seria,
    FwCfg,
}

pub struct Fdt {
    datumoj: &'static [u8],
    off_struct: usize,
    off_strings: usize,
    size_struct: usize,
    memoro: [MemoraRegiono; MAKSAJ_MEMORAJ],
    nombro: usize,
    seria: Option<u64>,
    fw_cfg: Option<u64>,
    initrd: Option<(u64, u64)>,
}

impl Fdt {
    pub fn nova(dtb: *const u8) -> Option<Self> {
        if dtb.is_null() {
            return None;
        }
        // SAFETY: la praŝarĝilo transdonas validan DTB-on; ni legas nur la
        // unuajn ok bajtojn antaŭ ol kontroli `totalsize`.
        let magio = u32::from_be(unsafe { (dtb as *const u32).read_unaligned() });
        if magio != MAGIO {
            return None;
        }
        let totalo = u32::from_be(unsafe { (dtb.add(4) as *const u32).read_unaligned() }) as usize;
        if !(40..=MAKSIMUMA_GRANDO).contains(&totalo) {
            return None;
        }
        // SAFETY: `totalo` estis kontrolita; la DTB restas valida dum la tuta
        // praŝarĝo, do la `'static`-vivo estas pravigita.
        let datumoj = unsafe { slice::from_raw_parts(dtb, totalo) };

        let mut fdt = Self {
            datumoj,
            off_struct: 0,
            off_strings: 0,
            size_struct: 0,
            memoro: [MemoraRegiono {
                komenco: 0,
                longo: 0,
            }; MAKSAJ_MEMORAJ],
            nombro: 0,
            seria: None,
            fw_cfg: None,
            initrd: None,
        };
        fdt.off_struct = fdt.be32(8)? as usize;
        fdt.off_strings = fdt.be32(12)? as usize;
        let deklarita = fdt.be32(36).unwrap_or(0) as usize;
        fdt.size_struct = if deklarita == 0 {
            datumoj.len().checked_sub(fdt.off_struct)?
        } else {
            deklarita
        };
        if fdt.off_struct.checked_add(fdt.size_struct)? > datumoj.len()
            || fdt.off_strings >= datumoj.len()
        {
            return None;
        }
        fdt.analizu()?;
        Some(fdt)
    }

    pub fn memoro(&self) -> &[MemoraRegiono] {
        &self.memoro[..self.nombro]
    }

    pub fn seria_adreso(&self) -> Option<u64> {
        self.seria
    }

    /// La bazo de la `fw_cfg`-MMIO (`qemu,fw-cfg-mmio`).
    pub fn fw_cfg_adreso(&self) -> Option<u64> {
        self.fw_cfg
    }

    pub fn initrd(&self) -> Option<(u64, u64)> {
        self.initrd
    }

    fn analizu(&mut self) -> Option<()> {
        let mut off = self.off_struct;
        let fino = self.off_struct + self.size_struct;
        let mut profundo = 0usize;
        let mut celoj = [(2usize, 1usize); MAKSAJ_NODOJ];
        let mut speco = [Speco::Alia; MAKSAJ_NODOJ];
        let mut reg_bazo = [None::<u64>; MAKSAJ_NODOJ];
        let mut initrd_komenco = None;
        let mut initrd_fino = None;
        let mut mmap_komenco = None;
        let mut mmap_longo = None;
        let mut mmap_ero = None;

        while off + 4 <= fino {
            let token = self.be32(off)?;
            off += 4;
            match token {
                1 => {
                    let (nomo, sekva) = self.nodo_nomo(off)?;
                    off = sekva;
                    profundo += 1;
                    if profundo < MAKSAJ_NODOJ {
                        celoj[profundo] = celoj[profundo - 1];
                        // La antaŭa frato lasis sian `reg` ĉi tie; `compatible`
                        // de ĉi tiu nodo ne rajtas legi ĝin.
                        reg_bazo[profundo] = None;
                        speco[profundo] = if estas(nomo, b"memory") || nomo.starts_with(b"memory@")
                        {
                            Speco::Memoro
                        } else if estas(nomo, b"chosen") {
                            Speco::Elektita
                        } else if estas(nomo, b"serial") || nomo.starts_with(b"serial@") {
                            Speco::Seria
                        } else if nomo.starts_with(b"fw-cfg") {
                            Speco::FwCfg
                        } else {
                            Speco::Alia
                        };
                    }
                }
                2 => {
                    if profundo == 0 {
                        return None;
                    }
                    profundo -= 1;
                }
                3 => {
                    let longo = self.be32(off)? as usize;
                    let poseda = self.be32(off + 4)? as usize;
                    off += 8;
                    let valoro_off = off;
                    off = alinii4(off.checked_add(longo)?);
                    if off > fino || valoro_off.checked_add(longo)? > fino {
                        return None;
                    }
                    let nomo = self.poseda_nomo(poseda)?;
                    if profundo == 0 || profundo >= MAKSAJ_NODOJ {
                        continue;
                    }

                    if estas(nomo, b"#address-cells") && longo >= 4 {
                        if let Some(valoro) = self.be32(valoro_off) {
                            celoj[profundo].0 = valoro as usize;
                        }
                    } else if estas(nomo, b"#size-cells") && longo >= 4 {
                        if let Some(valoro) = self.be32(valoro_off) {
                            celoj[profundo].1 = valoro as usize;
                        }
                    } else if estas(nomo, b"device_type") && longo > 0 {
                        if self.datumoj[valoro_off..valoro_off + longo].starts_with(b"memory") {
                            speco[profundo] = Speco::Memoro;
                        }
                    } else if estas(nomo, b"compatible") && longo > 0 {
                        let valoro = &self.datumoj[valoro_off..valoro_off + longo];
                        if enhavas(valoro, b"arm,pl011") {
                            speco[profundo] = Speco::Seria;
                            if let Some(bazo) = reg_bazo[profundo] {
                                self.seria = Some(bazo);
                            }
                        } else if enhavas(valoro, b"qemu,fw-cfg-mmio") {
                            speco[profundo] = Speco::FwCfg;
                            if let Some(bazo) = reg_bazo[profundo] {
                                self.fw_cfg = Some(bazo);
                            }
                        }
                    } else if estas(nomo, b"reg") {
                        // `reg` povas antaŭi `compatible`; konservu la bazan
                        // adreson por ĉiu nodo, por ke la speco-decidiĝo ĝin
                        // trovu sendepende de la propraĵa ordo.
                        let adresaj = celoj[profundo].0;
                        if let Some(bazo) = self.celoj(valoro_off, adresaj) {
                            reg_bazo[profundo] = Some(bazo);
                        }
                        if speco[profundo] == Speco::Memoro {
                            let (adresaj, grandaj) = celoj[profundo];
                            if let (Some(komenco), Some(longo)) = (
                                self.celoj(valoro_off, adresaj),
                                self.celoj(valoro_off.checked_add(adresaj * 4)?, grandaj),
                            ) && longo > 0
                                && self.nombro < MAKSAJ_MEMORAJ
                            {
                                self.memoro[self.nombro] = MemoraRegiono { komenco, longo };
                                self.nombro += 1;
                            }
                        } else if speco[profundo] == Speco::Seria {
                            self.seria = reg_bazo[profundo];
                        } else if speco[profundo] == Speco::FwCfg {
                            self.fw_cfg = reg_bazo[profundo];
                        }
                    } else if speco[profundo] == Speco::Elektita {
                        if estas(nomo, b"linux,initrd-start") {
                            initrd_komenco = self.valoron(valoro_off, longo);
                        } else if estas(nomo, b"linux,initrd-end") {
                            initrd_fino = self.valoron(valoro_off, longo);
                        } else if estas(nomo, b"linux,uefi-mmap-start") {
                            mmap_komenco = self.valoron(valoro_off, longo);
                        } else if estas(nomo, b"linux,uefi-mmap-size") {
                            mmap_longo = self.valoron(valoro_off, longo).map(|v| v as usize);
                        } else if estas(nomo, b"linux,uefi-mmap-desc-size") {
                            mmap_ero = self.valoron(valoro_off, longo).map(|v| v as usize);
                        }
                    }
                }
                4 => {}
                9 => break,
                _ => return None,
            }
        }

        if let (Some(komenco), Some(fino)) = (initrd_komenco, initrd_fino)
            && fino > komenco
        {
            self.initrd = Some((komenco, fino));
        }
        // Limine transdonas sub UEFI nur minimuman DTB-on kun `/chosen`; la
        // uzebla memoro venas el la UEFI-memormapo, ne el `/memory`.
        if self.nombro == 0
            && let (Some(komenco), Some(longo), Some(ero)) = (mmap_komenco, mmap_longo, mmap_ero)
        {
            self.legi_uefi_mmap(komenco, longo, ero);
        }
        Some(())
    }

    fn legi_uefi_mmap(&mut self, komenco: u64, longo: usize, ero: usize) {
        if komenco == 0
            || komenco >= 0x1_0000_0000
            || komenco.checked_add(longo as u64).is_none()
            || !(UEFI_DESKRIPTO_BAJTOJ..=4096).contains(&ero)
            || !(UEFI_DESKRIPTO_BAJTOJ..=UEFI_MAKSIMUMA_MAPO).contains(&longo)
        {
            return;
        }
        let nombro = longo / ero;
        for indekso in 0..nombro {
            if self.nombro >= MAKSAJ_MEMORAJ {
                break;
            }
            let adreso = komenco + (indekso * ero) as u64;
            // SAFETY: la adreso kaj la longo venas el la UEFI-memormapo de la
            // praŝarĝilo; la mapo kuŝas en identece atingebla RAM.
            let tipo = unsafe { (adreso as *const u32).read_unaligned() };
            if tipo != UEFI_KONVENA_MEMORO {
                continue;
            }
            // SAFETY: sama memorregiono kiel supre; la kampoj sekvas la UEFI-
            // priskribon (`PhysicalStart` ĉe 8, `NumberOfPages` ĉe 24).
            let regiona_komenco = unsafe { ((adreso + 8) as *const u64).read_unaligned() };
            let pagoj = unsafe { ((adreso + 24) as *const u64).read_unaligned() };
            let Some(regiona_longo) = pagoj.checked_mul(4096) else {
                continue;
            };
            if regiona_longo == 0 {
                continue;
            }
            if self.nombro > 0 {
                let antaua = &mut self.memoro[self.nombro - 1];
                if antaua.komenco.checked_add(antaua.longo) == Some(regiona_komenco)
                    && let Some(kunigita) = antaua.longo.checked_add(regiona_longo)
                {
                    antaua.longo = kunigita;
                    continue;
                }
            }
            self.memoro[self.nombro] = MemoraRegiono {
                komenco: regiona_komenco,
                longo: regiona_longo,
            };
            self.nombro += 1;
        }
    }

    fn nodo_nomo(&self, off: usize) -> Option<(&'static [u8], usize)> {
        let datumoj = self.datumoj;
        let mut fino = off;
        while fino < datumoj.len() && datumoj[fino] != 0 {
            fino += 1;
        }
        if fino >= datumoj.len() {
            return None;
        }
        Some((&datumoj[off..fino], alinii4(fino + 1)))
    }

    fn poseda_nomo(&self, nameoff: usize) -> Option<&'static [u8]> {
        let datumoj = self.datumoj;
        let komenco = self.off_strings.checked_add(nameoff)?;
        if komenco >= datumoj.len() {
            return None;
        }
        let mut fino = komenco;
        while fino < datumoj.len() && datumoj[fino] != 0 {
            fino += 1;
        }
        if fino >= datumoj.len() {
            return None;
        }
        Some(&datumoj[komenco..fino])
    }

    fn celoj(&self, off: usize, nombro: usize) -> Option<u64> {
        if nombro == 0 || nombro > 2 {
            return None;
        }
        let mut valoro = 0u64;
        for indekso in 0..nombro {
            valoro = (valoro << 32) | self.be32(off.checked_add(indekso * 4)?)? as u64;
        }
        Some(valoro)
    }

    fn valoron(&self, off: usize, longo: usize) -> Option<u64> {
        match longo {
            4 => self.be32(off).map(u64::from),
            8 => self.be64(off),
            _ => None,
        }
    }

    fn be32(&self, off: usize) -> Option<u32> {
        let datumoj = self.datumoj;
        let fino = off.checked_add(4)?;
        if fino > datumoj.len() {
            return None;
        }
        Some(u32::from_be_bytes([
            datumoj[off],
            datumoj[off + 1],
            datumoj[off + 2],
            datumoj[off + 3],
        ]))
    }

    fn be64(&self, off: usize) -> Option<u64> {
        let alta = self.be32(off)? as u64;
        let malalta = self.be32(off.checked_add(4)?)? as u64;
        Some((alta << 32) | malalta)
    }
}

fn estas(nomo: &[u8], valoro: &[u8]) -> bool {
    nomo == valoro
}

fn enhavas(valoro: &[u8], serĉata: &[u8]) -> bool {
    !serĉata.is_empty()
        && valoro
            .windows(serĉata.len())
            .any(|fenestro| fenestro == serĉata)
}

const fn alinii4(valoro: usize) -> usize {
    (valoro + 3) & !3
}
