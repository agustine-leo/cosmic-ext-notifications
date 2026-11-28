# cosmic-ext-notifications

A fork of COSMIC's notifications daemon
([pop-os/cosmic-notifications](https://github.com/pop-os/cosmic-notifications))
that draws notification banners set like a printed timetable: a 420px navy
strip with a fixed 72px time column (bold 24-hour figures in fixed cells, app
name below), a hairline rule, then the sender picture, bold title, two-line
body preview, a × close button and up to three text action buttons. Urgency
lives in the time column: slate figures for low, paper for normal, a red
column for critical. The design system is recorded in `DESIGN.md`.

Only the banner drawing changes. The D-Bus server, the applet socket, config
keys, Do Not Disturb, timeouts, limits, grouping and activation all behave as
in the stock daemon.

![Banner screenshot](docs/screenshot.png) <!-- placeholder -->

## Requirements

It is based on upstream tag `epoch-1.11.0` and only installs on top of the
stock `cosmic-notifications` package at version 1.11.0. `just install` checks
this with `rpm -q --qf '%{VERSION}' cosmic-notifications` and refuses to
install against any other version. When COSMIC updates, the fork has to be
rebased onto the new upstream tag first.

Build dependencies are the same as upstream: `cargo`, `just`, `pkg-config`,
`libxkbcommon-devel`.

## Install

```sh
just build-release
just install     # asks for your sudo password
```

This installs three files and leaves the stock package untouched:

- the fork's binary at `/usr/local/libexec/cosmic-ext-notifications/cosmic-notifications`
- a version stamp at `/usr/local/libexec/cosmic-ext-notifications/built-for-version`,
  holding the stock `cosmic-notifications` version the fork was built for (1.11.0)
- a small launcher script at `/usr/local/bin/cosmic-notifications`

`/usr/local/bin` comes before `/usr/bin` in `PATH`, so cosmic-session starts
the launcher instead of the stock daemon. The change takes effect at your next
login. Nothing is restarted during install.

## Fallback launcher

Before starting the fork, the launcher compares the version stamp with the
installed stock package (`rpm -q --qf '%{VERSION}' cosmic-notifications`). If
they differ, which is what happens after a `dnf upgrade` brings in a new
COSMIC, or if either is missing, it starts the stock daemon directly and the
fork stays out of the way until it is rebased and reinstalled.

Otherwise the launcher starts the fork. If the fork exits with an error within 5 seconds
of starting (a crash at startup, a missing library after an update), the
launcher starts the stock daemon `/usr/bin/cosmic-notifications` in its place,
so the session is never left without notifications. Signals from
cosmic-session (TERM, INT, HUP) are passed on to the fork and do not trigger
the fallback. `sh tests/launcher.sh` (or `just test-launcher`) tests it.

One known gap: if the fork crashes 1 to 5 seconds after starting, the
panel's notification applet may already have connected to it. The stock
daemon that takes over might not reconnect to the applet's socket, so the
notification list in the panel applet can stay empty until your next login.
Popups still appear, in the stock style.

## Uninstall, or if something breaks

To go back to COSMIC's stock daemon, remove the fork and log out and back in:

```sh
just uninstall
# or, without the repo at hand:
sudo rm -f /usr/local/bin/cosmic-notifications /usr/local/libexec/cosmic-ext-notifications/cosmic-notifications /usr/local/libexec/cosmic-ext-notifications/built-for-version
```

## Development

```sh
cargo test --locked
sh tests/launcher.sh
cargo run --release --example preview   # renders sample banners in a window
```

## Credits

The daemon is the work of System76 and the COSMIC contributors in
[pop-os/cosmic-notifications](https://github.com/pop-os/cosmic-notifications).
This fork only replaces the banner layout. Licensed GPL-3.0, like upstream.
