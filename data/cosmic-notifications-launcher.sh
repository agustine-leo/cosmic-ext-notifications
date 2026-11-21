#!/bin/sh
# cosmic-ext-notifications launcher, installed as /usr/local/bin/cosmic-notifications
# (ahead of /usr/bin in PATH, so cosmic-session starts it instead of the stock daemon).
#
# Runs the fork, unless the stock cosmic-notifications rpm is no longer the
# version the fork was built for (recorded by `just install` in the
# built-for-version stamp): after a COSMIC upgrade the stock daemon is
# started directly until the fork is rebased and reinstalled.
#
# If the fork exits non-zero within 5 seconds of starting
# (crash on startup, missing library, ...), it hands over to the stock
# daemon so the session never ends up without notifications.
#
# CEN_FORK, CEN_STOCK, CEN_STOCK_ARGS, CEN_STAMP and CEN_RPM_VERSION (the
# current stock version, normally from rpm) may be overridden from the
# environment (tests). The prefix keeps stray session variables from
# redirecting the daemon.

FORK=${CEN_FORK:-/usr/local/libexec/cosmic-ext-notifications/cosmic-notifications}
STOCK=${CEN_STOCK:-/usr/bin/cosmic-notifications}
STOCK_ARGS=${CEN_STOCK_ARGS:-}
STAMP=${CEN_STAMP:-/usr/local/libexec/cosmic-ext-notifications/built-for-version}
WINDOW=5

if [ -n "${CEN_RPM_VERSION+set}" ]; then
    current=$CEN_RPM_VERSION
else
    current=$(timeout 3 rpm -q --qf '%{VERSION}' cosmic-notifications 2>/dev/null) || current=
fi
built=$(cat "$STAMP" 2>/dev/null) || built=
if [ -z "$built" ] || [ "$built" != "$current" ]; then
    echo "cosmic-ext-notifications was built for cosmic-notifications '$built', installed is '$current'; starting $STOCK" >&2
    # shellcheck disable=SC2086 # CEN_STOCK_ARGS is a word list on purpose
    exec "$STOCK" $STOCK_ARGS "$@"
fi

start=$(date +%s)
"$FORK" "$@" &
child=$!

stopping=0
forward() {
    stopping=1
    kill -"$1" "$child" 2>/dev/null
}
trap 'forward TERM' TERM
trap 'forward INT' INT
trap 'forward HUP' HUP

# `wait` returns early when a trapped signal arrives; keep waiting until the
# child has actually exited.
while :; do
    wait "$child"
    status=$?
    kill -0 "$child" 2>/dev/null || break
done

if [ "$stopping" = 0 ] && [ "$status" -ne 0 ] && [ $(( $(date +%s) - start )) -lt "$WINDOW" ]; then
    echo "cosmic-ext-notifications exited with status $status at startup; starting $STOCK" >&2
    # shellcheck disable=SC2086 # CEN_STOCK_ARGS is a word list on purpose
    exec "$STOCK" $STOCK_ARGS "$@"
fi

exit "$status"
