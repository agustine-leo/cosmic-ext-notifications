#!/bin/sh
# Tests for data/cosmic-notifications-launcher.sh. Run: sh tests/launcher.sh
here=$(cd "$(dirname "$0")/.." && pwd)
launcher="$here/data/cosmic-notifications-launcher.sh"
fail=0

check() { # name expected actual
    if [ "$2" = "$3" ]; then
        echo "ok   - $1"
    else
        echo "FAIL - $1: expected '$2', got '$3'"
        fail=1
    fi
}

[ -f "$launcher" ] || { echo "FAIL - launcher missing: $launcher"; exit 1; }

# Fork fails immediately -> stock is exec'd with STOCK_ARGS.
out=$(FORK=/bin/false STOCK=/bin/echo STOCK_ARGS=stock-started sh "$launcher" 2>/dev/null)
check "fork fails fast -> stock started" "stock-started" "$out"

# Fork exits 0 -> stock not invoked, launcher exits 0.
out=$(FORK=/bin/true STOCK=/bin/echo STOCK_ARGS=stock-started sh "$launcher")
status=$?
check "fork succeeds -> stock not invoked" "" "$out"
check "fork succeeds -> exit 0" "0" "$status"

# Fork fails after the 5 s window -> stock not invoked, fork status kept.
slow=$(mktemp)
printf '#!/bin/sh\nsleep 6\nexit 3\n' > "$slow"
chmod +x "$slow"
out=$(FORK="$slow" STOCK=/bin/echo STOCK_ARGS=stock-started sh "$launcher")
status=$?
rm -f "$slow"
check "fork fails late -> stock not invoked" "" "$out"
check "fork fails late -> fork status" "3" "$status"

# TERM is forwarded to the fork, and does not trigger the fallback.
marker=$(mktemp)
term=$(mktemp)
printf '#!/bin/sh\ntrap "echo got-term > %s; exit 1" TERM\nwhile :; do sleep 0.1; done\n' "$marker" > "$term"
chmod +x "$term"
out_file=$(mktemp)
FORK="$term" STOCK=/bin/echo STOCK_ARGS=stock-started sh "$launcher" > "$out_file" 2>/dev/null &
lpid=$!
sleep 1
kill -TERM "$lpid"
wait "$lpid"
sleep 0.3
check "TERM forwarded to fork" "got-term" "$(cat "$marker")"
check "TERM -> no fallback" "" "$(cat "$out_file")"
rm -f "$marker" "$term" "$out_file"

[ "$fail" = 0 ] && echo "all launcher tests passed"
exit "$fail"
