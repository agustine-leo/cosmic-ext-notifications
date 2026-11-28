---
version: 1
slug: "src-banner-rs"
primary_target: "src/banner.rs"
related_targets: []
---

## Scope

Notification banner (`src/banner.rs`), drawn by the cosmic-ext-notifications daemon under Clock Center. Mode: Operate. Glanced at for about a second; chat, browser/web apps and system traffic.

## Direction contract

THESIS: The banner stack is a printed Nordic railway timetable: time first, then who, then what. It refuses the category's rounded icon-tile card (macOS/GNOME), which the user rejected as generic.

OWN-WORLD: Deep navy strip (#121c28), paper-white type (#f2f0ea), slate secondary (#8c98a6), one timetable red (#c8243a) reserved for critical. Cantarell (the desktop face), bold tabular time figures. A hairline column rule (#2a3647) splits time from content like a timetable's ruled columns. Flat, no shadow, 6px radius.

STORY: The user sees when and from which app in the left column, who and what on the right, and either clicks to open, presses an action, dismisses with ×, or lets it time out.

FIRST VIEWPORT: 420px strips stacked 8px apart top-centre. Left column 72px: bold 20px time, app name below it in small slate type. Right column: sender picture/app icon 32px beside bold title and two-line body; × top right; actions as bold text buttons under the body. Critical: the time column fills timetable red with paper-white figures.

FORM: Nordic Timetable, position 1 on my ordered list (IMPECCABLE'S PICK, chosen by user over the rolled candidate 7). seed:4b8e7f0ef18b1654. Signature move: the time column at a fixed x on every banner, so a stack reads as one departure table.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
