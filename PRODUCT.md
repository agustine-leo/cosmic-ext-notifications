# Product

<!-- impeccable:product-schema 1 -->

## Platform

linux-desktop (COSMIC 1.11 on Fedora; native Rust, libcosmic/iced layer-shell surfaces — not web, not mobile)

## Users

One person on their own laptop (ThinkPad, Fedora + COSMIC), working at the desk. Banners interrupt whatever they are doing: they glance, decide in about a second whether to act, and either click, dismiss, or let it time out.

## Product Purpose

A fork of COSMIC's notifications daemon that changes only how a notification banner is drawn. Success: a banner that is read at a glance and has a recognisable identity of its own on this desktop, instead of a clone of another desktop's banner.

## Positioning

The only way to restyle COSMIC banners: COSMIC exposes no theming or layout hook for them. The fork keeps the stock D-Bus server, applet socket, config keys and behaviour, so Clock Center (the user's panel applet) keeps receiving notifications.

## Operating Context

- Banners appear top-centre, stacked under Clock Center (the clock + notification-centre panel applet), 8px apart.
- Main traffic: chat (WhatsApp via Whatsie, sender pictures + message preview), browser and web apps (Zen Browser via portal: mail, calendar, sites), and system (battery, disk, downloads, updates, Tailscale).
- Desktop theme: COSMIC dark, Nordic GTK theme for flatpaks, bold Cantarell UI font.

## Capabilities and Constraints

- Per notification: app icon or sender picture (image-data raster, kept sharp at output scale), summary, body (markup stripped), up to three app actions, click-to-activate, explicit dismiss.
- Behaviour fixed by upstream: timeouts, max banners, max per app, anchor, grouping, Do Not Disturb (from `com.system76.CosmicNotifications`). Critical urgency bypasses DnD.
- Built against upstream tag `epoch-1.11.0` and its Cargo.lock; banner drawing lives in `src/banner.rs`, `src/app.rs` changes stay minimal so rebases stay easy.
- Restarting the daemon mid-session breaks Clock Center's socket until re-login (or a panel restart).
- No settings UI for banner look; no new config keys (non-goal).

## Brand Commitments

None binding. Previous look (GNOME-style card) is rejected as "generic".

## Evidence on Hand

- `examples/preview.rs`: renders sample banners in a window for visual checks.
- Original intent and constraints: `docs/superpowers/specs/2026-11-21-cosmic-gnome-banners-design.md`.

## Product Principles

1. A glance is the whole interaction: sender and gist must land in under a second.
2. Never break the pipeline: Clock Center, DnD and actions working beats any look.
3. Draw only the banner; everything else stays upstream.
