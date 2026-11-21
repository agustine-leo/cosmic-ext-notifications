# GNOME-style COSMIC Banners Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A fork of cosmic-notifications 1.11.0 whose only change is GNOME-style banner cards, installed safely via `/usr/local/bin` with a fallback launcher.

**Architecture:** Fork upstream at `epoch-1.11.0` keeping history. A new `src/banner.rs` renders one card, generic over the message type so a preview example can render it without the daemon. `src/app.rs` swaps the stock `cards(...)` content for `banner::card` and uses `BANNER_WIDTH` at every banner size site. A POSIX sh launcher falls back to the stock daemon if the fork dies at start.

**Tech Stack:** Rust (Fedora rustc 1.100), libcosmic as locked by upstream `Cargo.lock` at 382927d, just, POSIX sh.

**Spec:** `/home/agustine/dotfiles/docs/superpowers/specs/2026-11-21-cosmic-gnome-banners-design.md` (sections "Decisions for the build" and "Decisions after builder check" bind)

## Global Constraints

- Repo `agustine-leo/cosmic-ext-notifications` (public), clone `~/.local/src/cosmic-ext-notifications`, based on upstream tag `epoch-1.11.0` (382927d) with upstream history; remote `upstream` = `https://github.com/pop-os/cosmic-notifications`.
- Licence GPL-3.0 (inherited). README credits pop-os/cosmic-notifications.
- `BANNER_WIDTH = 420.0`; banner gap 8px; card padding 12; icon 40px; title 1 line ellipsized; body 2 lines ellipsized; × is an embedded SVG; action row equal-width buttons with 1px dividers, flush to the card bottom.
- Action buttons: max 3; `ActionId::Default` never shown as a button; one non-default action still shown.
- No behaviour change outside banner rendering (D-Bus server, applet socket, config keys, DND, timeouts, limits, grouping, activation).
- Install: binary → `/usr/local/libexec/cosmic-ext-notifications/cosmic-notifications`; launcher → `/usr/local/bin/cosmic-notifications`; launcher falls back to `/usr/bin/cosmic-notifications` if the fork exits non-zero within 5 s. `just install` refuses unless `rpm -q --qf '%{VERSION}' cosmic-notifications` = `1.11.0`.
- Never restart cosmic-panel or cosmic-notifications during development; the user sees the result after next login.
- Builds use `--locked`. Commits end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Dotfiles changes stay uncommitted.

## Review Focus

1. Notification with an empty body (`notify-send "Title"`) — card shows title only, no blank second line → Task 2 test `body_preview_empty_is_none`.
2. Body with markup/links (`<b>`, `<a href>`) and long whitespace runs — rendered as clean text → Task 2 test `body_preview_strips_markup_and_collapses_whitespace`.
3. App with 4+ actions or only a `default` action — at most 3 buttons; default-only → no button row → Task 2 tests `action_buttons_caps_at_three`, `action_buttons_default_only_is_empty`.
4. Fork crashes at login — stock daemon starts instead → Task 4 launcher test `launcher_falls_back_when_fork_fails_fast`.
5. COSMIC upgraded past 1.11.0 — install refuses; dotfiles step warns and skips → Task 4 `just install` guard check + Task 5 script guard.

---

### Task 1: Fork the repo

**Files:** new repo (no source changes).

- [ ] **Step 1:** `gh repo create agustine-leo/cosmic-ext-notifications --public --description "COSMIC notifications daemon with GNOME-style banners (fork of pop-os/cosmic-notifications)"`; clone upstream at tag: `git clone --branch epoch-1.11.0 https://github.com/pop-os/cosmic-notifications ~/.local/src/cosmic-ext-notifications`; `git remote rename origin upstream`; `git remote add origin https://github.com/agustine-leo/cosmic-ext-notifications`; `git checkout -b main`.
- [ ] **Step 2: Verify:** `cargo build --release --locked` succeeds (stock code); `git log --oneline -1` = 382927d.
- [ ] **Step 3:** Copy the spec and this plan into `docs/superpowers/{specs,plans}/`; commit `docs: banner redesign spec and plan`; `git push -u origin main`.

### Task 2: Pure banner helpers

**Files:** Create `src/banner.rs` (helpers + tests); Modify `src/main.rs` (`mod banner;`).

**Interfaces:**
- Produces: `pub const BANNER_WIDTH: f32 = 420.0;`, `pub fn body_preview(body: &str) -> Option<String>`, `pub fn action_buttons(actions: &[(ActionId, String)]) -> Vec<(ActionId, String)>`.

- [ ] **Step 1: Failing tests** in `src/banner.rs`:
```rust
#[test] fn body_preview_strips_markup_and_collapses_whitespace() {
    assert_eq!(body_preview("<b>Disk</b>   is\n\n  <a href=\"x\">full</a>").as_deref(), Some("Disk is full"));
}
#[test] fn body_preview_empty_is_none() { assert_eq!(body_preview("  \n "), None); }
#[test] fn action_buttons_drops_default_and_keeps_order() {
    let a = vec![(ActionId::Default, "Open".into()), (ActionId::Custom("snooze".into()), "Snooze".into()), (ActionId::Custom("stop".into()), "Stop".into())];
    assert_eq!(action_buttons(&a).iter().map(|(_, l)| l.as_str()).collect::<Vec<_>>(), ["Snooze", "Stop"]);
}
#[test] fn action_buttons_caps_at_three() {
    let a: Vec<_> = (0..5).map(|i| (ActionId::Custom(i.to_string()), format!("A{i}"))).collect();
    assert_eq!(action_buttons(&a).len(), 3);
}
#[test] fn action_buttons_default_only_is_empty() {
    assert!(action_buttons(&[(ActionId::Default, "Open".into())]).is_empty());
}
#[test] fn action_buttons_single_custom_is_shown() {
    assert_eq!(action_buttons(&[(ActionId::Custom("open".into()), "Open".into())]).len(), 1);
}
```
- [ ] **Step 2:** `cargo test --locked banner` → FAIL (missing fns).
- [ ] **Step 3:** Implement; strip markup by dropping `<…>` tags where `<` is followed by an ASCII letter or `/`, and decode `&amp; &lt; &gt; &quot; &#39;`.
- [ ] **Step 4:** `cargo test --locked banner` → PASS. Commit `feat: banner text and action helpers`.

### Task 3: Banner card view + preview example

**Files:** Modify `src/banner.rs`; Create `examples/preview.rs`.

**Interfaces:**
- Consumes: Task 2 helpers.
- Produces: `pub fn card<'a, M: Clone + 'a>(n: &'a Notification, on_activate: M, on_dismiss: M, on_action: impl Fn(ActionId) -> M + 'a) -> cosmic::Element<'a, M>` — no dependency on `crate::app`.

- [ ] **Step 1:** Implement `card` per Global Constraints: theme card container (corner radius `radius_m`, same value later passed to `corner_radius(...)` in Task 4), padding 12; row: icon 40px (notification icon/image, else `dialog-information-symbolic`), column(title `text::heading` 1 line ellipsized, body `text::body` 2 lines ellipsized when `body_preview` is Some); × embedded SVG button top-right → `on_dismiss`; whole upper area → `on_activate` (mouse_area); if `action_buttons` non-empty: horizontal divider then a row of equal-width (`Length::Fill`) `button::text` separated by 1px vertical dividers, each → `on_action(id)`. Width `Length::Fixed(BANNER_WIDTH)`.
- [ ] **Step 2:** `examples/preview.rs`: a `cosmic::Application` window (`#[path = "../src/banner.rs"] mod banner;`) that stacks `banner::card` for 5 sample notifications: plain; long multi-line body; one custom action "Open"; actions Snooze/Stop + Examine Disk (3); no icon. Messages just log.
- [ ] **Step 3: Verify:** `cargo build --release --locked --example preview`; `cargo test --locked`. Ask the controller to have the user run `cargo run --release --example preview` and approve the look before Task 4 (visual gate).
- [ ] **Step 4:** Commit `feat: GNOME-style banner card with preview example`; push.

### Task 4: Wire into the daemon, launcher, install

**Files:** Modify `src/app.rs` (banner content ~460-545; size sites ~318, ~320-325, ~354-357, ~361, ~402-405; `corner_radius(...)` ~552; add `Message::InvokeAction(u32, ActionId)`); Create `data/cosmic-notifications-launcher.sh`, `justfile`, `tests/launcher.sh`; Modify `README.md`.

**Interfaces:**
- Consumes: `banner::card`, `BANNER_WIDTH`.

- [ ] **Step 1: Failing launcher test** `tests/launcher.sh` (POSIX sh, run with `sh tests/launcher.sh`): run the launcher with env overrides `FORK=/bin/false STOCK=/bin/echo` and assert stdout is `stock-started` (STOCK invoked with arg `stock-started` via `STOCK_ARGS`); and with `FORK=/bin/true` assert STOCK is not invoked (fork exited 0 → launcher exits 0). Expected first run: FAIL (launcher missing).
- [ ] **Step 2: Implement** `data/cosmic-notifications-launcher.sh`: defaults `FORK=/usr/local/libexec/cosmic-ext-notifications/cosmic-notifications`, `STOCK=/usr/bin/cosmic-notifications`; start fork in background, record start time; `wait`; if exit status ≠ 0 and elapsed < 5 s → `exec "$STOCK" "$@"`; else exit with the fork's status. Must forward signals (trap TERM/INT → kill child). `sh tests/launcher.sh` → PASS.
- [ ] **Step 3: Wire app.rs:** replace the `cosmic::widget::cards(...)` content with a column of `banner::card(n, Message::ActivateNotification(id), Message::Dismissed(id), move |a| Message::InvokeAction(id, a))` spaced 8; replace every `300`/`300.0` banner size with `BANNER_WIDTH`; `corner_radius(...)` uses `radius_m`. `InvokeAction(id, action)` follows the same path as `ActivateNotification` but with the given action (reuse `ActivationToken(…, id, Some(action))`).
- [ ] **Step 4: justfile** recipes: `build-release` (`cargo build --release --locked`); `install` (guard: `[ "$(rpm -q --qf '%{VERSION}' cosmic-notifications)" = 1.11.0 ] || { echo "cosmic-notifications is not 1.11.0 — rebase the fork first"; exit 1; }`, then `sudo install -Dm755` binary to the libexec path and launcher to `/usr/local/bin/cosmic-notifications`); `uninstall` (`sudo rm -f` both). README: what it is, screenshot placeholder, install/uninstall, "takes effect at next login", the fallback launcher, the version guard, credits.
- [ ] **Step 5: Verify:** `cargo test --locked`; `cargo build --release --locked` 0 warnings; `sh tests/launcher.sh`; `just --dry-run install` shows the guard. Commit `feat: GNOME-style banners in the daemon, safe launcher and install`; push; tag `v0.1.0`; push tag.
- [ ] **Step 6:** Hand the controller the exact `just install` command for the user to run (needs their sudo password); no restarts.

### Task 5: Dotfiles integration (uncommitted)

**Files:** Modify `/home/agustine/dotfiles/scripts/40-cosmic.sh`.

- [ ] **Step 1:** After the Clock Center block, add a block that: skips with `log_warn` unless `rpm -q --qf '%{VERSION}' cosmic-notifications` = `1.11.0`; clones/pulls `agustine-leo/cosmic-ext-notifications` into `~/.local/src/`; runs `just install`; every failure only `log_warn`s ("GNOME banners not installed — COSMIC's stock banners stay. Fix: cd ~/.local/src/cosmic-ext-notifications && just install"). Must be `set -euo pipefail` safe (wrap in `if { …; }; then …; else …; fi`).
- [ ] **Step 2:** `bash -n scripts/40-cosmic.sh`. Do not commit.
