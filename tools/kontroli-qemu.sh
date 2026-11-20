#!/usr/bin/env bash
# Senkapa praŝarĝo de unu QEMU-varianto kun aŭtomata `fini`.
#
# Uzado: tools/kontroli-qemu.sh <sekundoj> <komando> [argumentoj...]
#
# La skripto atendas la Ligo-prompton `ligo:~>` sur la seria protokolo kaj
# skribas `fini`, ĉar la kerno nur tiam raportas la finan staton. Sukceso
# postulas la finan staton sen `MALBONE`; la tempolimo finas la senfinan
# buklon de la kerno post la raporto.

set -uo pipefail

LIMO="${1:?mankas tempolimo en sekundoj}"
shift

KAJUTO="$(mktemp -d)"
LOG="$KAJUTO/qemu.log"
ENIGO="$KAJUTO/enigo"
mkfifo "$ENIGO"
trap 'rm -rf "$KAJUTO"' EXIT

timeout -k 10 "$LIMO" "$@" < "$ENIGO" > "$LOG" 2>&1 &
qemu_pid=$!

# Malfermi la skriban finon malblokigas la legadon de QEMU; la enigo mem venas
# nur post la prompto.
exec 3> "$ENIGO"

(
    while kill -0 "$qemu_pid" 2>/dev/null; do
        if grep -q 'ligo:~>' "$LOG" 2>/dev/null; then
            printf 'fini\n' >&3
            break
        fi
        sleep 0.2
    done
) &
observanto=$!

wait "$qemu_pid"
stato=$?
exec 3>&-
kill "$observanto" 2>/dev/null || true
wait "$observanto" 2>/dev/null || true

cat "$LOG"

# 0: la guest mem finiĝis (ekz. PSCI SYSTEM_OFF);
# 124/137: `timeout` finis la senfinan buklon de la kerno.
if [[ $stato -ne 0 && $stato -ne 124 && $stato -ne 137 ]]; then
    echo "QEMU finiĝis kun $stato" >&2
    exit 1
fi
if ! grep -Eq '\[kerno\] fina stato: [0-9]+ mortintaj' "$LOG"; then
    echo "La fina stato mankas (ĉu la prompto aperis?)" >&2
    exit 1
fi
if grep -q 'MALBONE' "$LOG"; then
    echo "La kerno raportis MALBONE" >&2
    exit 1
fi
grep -E '\[kerno\] fina stato' "$LOG"
