#!/bin/sh
# cosmic-ext-notifications launcher, installed as /usr/local/bin/cosmic-notifications
# (ahead of /usr/bin in PATH, so cosmic-session starts it instead of the stock daemon).
#
# Runs the fork. If the fork exits non-zero within 5 seconds of starting
# (crash on startup, missing library, ...), it hands over to the stock
# daemon so the session never ends up without notifications.
#
# FORK, STOCK and STOCK_ARGS may be overridden from the environment (tests).

FORK=${FORK:-/usr/local/libexec/cosmic-ext-notifications/cosmic-notifications}
STOCK=${STOCK:-/usr/bin/cosmic-notifications}
WINDOW=5

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
    # shellcheck disable=SC2086 # STOCK_ARGS is a word list on purpose
    exec "$STOCK" $STOCK_ARGS "$@"
fi

exit "$status"
