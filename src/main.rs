use serde::Serialize;
use std::env;
use std::fs;
use std::io::{self, Read};

/// Attribute bits we (currently) know
const A_NOTE: u8 = 0x80; // has note bytes (then u16 noteLen + bytes)
const A_CURSOR: u8 = 0x20; // caret on this heading (displayed only with --show-cursor)
const A_SIBFOLLOWS: u8 = 0x08; // there exists a later sibling at same level
const A_HASKIDS: u8 = 0x04; // semantics under study; shown as k/K; validation optional

#[derive(Debug, Clone)]
struct Rec {
    text: String,
    delta: i16,      // relative level change (i16 LE)
    attr: u8,        // raw attribute flags
    marker_u16: u16, // raw marker word (FFFF/-1 expanded, FFFE/-2 collapsed)
    collapsed: bool, // convenience (marker == FFFE)
    note: Option<String>,
    flags: Flags,

    // Byte offsets (for --offsets)
    off_text: usize,       // first heading byte
    len_text: usize,       // heading byte count (before 0xFF terminator)
    off_terminator: usize, // 0xFF that ends heading text
    off_attr: usize,       // attr byte
    off_marker: usize,     // first of the 2 marker bytes
    off_delta: usize,      // first of the 2 delta bytes
    off_note_len: Option<usize>,
    off_note: Option<usize>,

    // Raw note length from file (u16), independent of --enc decoding (0 if no note)
    note_len: usize,
}

#[derive(Debug, Clone, Serialize)]
struct Flags {
    has_note: bool,         // attr & 0x80
    selected: bool,         // attr & 0x20
    has_next_sibling: bool, // attr & 0x08
    has_child: bool,        // attr & 0x04 (shown only)
}

#[derive(Debug, Clone, Serialize)]
struct Node {
    text: String,
    note: Option<String>,
    collapsed: bool,
    flags: Flags,
    #[serde(skip)]
    synthetic: bool, // true for root / filler nodes
    children: Vec<Node>,
}

const MAGIC: [u8; 3] = [0x1a, 0x93, 0x1a];
const PREAMBLE: [u8; 6] = [0xff, 0x00, 0xff, 0xff, 0xff, 0xff];
const M_EXPANDED: u8 = 0xff;
const M_COLLAPSED: u8 = 0xfe;

// Guardrails (format is 16-bit; these just prevent runaway reads)
const MAX_TEXTLEN: usize = 1 << 20; // 1 MiB heading (paranoid limit)
const MAX_NOTELEN: usize = 0xFFFF; // format max (u16)

fn usage(prog: &str) -> ! {
    eprintln!(
        "Usage: {prog} <file | -> \
         [--json] [--dump] [--offsets] [--validate] \
         [--enc utf8|latin1|ascii] [--text] [--canon] \
         [--show-cursor] [--assume-child-bit] \
         [--diff <prev> <curr>]"
    );
    std::process::exit(2);
}

fn decode_note(bytes: &[u8], enc: &str) -> String {
    match enc {
        "utf8" => String::from_utf8_lossy(bytes).to_string(),
        "latin1" => bytes.iter().map(|&b| b as char).collect::<String>(),
        "ascii" => bytes
            .iter()
            .map(|&b| (b & 0x7f) as char)
            .collect::<String>(),
        _ => String::from_utf8_lossy(bytes).to_string(),
    }
}

// Decode heading bytes: char = b & 0x7F; if high bit set, append a space.
fn decode_heading(bytes: &[u8]) -> String {
    let mut s = String::new();
    for &b in bytes {
        s.push((b & 0x7f) as char);
        if (b & 0x80) != 0 {
            s.push(' ');
        }
    }
    s
}

fn parse_otl(buf: &[u8], note_enc: &str) -> io::Result<Vec<Rec>> {
    let mut i = 0usize;
    let mut out = Vec::<Rec>::new();

    if buf.len() >= 3 && buf[0..3] == MAGIC {
        i += 3;
    }
    if buf.len() >= i + 6 && buf[i..i + 6] == PREAMBLE {
        i += 6;
    }

    while i < buf.len() {
        // explicit EOF sentinels
        if i == buf.len() - 1 && buf[i] == 0x1a {
            break;
        }
        if i + 2 < buf.len() && buf[i] == 0xff && buf[i + 1] == 0xff && buf[i + 2] == 0x1a {
            break;
        }

        // Find next 0xFF; heading text may be zero-length.
        let mut k = i;
        while k < buf.len() && buf[k] != 0xff {
            k += 1;
        }
        if k >= buf.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unterminated heading text",
            ));
        }

        // Must have at least 4 bytes after the terminator for attr+marker+delta.
        if k + 4 >= buf.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "truncated record header",
            ));
        }
        let attr = buf[k + 1];
        let mark1 = buf[k + 2];
        let mark2 = buf[k + 3];

        // Stray 0xFF? (marker must be FE/FF followed by FF). If not, skip this 0xFF and keep searching.
        if mark2 != 0xff || (mark1 != M_EXPANDED && mark1 != M_COLLAPSED) {
            i = k + 1;
            continue;
        }

        // Valid record
        let text_bytes = &buf[i..k];
        let text = decode_heading(text_bytes);
        let marker_u16 = u16::from_le_bytes([mark1, mark2]);
        let collapsed = marker_u16 == 0xFFFE;
        let delta = i16::from_le_bytes([buf[k + 4], buf[k + 5]]);

        let off_text = i;
        let len_text = k - i;
        if len_text > MAX_TEXTLEN {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "heading too large",
            ));
        }
        let off_terminator = k;
        let off_attr = k + 1;
        let off_marker = k + 2;
        let off_delta = k + 4;

        i = k + 6;

        // Optional note
        let mut note: Option<String> = None;
        let mut off_note_len: Option<usize> = None;
        let mut off_note: Option<usize> = None;
        let mut note_len: usize = 0;

        if (attr & A_NOTE) != 0 {
            if i + 2 > buf.len() {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "truncated note length",
                ));
            }
            off_note_len = Some(i);
            let nlen = u16::from_le_bytes([buf[i], buf[i + 1]]) as usize;
            if nlen > MAX_NOTELEN {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "note too large for u16 length",
                ));
            }
            i += 2;
            if i + nlen > buf.len() {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "truncated note bytes",
                ));
            }
            off_note = Some(i);
            note_len = nlen;
            note = Some(decode_note(&buf[i..i + nlen], note_enc));
            i += nlen;
        }

        let flags = Flags {
            has_note: (attr & A_NOTE) != 0,
            selected: (attr & A_CURSOR) != 0,
            has_next_sibling: (attr & A_SIBFOLLOWS) != 0,
            has_child: (attr & A_HASKIDS) != 0, // shown, not validated by default
        };

        out.push(Rec {
            text,
            delta,
            attr,
            marker_u16,
            collapsed,
            note,
            flags,
            off_text,
            len_text,
            off_terminator,
            off_attr,
            off_marker,
            off_delta,
            off_note_len,
            off_note,
            note_len,
        });
    }

    Ok(out)
}

fn build_tree(recs: &[Rec]) -> Vec<Node> {
    let mut root = Node {
        text: String::new(),
        note: None,
        collapsed: false,
        flags: Flags {
            has_note: false,
            selected: false,
            has_next_sibling: false,
            has_child: false,
        },
        synthetic: true,
        children: Vec::new(),
    };

    let mut path: Vec<usize> = Vec::new(); // indexes from root to current parent/node
    let mut level: i32 = 0;

    for r in recs {
        level += r.delta as i32;
        if level < 0 {
            level = 0;
        }

        // shrink to target level
        while (path.len() as i32) > level {
            path.pop();
        }
        // if we jumped more than +1, create dummy intermediates
        while (path.len() as i32) < level {
            let dummy = Node {
                text: String::new(),
                note: None,
                collapsed: false,
                flags: Flags {
                    has_note: false,
                    selected: false,
                    has_next_sibling: false,
                    has_child: false,
                },
                synthetic: true,
                children: Vec::new(),
            };
            push_child(&mut root, &mut path, dummy);
        }

        let node = Node {
            text: r.text.clone(),
            note: r.note.clone(),
            collapsed: r.collapsed,
            flags: r.flags.clone(),
            synthetic: false,
            children: Vec::new(),
        };
        push_child(&mut root, &mut path, node);
    }

    root.children
}

fn push_child(root: &mut Node, path: &mut Vec<usize>, child: Node) {
    // Walk the path safely to get a mutable reference to the parent.
    let mut parent: &mut Node = root;
    for &idx in path.iter() {
        parent = parent
            .children
            .get_mut(idx)
            .expect("path index out of bounds while building tree");
    }
    parent.children.push(child);
    let new_idx = parent.children.len() - 1;
    path.push(new_idx); // make the new node current
}

fn render_plain_all(nodes: &[Node], depth: usize) -> String {
    let mut out = String::new();
    for n in nodes {
        if n.synthetic {
            out.push_str(&render_plain_all(&n.children, depth));
            continue;
        }
        let indent = " ".repeat(depth * 2);
        out.push_str(&format!("{indent}{}\n", n.text));

        if let Some(note) = &n.note {
            let note_indent = " ".repeat((depth + 1) * 2);
            for line in note.replace("\r\n", "\n").lines() {
                out.push_str(&format!("{note_indent}{}\n", line));
            }
        }
        // Always descend (ignore collapsed)
        out.push_str(&render_plain_all(&n.children, depth + 1));
    }
    out
}

fn render_indented(nodes: &[Node], prefix: &str) -> String {
    let mut out = String::new();
    for n in nodes {
        if n.synthetic {
            out.push_str(&render_indented(&n.children, prefix));
            continue;
        }
        let fold = if n.collapsed { "[+]" } else { "[-]" };
        let sel = if n.flags.selected { "*" } else { " " };
        out.push_str(&format!("{prefix}{fold}{sel} {}\n", n.text));

        if let Some(note) = &n.note {
            for line in note.replace("\r\n", "\n").lines() {
                out.push_str(&format!("{prefix}    > {}\n", line));
            }
        }
        if !n.collapsed && !n.children.is_empty() {
            out.push_str(&render_indented(
                &n.children,
                &(prefix.to_string() + "    "),
            ));
        }
    }
    out
}

fn dump_recs(recs: &[Rec]) -> String {
    let mut lvl: i32 = 0;
    let mut s = String::new();
    for (idx, r) in recs.iter().enumerate() {
        lvl += r.delta as i32;
        let c = if r.collapsed { 'C' } else { 'E' };
        let sel = if r.flags.selected { 'S' } else { ' ' };
        let nxt = if r.flags.has_next_sibling { 'S' } else { ' ' }; // 'S' to hint "sib follows"
        let nlen = r.note_len;
        s.push_str(&format!(
            "{:>4}  L={:>3}  d={:>3}  attr=0x{:02x}  {} {} {}  note={:>5}  {}\n",
            idx, lvl, r.delta, r.attr, c, sel, nxt, nlen, r.text
        ));
    }
    s
}

fn dump_offsets(recs: &[Rec]) -> String {
    let mut s = String::new();
    for (idx, r) in recs.iter().enumerate() {
        let m = if r.collapsed { "FE FF" } else { "FF FF" };
        s.push_str(&format!(
            "#{:03} text[{:#06x}+{:>4}] 0xFF[{:#06x}] attr[{:#06x}=0x{:02x}] \
mark[{:#06x}={:<5}] delta[{:#06x}]{}{}  {}\n",
            idx,
            r.off_text,
            r.len_text,
            r.off_terminator,
            r.off_attr,
            r.attr,
            r.off_marker,
            m,
            r.off_delta,
            match r.off_note_len {
                Some(o) => format!(" nlen[{:#06x}]", o),
                None => "".into(),
            },
            match r.off_note {
                Some(o) => format!(" note[{:#06x}]", o),
                None => "".into(),
            },
            r.text
        ));
    }
    s
}

// Escape just backslash and quote for compact one-line headline printing
fn escape_headline(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(ch),
        }
    }
    out
}

// Format attr bits as a compact string (bits 7..0):
// Known bits -> letters (lowercase=0, UPPER=1):
//   0x80 -> n/N (note present)
//   0x20 -> c/C (cursor/selected) -- printed only if show_cursor=true
//   0x08 -> s/S (sibling follows later at this level)
//   0x04 -> k/K (bit present; semantics under study)
// Unknown bits: print '1' when set; print nothing when clear.
fn fmt_attr_bits(attr: u8, show_cursor: bool) -> String {
    let mut s = String::new();
    for i in (0..8).rev() {
        let mask = 1u8 << i;
        let set = (attr & mask) != 0;
        let ch = match mask {
            x if x == A_NOTE => {
                if set {
                    'N'
                } else {
                    'n'
                }
            }
            x if x == A_CURSOR => {
                if !show_cursor {
                    '\0'
                } else if set {
                    'C'
                } else {
                    'c'
                }
            }
            x if x == A_SIBFOLLOWS => {
                if set {
                    'S'
                } else {
                    's'
                }
            }
            x if x == A_HASKIDS => {
                if set {
                    'K'
                } else {
                    'k'
                }
            }
            _ => {
                if set {
                    '1'
                } else {
                    '\0'
                }
            }
        };
        if ch != '\0' {
            s.push(ch);
        }
    }
    s
}

fn mark_field(u: u16) -> String {
    let s = u as i16;
    match s {
        -1 => "-1:+".to_string(),
        -2 => "-2:-".to_string(),
        _ => format!("0x{:04x}", u),
    }
}

// Print signed single-digit deltas with a sign ("+0","-1","+7");
// fall back to raw 16-bit hex for anything outside -9..9.
fn delta_field(d: i16) -> String {
    if (-9..=9).contains(&d) {
        format!("{:+}", d) // "+0", "+1", "-1", ... (keeps columns aligned)
    } else {
        format!("0x{:04x}", d as u16)
    }
}

/// Offset-free, insertion-stable, bit-complete dump:
/// <attrbits> mark=-1:+|-2:-|0xnnnn delta=+1|0|-1|0xnnnn textLen=%04x "text"
/// If a note exists, emit on following lines:
///   noteLen=%04x
///   note
///   <note text...>   (CRLF normalized to LF)
///   /note
fn render_canon(recs: &[Rec], show_cursor: bool) -> String {
    let mut out = String::new();
    for r in recs {
        let mark = mark_field(r.marker_u16);
        let delta_disp = delta_field(r.delta);
        let text_len_raw: u16 = r.len_text as u16;

        out.push_str(&format!(
            "{} mark={} delta={} textLen={:04x} \"{}\"\n",
            fmt_attr_bits(r.attr, show_cursor),
            mark,
            delta_disp,
            text_len_raw,
            escape_headline(&r.text)
        ));

        if r.flags.has_note {
            out.push_str(&format!("noteLen={:04x}\n", r.note_len as u16));
            out.push_str("note\n");
            let note_norm = r.note.as_deref().unwrap_or("").replace("\r\n", "\n");
            out.push_str(&note_norm);
            if !note_norm.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("/note\n");
        }
    }
    out
}

// Encode helpers to write .OTL from a Node tree
#[cfg(test)]
fn encode_heading_from_text(text: &str) -> Vec<u8> {
    // Best-effort 7-bit mapping; non-ASCII becomes '?'. We do not use the high-bit space encoding.
    let mut v = Vec::with_capacity(text.len());
    for ch in text.chars() {
        let b = if (ch as u32) < 0x80 { ch as u8 } else { b'?' } & 0x7f;
        v.push(b);
    }
    v
}

#[cfg(test)]
fn encode_note_bytes(note: &str, enc: &str) -> Vec<u8> {
    match enc {
        "utf8" => note.as_bytes().to_vec(),
        "latin1" => note
            .chars()
            .map(|c| if (c as u32) <= 0xFF { c as u8 } else { b'?' })
            .collect(),
        "ascii" => note.bytes().map(|b| b & 0x7f).collect(),
        _ => note.as_bytes().to_vec(),
    }
}

#[cfg(test)]
fn serialize_tree_to_otl(nodes: &[Node], note_enc: &str) -> Vec<u8> {
    #[derive(Clone)]
    struct Flat {
        level: usize,
        attr: u8,
        marker_first: u8,
        text: Vec<u8>,
        note: Option<Vec<u8>>,
    }

    fn walk(nodes: &[Node], level: usize, out: &mut Vec<Flat>, note_enc: &str) {
        for (idx, n) in nodes.iter().enumerate() {
            if n.synthetic {
                // skip synthetic, descend
                walk(&n.children, level, out, note_enc);
                continue;
            }
            let mut attr: u8 = 0;
            if n.note.is_some() {
                attr |= A_NOTE;
            }
            if n.flags.selected {
                attr |= A_CURSOR;
            }
            if idx + 1 < nodes.len() {
                attr |= A_SIBFOLLOWS;
            }
            // Intentionally do not set A_HASKIDS; semantics under study
            let marker_first = if n.collapsed { M_COLLAPSED } else { M_EXPANDED };
            let text = encode_heading_from_text(&n.text);
            let note = n.note.as_ref().map(|s| encode_note_bytes(s, note_enc));
            out.push(Flat {
                level,
                attr,
                marker_first,
                text,
                note,
            });
            // descend
            walk(&n.children, level + 1, out, note_enc);
        }
    }

    let mut flats = Vec::<Flat>::new();
    walk(nodes, 0, &mut flats, note_enc);

    let mut buf = Vec::<u8>::new();
    buf.extend(MAGIC);
    buf.extend(PREAMBLE);

    let mut prev_level: isize = 0;
    for f in flats {
        buf.extend(&f.text);
        buf.push(0xFF);
        buf.push(f.attr);
        buf.push(f.marker_first);
        buf.push(0xFF);
        let delta: i16 = (f.level as isize - prev_level) as i16;
        buf.extend_from_slice(&delta.to_le_bytes());
        if let Some(note_bytes) = f.note {
            let nlen = u16::try_from(note_bytes.len()).unwrap_or(u16::MAX);
            buf.extend_from_slice(&nlen.to_le_bytes());
            buf.extend(&note_bytes[..usize::from(nlen)]);
        }
        prev_level = f.level as isize;
    }

    buf.push(0x1a); // EOF sentinel many files include; safe to add
    buf
}

/// Validate derived invariants and print warnings to stderr.
/// By default we only assert bits we're confident in (0x08 sibling follows).
/// Use `assume_child_bit=true` to test the hypothesis that 0x04 == "has child".
fn validate(recs: &[Rec], assume_child_bit: bool) {
    // compute levels
    let mut levels = Vec::with_capacity(recs.len());
    let mut lvl = 0i32;
    for r in recs {
        lvl += r.delta as i32;
        if lvl < 0 {
            lvl = 0;
        }
        levels.push(lvl);
    }

    for i in 0..recs.len() {
        let my = levels[i];

        // 0x08 sibling-follows check -- solid
        let mut has_later_sibling = false;
        for &level in levels.iter().skip(i + 1) {
            if level < my {
                break;
            }
            if level == my {
                has_later_sibling = true;
                break;
            }
        }
        let bit_sib = (recs[i].attr & A_SIBFOLLOWS) != 0;
        if has_later_sibling != bit_sib {
            eprintln!(
                "WARN: rec #{:03} sibling bit mismatch (attr={}, expected={}) at attr[{:#06x}]",
                i, bit_sib, has_later_sibling, recs[i].off_attr
            );
        }

        // Optional hypothesis check for 0x04
        if assume_child_bit {
            let has_child_struct = i + 1 < recs.len() && levels[i + 1] > my;
            let bit_child = (recs[i].attr & A_HASKIDS) != 0;
            if has_child_struct != bit_child {
                eprintln!(
                    "WARN: rec #{:03} 0x04!=has_child (attr={}, expected={}) at attr[{:#06x}]",
                    i, bit_child, has_child_struct, recs[i].off_attr
                );
            }
        }

        // Unknown bits: exclude 0x80, 0x20, 0x08, 0x04 always (we show 0x04 but don't warn by default)
        let known = A_NOTE | A_CURSOR | A_SIBFOLLOWS | A_HASKIDS;
        let unknown = recs[i].attr & !known;
        if unknown != 0 {
            eprintln!(
                "WARN: rec #{:03} unknown attr bits set: 0x{:02x} at attr[{:#06x}]",
                i, unknown, recs[i].off_attr
            );
        }
    }
}

/**************
 * Tests
 **************/
#[cfg(test)]
mod tests {
    use super::*;

    fn le_u16(n: u16) -> [u8; 2] {
        n.to_le_bytes()
    }
    fn le_i16(n: i16) -> [u8; 2] {
        n.to_le_bytes()
    }

    // Build a single record. Marker first byte is 0xFF (expanded) or 0xFE (collapsed).
    fn rec_bytes(
        text: &str,
        attr: u8,
        marker_first: u8,
        delta: i16,
        note: Option<&[u8]>,
    ) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend(text.as_bytes());
        v.push(0xFF); // text terminator
        v.push(attr);
        v.push(marker_first);
        v.push(0xFF);
        v.extend(le_i16(delta));
        if (attr & A_NOTE) != 0 {
            let nb = note.unwrap_or(&[]);
            v.extend(le_u16(nb.len() as u16));
            v.extend(nb);
        }
        v
    }

    // Build a minimal .OTL with MAGIC + PREAMBLE and provided records.
    fn otl_file(records: Vec<Vec<u8>>) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend(MAGIC);
        v.extend(PREAMBLE);
        for r in records {
            v.extend(r);
        }
        v
    }

    #[test]
    fn parse_and_build_tree_basic() {
        // Parent (level 0), then Child1 with a note (level +1), then Child2 (sibling at same level)
        let parent = rec_bytes("Parent", 0x00, M_EXPANDED, 0, None);
        let note_text = b"Line1\r\nLine2"; // CRLF normalized later
        let child1 = rec_bytes("Child1", A_NOTE, M_EXPANDED, 1, Some(note_text));
        let child2 = rec_bytes("Child2", 0x00, M_EXPANDED, 0, None);
        let buf = otl_file(vec![parent, child1, child2]);

        let recs = parse_otl(&buf, "latin1").expect("parse otl");
        assert_eq!(recs.len(), 3);
        assert!(recs[1].flags.has_note);
        assert_eq!(recs[1].note.as_deref().unwrap(), "Line1\r\nLine2");

        let tree = build_tree(&recs);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].text, "Parent");
        assert_eq!(tree[0].children.len(), 2);
        assert_eq!(tree[0].children[0].text, "Child1");
        assert_eq!(tree[0].children[1].text, "Child2");

        // Plain render normalizes CRLF to LF inside notes
        let plain = render_plain_all(&tree, 0);
        assert!(plain.contains("Child1"));
        assert!(plain.contains("Line1"));
        assert!(plain.contains("Line2"));

        // Canon includes noteLen and note section
        let canon = render_canon(&recs, false);
        assert!(canon.contains("noteLen="));
        assert!(canon.contains("note\nLine1\nLine2\n/note"));
    }

    #[test]
    fn roundtrip_tree_to_otl_and_back() {
        // Build initial bytes via record helpers
        let parent = rec_bytes("Parent", 0x00, M_EXPANDED, 0, None);
        let child1 = rec_bytes("Child1", A_NOTE, M_EXPANDED, 1, Some(b"Line1\r\nLine2"));
        let child2 = rec_bytes("Child2", 0x00, M_EXPANDED, 0, None);
        let buf = otl_file(vec![parent, child1, child2]);

        // Parse and build tree
        let recs = parse_otl(&buf, "latin1").expect("parse otl");
        let tree = build_tree(&recs);

        // Serialize tree back to .OTL and parse again
        let buf2 = serialize_tree_to_otl(&tree, "latin1");
        let recs2 = parse_otl(&buf2, "latin1").expect("re-parse otl");
        let tree2 = build_tree(&recs2);

        // Compare using plain text rendering (includes notes, normalized)
        let plain1 = render_plain_all(&tree, 0);
        let plain2 = render_plain_all(&tree2, 0);
        assert_eq!(plain1, plain2);
    }

    // Generates a sample .OTL from a small tree and writes it to a temp dir.
    // Run manually: cargo test generate_sample_tree_otl -- --ignored --nocapture
    #[test]
    #[ignore]
    fn generate_sample_tree_otl() {
        use std::path::PathBuf;
        // Create a tree by parsing some bytes, then re-serialize via our serializer
        let a = rec_bytes("Demo", 0x00, M_EXPANDED, 0, None);
        let intro = rec_bytes(
            "Intro",
            A_NOTE | A_SIBFOLLOWS,
            M_EXPANDED,
            0,
            Some(b"Created by tests\r\nEnjoy!"),
        );
        let tasks = rec_bytes("Tasks", 0x00, M_EXPANDED, 1, None);
        let item1 = rec_bytes("Item1", 0x00, M_EXPANDED, 1, None);
        let item2 = rec_bytes("Item2", 0x00, M_EXPANDED, 0, None);
        let buf = otl_file(vec![a, intro, tasks, item1, item2]);
        let recs = parse_otl(&buf, "latin1").expect("parse otl");
        let tree = build_tree(&recs);

        let out_bytes = serialize_tree_to_otl(&tree, "latin1");

        // Use target tmpdir for test outputs
        let mut out =
            PathBuf::from(std::env::var("CARGO_TARGET_TMPDIR").unwrap_or_else(|_| "target".into()));
        out.push("roundtrip-demo.SKP.OTL");
        std::fs::create_dir_all(out.parent().unwrap()).ok();
        std::fs::write(&out, &out_bytes).expect("write sample .OTL");
        println!("Wrote sample .OTL to {}", out.display());
    }
    #[test]
    fn tree_with_level_jumps_and_neg_deltas() {
        // A (level 0)
        //   <filler>
        //     B (jump +2 creates filler under A)
        //   C (delta -1 brings us back to level 1 under A)
        //   D (sibling of C at same level)
        let a = rec_bytes("A", 0x00, M_EXPANDED, 0, None);
        let b = rec_bytes("B", 0x00, M_EXPANDED, 2, None);
        let c = rec_bytes("C", 0x00, M_EXPANDED, -1, None);
        let d = rec_bytes("D", 0x00, M_EXPANDED, 0, None);
        let buf = otl_file(vec![a, b, c, d]);

        let recs = parse_otl(&buf, "latin1").expect("parse otl");
        let tree = build_tree(&recs);

        assert_eq!(tree.len(), 1);
        let a = &tree[0];
        assert_eq!(a.text, "A");
        assert_eq!(a.children.len(), 3, "A should have filler, C, D");

        let filler = &a.children[0];
        assert!(filler.synthetic, "first child is a synthetic filler node");
        assert_eq!(filler.children.len(), 1);
        assert_eq!(filler.children[0].text, "B");

        assert_eq!(a.children[1].text, "C");
        assert_eq!(a.children[2].text, "D");
    }

    #[test]
    fn canon_golden_minimal() {
        // Two records: A (no note), B (with CRLF note). Both expanded (-1:+).
        let a = rec_bytes("A", 0x00, M_EXPANDED, 0, None);
        let note = b"hello\r\nworld"; // length 12
        let b = rec_bytes("B", A_NOTE, M_EXPANDED, 1, Some(note));
        let buf = otl_file(vec![a, b]);

        let recs = parse_otl(&buf, "latin1").expect("parse otl");
        let canon = render_canon(&recs, false);

        let expected = concat!(
            "nsk mark=-1:+ delta=+0 textLen=0001 \"A\"\n",
            "Nsk mark=-1:+ delta=+1 textLen=0001 \"B\"\n",
            "noteLen=000c\n",
            "note\n",
            "hello\nworld\n",
            "/note\n",
        );
        assert_eq!(canon, expected);
    }

    // Round-trip real files from a directory you specify via env var.
    // Usage:
    //   OTL_SRC_RO_DIR=/path/to/your/otl cargo test roundtrip_real_dir -- --ignored --nocapture
    #[test]
    #[ignore]
    fn roundtrip_real_dir() {
        use std::path::{Path, PathBuf};

        fn ends_with_otl(p: &Path) -> bool {
            match p.extension().and_then(|e| e.to_str()) {
                Some(ext) => ext.eq_ignore_ascii_case("otl"),
                None => false,
            }
        }

        fn collect_files(root: &Path, out: &mut Vec<PathBuf>) {
            if let Ok(rd) = std::fs::read_dir(root) {
                for entry in rd.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        collect_files(&path, out);
                    } else if ends_with_otl(&path) {
                        out.push(path);
                    }
                }
            }
        }

        let dir = std::env::var("OTL_SRC_RO_DIR").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            format!("{}/SKPLUS", home)
        });
        let root = Path::new(&dir);
        assert!(root.is_dir(), "not a directory: {}", root.display());

        let mut files = Vec::<PathBuf>::new();
        collect_files(root, &mut files);
        assert!(
            !files.is_empty(),
            "no .OTL files found under {}",
            root.display()
        );

        // Output directory for round-tripped files
        let outdir = PathBuf::from(
            std::env::var("CARGO_TARGET_TMPDIR").unwrap_or_else(|_| "target/otl-roundtrip".into()),
        );
        let _ = std::fs::create_dir_all(&outdir);

        let mut mismatches = Vec::<String>::new();
        println!("Found {} .OTL files; round-tripping...", files.len());

        for path in files {
            let fname = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("file.OTL");
            let pretty = path.display().to_string();
            match std::fs::read(&path) {
                Ok(buf) => match parse_otl(&buf, "latin1") {
                    Ok(recs) => {
                        let tree = build_tree(&recs);
                        let buf2 = serialize_tree_to_otl(&tree, "latin1");
                        match parse_otl(&buf2, "latin1") {
                            Ok(recs2) => {
                                let tree2 = build_tree(&recs2);
                                let a = render_plain_all(&tree, 0);
                                let b = render_plain_all(&tree2, 0);
                                let mut outp = outdir.clone();
                                outp.push(format!("{}.rt.OTL", fname));
                                let _ = std::fs::write(&outp, &buf2);
                                if a != b {
                                    mismatches.push(pretty.clone());
                                    eprintln!("Mismatch: {} -> {}", pretty, outp.display());
                                } else {
                                    println!("OK: {}", pretty);
                                }
                            }
                            Err(e) => {
                                mismatches.push(pretty.clone());
                                eprintln!("Re-parse failed for {}: {}", pretty, e);
                            }
                        }
                    }
                    Err(e) => {
                        mismatches.push(pretty.clone());
                        eprintln!("Parse failed for {}: {}", pretty, e);
                    }
                },
                Err(e) => {
                    mismatches.push(pretty.clone());
                    eprintln!("Read failed for {}: {}", pretty, e);
                }
            }
        }

        if !mismatches.is_empty() {
            panic!(
                "Round-trip mismatches in {} files (see stderr)",
                mismatches.len()
            );
        }
    }
}

/**************
 * --diff mode
 **************/
fn render_mark_for_diff(u: u16) -> String {
    mark_field(u)
}
fn render_delta_for_diff(d: i16) -> String {
    delta_field(d)
}

fn diff_two_recs(prev: &Rec, curr: &Rec, show_cursor: bool) -> Vec<String> {
    let mut changes = Vec::new();
    if prev.attr != curr.attr {
        changes.push(format!(
            "  attr: {} -> {}",
            fmt_attr_bits(prev.attr, show_cursor),
            fmt_attr_bits(curr.attr, show_cursor)
        ));
    }
    if prev.marker_u16 != curr.marker_u16 {
        changes.push(format!(
            "  mark: {} -> {}",
            render_mark_for_diff(prev.marker_u16),
            render_mark_for_diff(curr.marker_u16)
        ));
    }
    if prev.delta != curr.delta {
        changes.push(format!(
            "  delta: {} -> {}",
            render_delta_for_diff(prev.delta),
            render_delta_for_diff(curr.delta)
        ));
    }
    if prev.len_text != curr.len_text {
        changes.push(format!(
            "  textLen: {:04x} -> {:04x}",
            prev.len_text as u16, curr.len_text as u16
        ));
    }
    if prev.note_len != curr.note_len {
        changes.push(format!(
            "  noteLen: {:04x} -> {:04x}",
            prev.note_len as u16, curr.note_len as u16
        ));
    }
    let prev_note = prev.note.as_deref().unwrap_or("");
    let curr_note = curr.note.as_deref().unwrap_or("");
    if prev_note != curr_note {
        if prev.note_len == curr.note_len {
            changes.push("  note: (text changed)".to_string());
        } else {
            changes.push("  note: (length and text changed)".to_string());
        }
    }
    changes
}

fn diff_mode(prev: &[Rec], curr: &[Rec], show_cursor: bool) -> String {
    // Greedy match by heading text (first unmatched occurrence)
    let mut out = String::new();
    let mut used_prev = vec![false; prev.len()];

    for c in curr.iter() {
        // find first unmatched prev with identical text
        let mut match_idx: Option<usize> = None;
        for (j, p) in prev.iter().enumerate() {
            if !used_prev[j] && p.text == c.text {
                match_idx = Some(j);
                break;
            }
        }
        if let Some(j) = match_idx {
            used_prev[j] = true;
            let changes = diff_two_recs(&prev[j], c, show_cursor);
            if !changes.is_empty() {
                out.push_str(&format!("~ \"{}\"\n", c.text));
                for line in changes {
                    out.push_str(&line);
                    out.push('\n');
                }
            }
        } else {
            out.push_str(&format!("+ \"{}\"\n", c.text));
        }
    }
    for (j, p) in prev.iter().enumerate() {
        if !used_prev[j] {
            out.push_str(&format!("- \"{}\"\n", p.text));
        }
    }
    out
}

fn main() -> io::Result<()> {
    // Fast path: --diff <prev> <curr> [--show-cursor]
    let raw_args: Vec<String> = env::args().skip(1).collect();
    if raw_args.first().map(|s| s.as_str()) == Some("--diff") {
        // Accept optional --show-cursor as a trailing flag
        let show_cursor = raw_args.iter().any(|s| s == "--show-cursor");
        let paths: Vec<&str> = raw_args
            .iter()
            .skip(1)
            .filter(|s| s.as_str() != "--show-cursor")
            .map(|s| s.as_str())
            .collect();
        if paths.len() != 2 {
            usage(&env::args().next().unwrap_or_else(|| "otl".into()));
        }
        let prev_buf = fs::read(paths[0])?;
        let curr_buf = fs::read(paths[1])?;
        let prev_recs = parse_otl(&prev_buf, "latin1")?;
        let curr_recs = parse_otl(&curr_buf, "latin1")?;
        let report = diff_mode(&prev_recs, &curr_recs, show_cursor);
        print!("{report}");
        return Ok(());
    }

    // Normal modes
    let mut args = env::args().skip(1);
    let mut file: Option<String> = None;
    let mut out_json = false;
    let mut do_dump = false;
    let mut do_offsets = false;
    let mut do_validate = false;
    let mut plain_text = false;
    let mut canon = false;
    let mut enc = String::from("latin1");
    let mut assume_child_bit = false;
    let mut show_cursor = false;

    while let Some(a) = args.next() {
        match a.as_str() {
            "--json" => out_json = true,
            "--dump" => do_dump = true,
            "--offsets" => do_offsets = true,
            "--validate" => do_validate = true,
            "--text" => plain_text = true,
            "--canon" => canon = true,
            "--assume-child-bit" => assume_child_bit = true,
            "--show-cursor" => show_cursor = true,
            "--enc" => {
                if let Some(v) = args.next() {
                    enc = v;
                } else {
                    usage(&env::args().next().unwrap_or_else(|| "otl".into()));
                }
            }
            _ => {
                if file.is_none() {
                    file = Some(a);
                } else {
                    usage(&env::args().next().unwrap_or_else(|| "otl".into()));
                }
            }
        }
    }

    let prog = env::args().next().unwrap_or_else(|| "otl".into());
    let file = file.unwrap_or_else(|| usage(&prog));

    let mut buf = Vec::new();
    if file == "-" {
        io::stdin().read_to_end(&mut buf)?;
    } else {
        buf = fs::read(&file)?;
    }

    let recs = parse_otl(&buf, &enc)?;
    if do_validate {
        validate(&recs, assume_child_bit);
    }
    if do_dump {
        print!("{}", dump_recs(&recs));
        // fall through to also print offsets if requested
    }
    if do_offsets {
        print!("{}", dump_offsets(&recs));
        if !out_json && !plain_text && !canon {
            return Ok(());
        }
    }

    let tree = build_tree(&recs);
    if out_json {
        println!("{}", serde_json::to_string_pretty(&tree).unwrap());
    } else if plain_text {
        print!("{}", render_plain_all(&tree, 0));
    } else if canon {
        print!("{}", render_canon(&recs, show_cursor));
    } else {
        print!("{}", render_indented(&tree, ""));
    }

    Ok(())
}
