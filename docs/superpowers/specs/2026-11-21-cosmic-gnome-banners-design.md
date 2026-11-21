# GNOME-style notification banners for COSMIC — design

Date: 2026-11-21 · Target: COSMIC 1.11.0 on Fedora 45 · Sibling of Clock Center

## Intent

The user finds COSMIC's notification banners "soo ugly" (big separate boxes,
lots of empty space) and wants them to look like GNOME's: a compact card under
the clock with an action-button row. COSMIC exposes no theming or layout hook
for banners, and no existing applet or extension modifies them. Swapping in a
different daemon (mako/swaync/noti) would break Clock Center, which receives
notifications from cosmic-notifications through the panel; making COSMIC's DND
permanently on so Clock Center can draw its own banners was rejected by the
user. So: a modified copy of cosmic-notifications that only changes how a
banner is drawn.

## What the user decided

- **Approach A:** fork `pop-os/cosmic-notifications` and change only the banner
  view; run it instead of the stock daemon.
- **Look (GNOME reference):** one compact rounded card per notification,
  ~420px wide, centred under the clock (existing `anchor = Top`); 8px between
  banners; large app icon left; bold title; body up to 2 lines then ellipsis;
  × in the top-right corner; action buttons as an equal-width row along the
  bottom, only when the app sends actions.
- **Behaviour:** click card → activate (default action, same as today); × →
  dismiss. Timeouts, max banners, max per app and anchor keep coming from
  COSMIC Settings (`com.system76.CosmicNotifications`).
- **Unchanged:** Do Not Disturb semantics; the panel/applet channel that Clock
  Center uses; Clock Center's popup-open DND behaviour.
- Public repo `agustine-leo/cosmic-ext-notifications`, GPL-3.0 (inherited).

## Goals
1. Banners render as the GNOME-style card above, in the active COSMIC theme
   (colours, corner radii, frosted glass if enabled).
2. Zero behaviour change outside banner rendering: same D-Bus server
   (`org.freedesktop.Notifications`), same applet socket protocol, same config
   keys, same DND, same timeouts/limits, same action invocation.
3. Install for the user without editing files under `/usr`: binary at
   `/usr/local/bin/cosmic-notifications`, which cosmic-session finds first
   (session PATH is `/usr/local/bin:/usr/bin:…`). `just uninstall` removes it
   so the stock daemon runs again at next login.
4. Easy to rebase on future COSMIC releases: banner rendering isolated in one
   new module (`src/banner.rs`); `src/app.rs` changes limited to calling it and
   the popup width.
5. Built by the dotfiles install (`scripts/40-cosmic.sh`), like Clock Center.

## Non-goals
- Changing banner position logic, stacking, timeouts, history, DND, the applet
  protocol, or adding new settings.
- Restyling anything other than banners (no panel applet in this repo).
- Packaging (flatpak/RPM). Source + `just install` only.

## Anti-goals
- Clock Center stops receiving notifications, or DND stops suppressing
  banners — failure even if banners look right.
- Leaving the user with no notification daemon (e.g. a broken
  `/usr/local/bin/cosmic-notifications` that crashes) with no easy way back.
- Writing to `/usr/bin` or other package-owned paths.

## Constraints and facts (cosmic-notifications epoch-1.11.0, commit 382927d)
- Fedora runs `cosmic-notifications-1.11.0-2.fc45`; fork that exact tag; keep
  its Cargo.lock (libcosmic rev) so theming/protocols match the running COSMIC.
- Banners: an anchored layer surface hosts a chain of xdg popups, one per
  visible card; each popup is sized via `autosize` with limits
  `min_width/max_width 300` (app.rs ~320-405) and its content is
  `cosmic::widget::cards(...)` with width `Fixed(300.)` (app.rs ~460-545).
  Replace that content with the new card and set width 420 in all three
  limit sites. Keep the chaining/positioning code as is.
- Click/dismiss/action messages already exist in app.rs (activation picks the
  `default` action or the first; ~650-680). Reuse them; add only a message for
  invoking a specific named action if none exists.
- cosmic-session launches `cosmic-notifications` by name from PATH and
  restarts it with backoff; the panel passes a socket to the notifications
  applet only after the daemon is up.
- Restarting the daemon mid-session breaks Clock Center's socket ("Broken
  pipe") until re-login — install takes effect at next login; never restart
  daemon/panel during development (lesson from Clock Center).
- `sudo` is needed once per install (copy into /usr/local/bin).

## Approaches considered
- **A. Fork cosmic-notifications, restyle banners only — chosen.** Keeps DND
  and Clock Center intact; cost: reapply on each COSMIC update.
- B. Replace with mako/swaync/noti — rejected: Clock Center loses its feed.
- C. Clock Center draws banners with COSMIC DND forced on — rejected by user
  (DND would show permanently on in COSMIC Settings).

## Decisions for the build (my calls unless noted)
- Repo `agustine-leo/cosmic-ext-notifications`, local clone
  `~/.local/src/cosmic-ext-notifications`, created from upstream tag
  `epoch-1.11.0` with upstream history kept (so later rebases are git merges);
  upstream remote kept as `upstream`.
- New `src/banner.rs`: `pub fn card<'a>(n: &'a Notification, …) -> Element<'a, Message>`
  plus pure helpers with unit tests: `body_preview(&str) -> String` (strip
  markup, collapse whitespace; 2-line clamp done by ellipsize), and
  `action_buttons(&[(ActionId, String)]) -> Vec<(ActionId, String)>` (drops the
  `default` action from the visible row since clicking the card triggers it;
  max 3 buttons).
- Card: `Container::Card`-like theme container, padding 12, icon 40px,
  title `text::heading` 1 line ellipsized, body `text::body` 2 lines ellipsized,
  × = embedded SVG (user's icon theme blanks `window-close-symbolic`), action
  row = equal-width `button::text` separated by 1px dividers, flush to the
  card's bottom edge.
- Width 420 everywhere banners are sized (constant `BANNER_WIDTH`).
- justfile: `build-release` (`--locked`), `install` (sudo install -m755 to
  /usr/local/bin/cosmic-notifications), `uninstall` (sudo rm). README with
  screenshots placeholder, warning, how to revert, credits.
- dotfiles `scripts/40-cosmic.sh`: clone/pull + `just install`, failures only
  warn (`set -e` safe), same pattern as Clock Center.
- Release v0.1.0 when tests + build + install pass; the user sees it after
  their next login.

## Decisions after builder check
- **Preview harness (my call):** `cargo run --example preview` opens a normal
  libcosmic window rendering `banner::card` for sample notifications (plain,
  long body, 1 action, 3+ actions, no icon). This is how the look is judged
  before any install; `banner::card` must not depend on daemon state.
- **Safe install (my call):** `/usr/local/bin/cosmic-notifications` is a small
  POSIX sh launcher that runs the fork from
  `/usr/local/libexec/cosmic-ext-notifications/cosmic-notifications`; if the
  fork exits non-zero within 5 seconds of start, the launcher `exec`s
  `/usr/bin/cosmic-notifications`. A broken build can't leave the user without
  a daemon.
- **Version guard (my call):** `just install` refuses unless
  `rpm -q --qf '%{VERSION}' cosmic-notifications` prints `1.11.0`;
  `scripts/40-cosmic.sh` only warns and skips on mismatch. Rebase the fork
  before supporting a new COSMIC version.
- **Actions (my call):** at most 3 buttons; `default` is never a button
  (clicking the card triggers it); a single non-default action (e.g. "Open")
  still shows as a button, as GNOME does.
- **Widths/radius (my call):** every banner size site in app.rs uses
  `BANNER_WIDTH` (420), including `size: Some((Some(300), …))` and the
  positioner `width`; the `corner_radius(...)` call uses the same radius as
  the card container so frosted corners line up.
- **Approved by the user:** create and push the public repo, tag v0.1.0 after
  tests+build pass, run the sudo install (takes effect next login), and add the
  dotfiles build step (left uncommitted until the user says so).

## Left to the builder
Exact theming calls, divider widget choice, how the × is overlaid, module
wiring, test layout.

## Testing
- Unit: `body_preview` (markup stripped, whitespace collapsed),
  `action_buttons` (default dropped, max 3, order kept).
- Build `--locked`, `cargo test`.
- After user's next login: `notify-send "Title" "Body"` → GNOME card under the
  clock; `notify-send -A open=Open -A later=Later "T" "B"` → two-button row;
  clicking a button invokes that action (`notify-send --wait` prints the action
  id); × dismisses; DND on → no banner, Clock Center still lists it.
- Revert check: `just uninstall` → next login runs `/usr/bin/cosmic-notifications`.
