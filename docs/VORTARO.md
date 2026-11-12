# VORTARO — 世界语 / 英语术语对照

Ĉi tiu vortaro fiksas la terminologion de MiKerno. Novaj identigiloj, komentoj
kaj protokolaj nomoj sekvu ĝin.

| Esperanto | English | 中文 | Rimarko |
| --- | --- | --- | --- |
| `procesoro` | processor / CPU | 处理器 | Neniam uzu `cpu` en identigiloj. |
| `memoro` | memory | 内存 | Neniam uzu `memory` aŭ `ram`. |
| `fadeno` | thread | 线程 | Neniam uzu `thread`. |
| `planisto` | scheduler | 调度器 | Neniam uzu `scheduler`. |
| `pagxo`, `paĝo` | page | 页 | Identigiloj uzas `pagx` aŭ `paĝ`. |
| `pagxado` | paging | 分页 | Modula nomo. |
| `kadro` | frame | 物理页帧 | Neniam uzu `frame`. |
| `kadra alokilo` | frame allocator | 页帧分配器 | `KadraAlokilo`. |
| `virtuala adreso` | virtual address | 虚拟地址 | |
| `fizika adreso` | physical address | 物理地址 | |
| `adresa spaco` | address space | 地址空间 | `AdresaSpaco`. |
| `paĝa tabelo` | page table | 页表 | `PaĝaTablo`. |
| `paĝa eraro` | page fault (#PF) | 缺页异常 | |
| `ĝenerala protekta eraro` | general protection fault (#GP) | 通用保护异常 | |
| `interrompo` | interrupt | 中断 | |
| `interrompa peto` | interrupt request (IRQ) | 中断请求 | Neniam uzu `IRQ` en identigiloj. |
| `interrompa kadro` | interrupt frame | 中断栈帧 | `InterrompaKadro`. |
| `interrompa traktilo` | interrupt handler | 中断处理程序 | `InterrompaTraktilo`. |
| `interrompa dispetĉo` | interrupt dispatch | 中断分发 | `interrompa_dispetcho`. |
| `temporizilo` | timer (PIT) | 定时器 | Anstataŭ `PIT` en interna kodo. |
| `temporizila tiko` | timer tick | 定时器滴答 | |
| `kontrolilo` | controller (APIC) | 控制器 | `apic` restas kiel modula nomo. |
| `tutmonda priskriba tabelo` | global descriptor table (GDT) | 全局描述符表 | `TutmondaPriskribaTabelo`. |
| `interrompa priskriba tabelo` | interrupt descriptor table (IDT) | 中断描述符表 | `InterrompaPriskribaTabelo`. |
| `taska stata segmento` | task state segment (TSS) | 任务状态段 | `TaskaStataSegmento`. |
| `priskribo` | descriptor | 描述符 | |
| `selektilo` | selector | 选择子 | |
| `pordego` | gate | 门 | |
| `interrompa pordego` | interrupt gate | 中断门 | Malŝaltas `RFLAGS.IF`. |
| `kapta pordego` | trap gate | 陷阱门 | Ne malŝaltas `RFLAGS.IF`. |
| `privilegia nivelo` | privilege level (DPL/RPL/CPL) | 特权级 | |
| `kerna reĝimo` | kernel mode (Ring 0) | 内核态 | |
| `uzanta reĝimo` | user mode (Ring 3) | 用户态 | |
| `uzantspaco` | user space | 用户空间 | |
| `sistemvoko` | system call | 系统调用 | `SYS_*` restas kiel ABI-prefikso. |
| `sistemvoka vektoro` | syscall vector | 系统调用向量 | `0x80`. |
| `sistemvoka numero` | system call number | 系统调用号 | `rax` ĉe la voko. |
| `uzanta kodo` | user code | 用户代码段 | DPL3. |
| `uzantaj datumoj` | user data | 用户数据段 | DPL3. |
| `kerna stako` | kernel stack | 内核栈 | `RSP0`. |
| `uzanta stako` | user stack | 用户栈 | |
| `kunteksto` | context | 上下文 | `Kunteksto`. |
| `kunteksta ŝanĝo` | context switch | 上下文切换 | |
| `enirpunkto` | entry point | 入口点 | |
| `bildo` | image | 镜像 | Ekz. `horizonto`. |
| `modulo` | module | 模块 | Multiboot2 module. |
| `praŝarĝilo` | bootloader | 引导加载器 | |
| `praŝarĝo` | boot | 引导 | |
| `pravalorizi` | initialize | 初始化 | |
| `halti` | halt | 停机 | |
| `atendi interrompon` | wait for interrupt | 等待中断 | `hlt`. |
| `ebligi` / `malebligi` | enable / disable | 启用 / 禁用 | |
| `mesaĝo` | message | 消息 | `Mesago`. |
| `mesaĝujo` | mailbox / message queue | 消息队列 | |
| `finpunkto` | endpoint | 端点 | `FinaPunkto`. |
| `kapablo` | capability | 能力 | `Kapablo`. |
| `permeso` | permission | 权限 | `Permesoj`. |
| `sendi` / `ricevi` | send / receive | 发送 / 接收 | |
| `bloki` / `veki` | block / wake | 阻塞 / 唤醒 | |
| `cedi` | yield | 让出 | `SYS_CEDI`. |
| `morti` / `fini` | exit | 退出 | |
| `unuopa` | uniprocessor | 单核 | |
| `antaŭokupa` | preemptive | 抢占式 | |
| `cirkla vico` | circular queue | 环形队列 | |
| `kapo` / `vosto` | head / tail | 队头 / 队尾 | |
| `bitmapo` | bitmap | 位图 | |
| `folio` | leaf page | 叶子页 | |
| `ero` | entry | 表项 | |
| `limo` | limit | 界限 | |
| `bazo` | base | 基址 | |
| `nuligi` | zero / clear | 清零 | |
| `kopii` | copy | 复制 | |
| `validigi` | validate | 校验 | |
| `eraro` | error | 错误 | |
| `sukceso` | success | 成功 | |
| `rezulto` | result | 结果 | |
| `argumento` | argument | 参数 | |
| `registro` | register | 寄存器 | |
| `stako` | stack | 栈 | |
| `pinto` | top | 栈顶 | |
| `kreski malsupren` | grow downward | 向下增长 | |
| `frambufo` / `ekrano` | framebuffer / screen | 帧缓冲 / 屏幕 | Ekzistanta kodo uzas `framebuffer` kaj `ekrano`. |
| `seria konzolo` | serial console | 串口控制台 | COM1. |
| `tiparo` | font | 字体 | |
| `glifo` | glyph | 字形 | |
| `streko` | strike / font size | 字号档位 | |
| `koloro` | color | 颜色 | |
| `pikselo` | pixel | 像素 | |
| `kaŝmemoro` | cache | 缓存 | |
| `nekaŝebla` | uncacheable | 不可缓存 | PCD. |
| `plenumebla` | executable | 可执行 | NX. |
| `skribebla` | writable | 可写 | |
| `ĉeesta` | present | 存在 | |
| `grandega paĝo` | huge page | 大页 | |
| `aliniiĝinta` | aligned | 对齐 | |

## Protokolaj kaj aparataraj nomoj

La jenaj nomoj restas en sia originala formo, ĉar ili estas aparataj aŭ
eksteraj protokolaj nomoj:

`x86_64`, `CR0`, `CR2`, `CR3`, `CR4`, `RSP0`, `RIP`, `RFLAGS`, `RAX`, `RDI`,
`RSI`, `RDX`, `R10`, `MSR`, `EFER`, `NXE`, `PAE`, `PML4`, `PDPT`, `PD`, `PT`,
`IDT`, `GDT`, `TSS`, `APIC`, `PIT`, `COM1`, `Multiboot2`, `Xen-PVH`, `ELF`,
`Limine`, `QEMU`, `int 0x80`, `iretq`, `syscall`, `sysret`.

## Reguloj

1. Novaj tipoj, funkcioj, kampoj kaj lokaj variabloj uzu Esperantajn vortojn.
2. Ne uzu `cpu`, `ram`, `memory`, `thread`, `scheduler`, `page`, `frame` en
   identigiloj.
3. Komentoj kaj logaj tekstoj estas Esperantaj.
4. Kiam necesas mencii anglan mallongigon, skribu unue la Esperantan nomon kaj
   poste la mallongigon en parentezo: `la tutmonda priskriba tabelo (GDT)`.
5. Registraj nomoj, instrukcioj kaj eksteraj protokoloj restas netuŝitaj.
