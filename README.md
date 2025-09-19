# OTL - "Sidekick Plus" Outliner file decoder

Provide comprehension of the file format (extension .OTL) used by the (circa 1988) Borland MS-DOS "Sidekick Plus" Outliner (called Outlook).

# Build & Dev

- Build: `make build` (use `RELEASE=0` for debug). Binary: `target/(release|debug)/otl`.
- Run: `make run ARGS='--canon path/to/file.OTL'`
- JSON: `make json FILE=path/to/file.OTL`
- Diff: `make diff PREV=prev.OTL CURR=curr.OTL CURSOR=1`
- Watch: `make watch TARGET=<file|dir> ARGS='--validate'` (needs `inotifywait`, `diff`, `awk`).
- Hygiene: `make check` (fmt + clippy + test). All targets: `make help`.

# Storytime

As someone who *actually used* DOS-era outliners (and found them VERY
useful, and lamented their passing from the scene), I have periodically pined
for them over the ensuing decades, and have even taken stabs over the past
4-5 yearrs at running Grandview 2.0 in DOSbox, and found it partially
satisfying, but not compelling, in light of the integration challenges any
DOSbox-hosted app presents (primarily, lack of host-OS-clipboard
integration).

Aside: in the past month (25.07), I tried using `Logseq` which was touted as
one of the mainstream OSS "desktop" apps maintaining the legacy of DOS-era
outliners (or perhaps more accurately as a inheritor of the later
extrapolations of the direction taken by GV2: PIM = outliner w/spreadsheet?).
The experience (running a 5yo app that appears to be somewhat popular) was
poor.  And as a recovering C/C++ dev & firmware engineer, I took a dim view
of the code bloat associated (to provide minimal functionality), which took
contributing to that project off the table as a possible direction for me.

In some respects I found the Sidekick Plus outliner (I'll call it SKPOTL)
superior: simplicity (vs GV2 which added a bunch of PIM features which I
never used) and (of all things) outline expand/collapse commands which work
incrementally (vs GV2's which are "all levels at once"), plus a different
"note editing" mode which might be more intuitive.  But it had been so very
long since I had used SKPOTL, that most of my memories of actual capabilities
beyond that had faded almost into oblivion.

Days ago I ran SKPOTL for the first time in DOSbox (on Linux).  While I'm
still toying with whether to actually use SKPOTL in anger, the answer depends
in part on whether an ability to import and export data in .OTL files to the
2025 world can be devised.  The creation of this simple utility, written with
gpt-5's help, which (fully?) comprehends the .OTL file format, solves (albeit
clumsily) the export problem.

However, the more I use SKPOTL in DOSbox, and with this repo's capability
established, I think the path of less resistance might be to recreate the
SKPOTL app as a Rust TUI app (using https://github.com/ratatui/ratatui?):

Cons:
 - WTF???  Recreating org-mode/logseq/Workflowy (with far far fewer features)?

Pros:
 - it appears to be a feasible 1-person part-time, AI-assisted project.
 - low bar: SKPOTL file format is fairly simple but seems capable of supporting most needs "as is", and "support" for it is already almost done.
 - ratatui is likely to be a complete solution to the TUI dev challenge.
 - Another rust crate is sure to offer host-OS-clipboard integration.

Base Feature-list:
 - OTL file read/write
 - import/export foreign structured content types, e.g. markdown (JSON?).
 - TUI rendering of OTL file.
 - Headline-text editing
 - Headline (OTL structure) modify-in-place commands (eg collapse/expand)
 - Headline (OTL structure) reorganization commands (eg move tree)
 - Note-text editing
 - host-OS-clipboard integration.

Extra Feature-list:
 - URL recognition (hiding) and "execution" (`xdg_open`/`start`?)


```
                                                      ch8 Outlook                         ch7 Notepad
                                                      |       Outlook Cmd Diagram          |
                                                      |       |   ch13 Installing          |
                                                      |       |   |       ch14 Changing    |
                                                      |       |   |       |       ch15 DYO |
                                                      |       |   |       |       |        |
cutpdf Borland.Sidekick.Plus.Owners.Handbook.1988.pdf 145-172,444,317-354,359-360,411-414,125-144
```
