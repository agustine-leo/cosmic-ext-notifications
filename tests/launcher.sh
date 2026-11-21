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

# Stray session variables with the old, unprefixed names must be ignored.
# They are set to harmless values so no test can ever reach a real daemon.
export FORK=/bin/false STOCK=/bin/echo STOCK_ARGS=stray-redirect

# Version stamp written by `just install`; matches the "installed" stock rpm.
stamp=$(mktemp)
echo 1.11.0 > "$stamp"
export CEN_STAMP="$stamp" CEN_RPM_VERSION=1.11.0

# Fork fails immediately -> stock is exec'd with CEN_STOCK_ARGS.
out=$(CEN_FORK=/bin/false CEN_STOCK=/bin/echo CEN_STOCK_ARGS=stock-started sh "$launcher" 2>/dev/null)
check "fork fails fast -> stock started" "stock-started" "$out"

# Fork exits 0 -> stock not invoked, launcher exits 0.
out=$(CEN_FORK=/bin/true CEN_STOCK=/bin/echo CEN_STOCK_ARGS=stock-started sh "$launcher")
status=$?
check "fork succeeds -> stock not invoked" "" "$out"
check "fork succeeds -> exit 0" "0" "$status"

# Fork fails after the 5 s window -> stock not invoked, fork status kept.
slow=$(mktemp)
printf '#!/bin/sh\nsleep 6\nexit 3\n' > "$slow"
chmod +x "$slow"
out=$(CEN_FORK="$slow" CEN_STOCK=/bin/echo CEN_STOCK_ARGS=stock-started sh "$launcher")
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
CEN_FORK="$term" CEN_STOCK=/bin/echo CEN_STOCK_ARGS=stock-started sh "$launcher" > "$out_file" 2>/dev/null &
lpid=$!
sleep 1
kill -TERM "$lpid"
wait "$lpid"
sleep 0.3
check "TERM forwarded to fork" "got-term" "$(cat "$marker")"
check "TERM -> no fallback" "" "$(cat "$out_file")"
rm -f "$marker" "$term" "$out_file"

# Version stamp: a fork that records it was started.
started=$(mktemp -u)
fork=$(mktemp)
printf '#!/bin/sh\necho fork > %s\nexit 0\n' "$started" > "$fork"
chmod +x "$fork"
was_started() { [ -e "$started" ] && echo yes || echo no; }

# Stock rpm upgraded since install -> stock exec'd, fork never started.
rm -f "$started"
out=$(CEN_RPM_VERSION=1.12.0 CEN_FORK="$fork" CEN_STOCK=/bin/echo CEN_STOCK_ARGS=stock-started sh "$launcher" 2>/dev/null)
check "version mismatch -> stock started" "stock-started" "$out"
check "version mismatch -> fork not started" "no" "$(was_started)"

# Stock rpm matches the stamp -> fork started, stock not invoked.
rm -f "$started"
out=$(CEN_FORK="$fork" CEN_STOCK=/bin/echo CEN_STOCK_ARGS=stock-started sh "$launcher" 2>/dev/null)
check "version match -> stock not invoked" "" "$out"
check "version match -> fork started" "yes" "$(was_started)"

# Stamp missing -> stock.
rm -f "$started"
out=$(CEN_STAMP=/nonexistent/built-for-version CEN_FORK="$fork" CEN_STOCK=/bin/echo CEN_STOCK_ARGS=stock-started sh "$launcher" 2>/dev/null)
check "stamp missing -> stock started" "stock-started" "$out"
check "stamp missing -> fork not started" "no" "$(was_started)"

# Stock version unknown (rpm missing / package not installed) -> stock.
rm -f "$started"
out=$(CEN_RPM_VERSION='' CEN_FORK="$fork" CEN_STOCK=/bin/echo CEN_STOCK_ARGS=stock-started sh "$launcher" 2>/dev/null)
check "stock version unknown -> stock started" "stock-started" "$out"
check "stock version unknown -> fork not started" "no" "$(was_started)"

rm -f "$started" "$fork" "$stamp"

[ "$fail" = 0 ] && echo "all launcher tests passed"
exit "$fail"
