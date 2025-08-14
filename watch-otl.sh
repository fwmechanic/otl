#!/usr/bin/env bash
# watch-otl.sh -- byte + semantic (offset-free) diffs for .OTL files

die() { printf %s "${@+$@$'\n'}" 1>&2 ; exit 1 ; }
see() ( { set -x; } 2>/dev/null ; "$@" )
have() { command -v "$1" &>/dev/null; }

usage() {
  cat 1>&2 <<USAGE
Usage: $(basename "$0") <file.OTL> [otl-args...]

Examples:
  $(basename "$0") ~/SKPLUS/OUTLINE.OTL                # default: --canon
  $(basename "$0") ~/SKPLUS/OUTLINE.OTL --validate     # add validator
USAGE
  exit 2
}

# --- args ---
[ $# -ge 1 ] || usage
FILE=$1; shift || true
EXTRA_ARGS=("$@")
[[ ${EXTRA_ARGS[0]:-} == "--" ]] && EXTRA_ARGS=("${EXTRA_ARGS[@]:1}")
[ -e "$FILE" ] || die "No such file: $FILE"

# --- deps ---
have inotifywait || die "Missing 'inotifywait' (sudo apt install inotify-tools)"
have cmp         || die "Missing 'cmp'"
have awk         || die "Missing 'awk'"
have diff        || die "Missing 'diff'"

# --- otl binary ---
if have otl; then OTL=otl
elif [ -x ./target/release/otl ]; then OTL=./target/release/otl
else die "Could not find 'otl' (PATH or ./target/release/otl)"; fi

# Default to canonical output (stable for diff)
case " ${EXTRA_ARGS[*]} " in
  *" --canon "*) : ;;
  *) EXTRA_ARGS=(--canon "${EXTRA_ARGS[@]}");;
esac

BASENAME=$(basename "$FILE")
SNAP_BIN=$(mktemp -t ".snap.${BASENAME}.bin.XXXXXX")   || die "mktemp failed"
SNAP_TXT=$(mktemp -t ".snap.${BASENAME}.txt.XXXXXX")   || die "mktemp failed"
trap 'rm -f "$SNAP_BIN" "$SNAP_TXT" "$CURR_BIN" "$CURR_TXT"' EXIT

cp -f -- "$FILE" "$SNAP_BIN" 2>/dev/null || :
# Initialize canonical baseline
CURR_TXT=$(mktemp -t ".curr.${BASENAME}.txt.XXXXXX") || die "mktemp failed"
"$OTL" --canon "$FILE" >"$CURR_TXT" 2>/dev/null || true
mv -f "$CURR_TXT" "$SNAP_TXT" 2>/dev/null || :

echo "Watching: $FILE"
echo "Using otl: $OTL ${EXTRA_ARGS[*]} $FILE"
echo "Baselines: $SNAP_BIN  (bytes),  $SNAP_TXT  (canon)"
echo

# pretty-print cmp -l (octal) as hex with original octal in parens
fmt_cmp_hex='
function o2d(s,  i,d,v){ v=0; for(i=1;i<=length(s);i++){ d=substr(s,i,1); v = v*8 + d } return v }
{ off=$1-1; old=o2d($2); new=o2d($3);
  printf "  0x%06X: 0x%02X (%03s) -> 0x%02X (%03s)\n", off, old, $2, new, $3
}'

run_event() {
  printf "\n=== %s ===\n" "$(date '+%Y-%m-%d %H:%M:%S')"

  # Retry until parse succeeds
  local tries=12 last_rc=1 last_out=""
  while (( tries-- > 0 )); do
    CURR_BIN=$(mktemp -t ".curr.${BASENAME}.bin.XXXXXX") || die "mktemp failed"
    cp -f -- "$FILE" "$CURR_BIN" 2>/dev/null || { rm -f "$CURR_BIN"; sleep 0.05; continue; }

    # Try canonical dump
    CURR_TXT=$(mktemp -t ".curr.${BASENAME}.txt.XXXXXX") || die "mktemp failed"
    last_out=$("$OTL" "${EXTRA_ARGS[@]}" "$CURR_BIN" >"$CURR_TXT" 2>&1); last_rc=$?
    if (( last_rc == 0 )); then
      # Byte deltas once
      local DIFF
      DIFF=$(cmp -l -- "$SNAP_BIN" "$CURR_BIN" 2>/dev/null || true)
      if [ -n "$DIFF" ]; then
        echo "Byte deltas:"
        printf "%s\n" "$DIFF" | awk "$fmt_cmp_hex"
      else
        echo "Byte deltas: (none)"
      fi

      # Semantic diff
      local SEMDIFF
      SEMDIFF=$(diff -u --label "prev(canon)" --label "curr(canon)" "$SNAP_TXT" "$CURR_TXT" || true)
      if [ -n "$SEMDIFF" ]; then
        echo "Semantic diff (canonical):"
        printf "%s\n" "$SEMDIFF" | sed 's/^/  /'
      else
        echo "Semantic diff (canonical): (none)"
      fi

      # Optional: print validator/output if user asked (e.g., --validate)
      # (stdout already captured to CURR_TXT; only stderr warnings remain)
      if [ -n "$last_out" ]; then
        echo "+ $OTL ${EXTRA_ARGS[*]} $FILE"
        printf "%s\n" "$last_out" | sed 's/^/  /'
      fi

      # Promote baselines
      cp -f -- "$CURR_BIN" "$SNAP_BIN" 2>/dev/null || :
      mv -f -- "$CURR_TXT" "$SNAP_TXT" 2>/dev/null || :
      rm -f "$CURR_BIN"
      return 0
    fi

    rm -f "$CURR_BIN" "$CURR_TXT"
    sleep 0.07
  done

  echo "Parse never succeeded; last output:"
  printf "%s\n" "$last_out" | sed 's/^/  /'
}

while :; do
  [ -e "$FILE" ] || {
    printf "Waiting for %s to appear...\n" "$FILE"
    inotifywait -qq -e create -e moved_to -- "$(dirname "$FILE")" || die "inotifywait failed"
    continue
  }
  inotifywait -qq -e close_write -- "$FILE" || { sleep 0.1; continue; }
  run_event
done
