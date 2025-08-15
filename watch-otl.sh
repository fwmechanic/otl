#!/usr/bin/env bash
# watch-otl.sh -- watch one .OTL file OR all .OTL files in a directory.
# - Archives each validated change:
#     <stem>.<YYYYMMDD_HHMMSS_mmm>.OTL          (binary)
#     <stem>.<YYYYMMDD_HHMMSS_mmm>.canon.txt     (raw otl --canon)
# - Shows zero-context unified diffs of canonical output (note bodies omitted).
# - No byte-level diffs.
#
# Env:
#   OTL_ARCHDIR        Override archive dir (default: <dir>/.otl-archive)
#   ALWAYS_ARCHIVE=1   Archive on every write, even if canonical unchanged

die() { printf %s "${@+$@$'\n'}" 1>&2 ; exit 1 ; }
see() ( { set -x; } 2>/dev/null ; "$@" )
have() { command -v "$1" &>/dev/null; }

usage() {
  cat 1>&2 <<USAGE
Usage:
  $(basename "$0") <path-to-.OTL | directory> [otl-args...]

Examples:
  $(basename "$0") ~/SKPLUS/OUTLINE.OTL --validate
  $(basename "$0") ~/SKPLUS --validate

Notes:
  - Requires: inotifywait (inotify-tools), diff, awk.
  - Finds 'otl' in PATH; else uses ./target/release/otl.
  - Archives to: <dir>/.otl-archive/<stem>.<YYYYMMDD_HHMMSS_mmm>.{OTL,canon.txt}
USAGE
  exit 2
}

# -------- args --------
[ $# -ge 1 ] || usage
TARGET=$1; shift || true
EXTRA_ARGS=("$@")
[[ ${EXTRA_ARGS[0]:-} == "--" ]] && EXTRA_ARGS=("${EXTRA_ARGS[@]:1}")

# -------- deps --------
have inotifywait || die "Missing 'inotifywait' (sudo apt install inotify-tools)"
have diff        || die "Missing 'diff'"
have awk         || die "Missing 'awk'"

# -------- otl binary --------
if have otl; then OTL=otl
elif [ -x ./target/release/otl ]; then OTL=./target/release/otl
else die "Could not find 'otl' (PATH or ./target/release/otl)"; fi

# Ensure --canon present (we'll filter note bodies below for diffs)
case " ${EXTRA_ARGS[*]} " in
  *" --canon "*) : ;;
  *) EXTRA_ARGS=(--canon "${EXTRA_ARGS[@]}");;
esac

abspath() { cd "$(dirname "$1")" && pwd -P; }
is_otl() {
  case "$1" in
    *.OTL|*.otl) return 0 ;;
    *) return 1 ;;
  esac
}

# Per-dir state: baseline store
STATE_DIR() { local d; d="$(abspath "$1")"; printf %s "$d/.otl-watch"; }
ARCH_DIR()  {
  local d; d="$(abspath "$1")"
  printf %s "${OTL_ARCHDIR:-$d/.otl-archive}"
}

# Produce canonical text with note bodies omitted (keeps "noteLen=", "note", "/note")
filter_canon() {
  awk '
    BEGIN { in_note=0 }
    /^note$/      { in_note=1; print "note"; print "[note body omitted]"; next }
    /^\/note$/    { in_note=0; print "/note"; next }
    in_note == 1  { next }
    { print }
  ' "$1"
}

# Build/refresh baseline for a file (if missing)
ensure_baseline() {
  local file="$1"
  local sdir; sdir="$(STATE_DIR "$file")"
  mkdir -p "$sdir" 2>/dev/null || :
  local base="$sdir/$(basename "$file").canon.txt"
  [ -f "$base" ] && return 0

  local tmp; tmp="$(mktemp -t ".canon.$(basename "$file").XXXXXX")" || return 0
  "$OTL" --canon "$file" >"$tmp" 2>/dev/null || { rm -f "$tmp"; return 0; }
  filter_canon "$tmp" >"$base"
  rm -f "$tmp"
}

# Handle one write/rename event for a file
handle_file_event() {
  local file="$1"
  is_otl "$file" || return 0
  [ -f "$file" ] || return 0

  ensure_baseline "$file"
  local sdir base; sdir="$(STATE_DIR "$file")"; base="$sdir/$(basename "$file").canon.txt"

  # Retry parse on partial writes
  local tries=12 last_rc=1 canon_raw canon_filt warn tmpbin
  while (( tries-- > 0 )); do
    tmpbin="$(mktemp -t ".curr.$(basename "$file").bin.XXXXXX")" || return 0
    cp -f -- "$file" "$tmpbin" 2>/dev/null || { rm -f "$tmpbin"; sleep 0.05; continue; }

    canon_raw="$(mktemp -t ".canon.$(basename "$file").raw.XXXXXX")" || { rm -f "$tmpbin"; return 0; }
    warn="$(mktemp -t ".otl.warn.XXXXXX")" || { rm -f "$tmpbin" "$canon_raw"; return 0; }
    "$OTL" "${EXTRA_ARGS[@]}" "$tmpbin" >"$canon_raw" 2>"$warn"
    last_rc=$?
    if (( last_rc == 0 )); then
      canon_filt="$(mktemp -t ".canon.$(basename "$file").filt.XXXXXX")" || { rm -f "$tmpbin" "$canon_raw" "$warn"; return 0; }
      filter_canon "$canon_raw" >"$canon_filt"

      local lbl_prev="prev($(basename "$file"))"
      local lbl_curr="curr($(basename "$file"))"
      local diffout
      if [ -s "$base" ]; then
        diffout=$(diff -u -U0 --label "$lbl_prev" --label "$lbl_curr" "$base" "$canon_filt" || true)
      else
        diffout=$(diff -u -U0 --label "$lbl_prev" --label "$lbl_curr" /dev/null "$canon_filt" || true)
      fi

      printf "\n=== %s -- %s ===\n" "$(date '+%Y-%m-%d %H:%M:%S')" "$file"
      if [ -n "$diffout" ]; then
        echo "Canonical diff (no context; note bodies omitted):"
        printf "%s\n" "$diffout" | sed 's/^/  /'

        # Archive binary + raw canonical with the SAME timestamp
        local arch; arch="$(ARCH_DIR "$file")"; mkdir -p "$arch" 2>/dev/null || :
        local stem ext ts bin_out canon_out
        stem="$(basename "$file")"; ext="${stem##*.}"; stem="${stem%.*}"
        ts=$(date +%Y%m%d_%H%M%S_%3N)
        bin_out="$arch/${stem}.${ts}.${ext}"
        canon_out="$arch/${stem}.${ts}.canon.txt"
        cp -f -- "$tmpbin" "$bin_out"   && echo "Archived: $bin_out"
        cp -f -- "$canon_raw" "$canon_out" && echo "Archived: $canon_out"
      else
        echo "Canonical diff: (none)"
        if [[ -n "${ALWAYS_ARCHIVE:-}" ]]; then
          local arch; arch="$(ARCH_DIR "$file")"; mkdir -p "$arch" 2>/dev/null || :
          local stem ext ts bin_out canon_out
          stem="$(basename "$file")"; ext="${stem##*.}"; stem="${stem%.*}"
          ts=$(date +%Y%m%d_%H%M%S_%3N)
          bin_out="$arch/${stem}.${ts}.${ext}"
          canon_out="$arch/${stem}.${ts}.canon.txt"
          cp -f -- "$tmpbin" "$bin_out"   && echo "Archived (ALWAYS): $bin_out"
          cp -f -- "$canon_raw" "$canon_out" && echo "Archived (ALWAYS): $canon_out"
        fi
      fi

      # Validator warnings (if user supplied --validate)
      if [ -s "$warn" ]; then
        echo "Validator:"
        sed 's/^/  /' "$warn"
      fi

      # Promote baseline
      mv -f -- "$canon_filt" "$base" 2>/dev/null || :
      rm -f "$tmpbin" "$canon_raw" "$warn"
      return 0
    fi
    rm -f "$tmpbin" "$canon_raw" "$warn"
    sleep 0.07
  done

  printf "\n=== %s -- %s ===\n" "$(date '+%Y-%m-%d %H:%M:%S')" "$file"
  echo "Parse never succeeded (skipping archive)."
  return 1
}

# -------- main --------
if [ -d "$TARGET" ]; then
  DIR="$(cd "$TARGET" && pwd -P)"
  # Baselines for existing *.OTL files
  shopt -s nullglob
  for f in "$DIR"/*.OTL "$DIR"/*.otl; do ensure_baseline "$f"; done
  shopt -u nullglob

  echo "Watching directory: $DIR"
  echo "Using otl: $OTL ${EXTRA_ARGS[*]} (applied per file)"
  echo "Archive base: $(ARCH_DIR "$DIR")"
  echo

  inotifywait -m -q -e close_write -e moved_to --format '%e %w%f' -- "$DIR" \
  | while read -r ev path; do
      is_otl "$path" || continue
      handle_file_event "$path"
    done

elif [ -f "$TARGET" ]; then
  FILE="$(cd "$(dirname "$TARGET")" && pwd -P)/$(basename "$TARGET")"
  ensure_baseline "$FILE"
  echo "Watching file: $FILE"
  echo "Using otl: $OTL ${EXTRA_ARGS[*]}"
  echo "Archive base: $(ARCH_DIR "$FILE")"
  echo

  inotifywait -m -q -e close_write --format '%w%f' -- "$FILE" \
  | while read -r path; do
      handle_file_event "$path"
    done
else
  die "Target is neither a file nor a directory: $TARGET"
fi
