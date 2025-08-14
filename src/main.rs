use serde::Serialize;
use std::env;
use std::fs;
use std::io::{self, Read};

#[derive(Debug, Clone)]
struct Rec {
    text: String,
    delta: i16,      // relative level change (i16 LE)
    attr: u8,        // raw attribute flags
    collapsed: bool, // marker FE FF vs FF FF
    note: Option<String>,
    flags: Flags,

    // Byte offsets (for --offsets)
    off_text: usize,        // first heading byte
    len_text: usize,        // heading byte count (before 0xFF terminator)
    off_terminator: usize,  // 0xFF that ends heading text
    off_attr: usize,        // attr byte
    off_marker: usize,      // first of the 2 marker bytes
    off_delta: usize,       // first of the 2 delta bytes
    off_note_len: Option<usize>,
    off_note: Option<usize>,

    // Raw note length from file (u16), independent of --enc decoding (0 if no note)
    note_len: usize,
}

#[derive(Debug, Clone, Serialize)]
struct Flags {
    has_note: bool,        // attr & 0x80
    selected: bool,        // attr & 0x20 (caret on this heading)
    has_next_sibling: bool // attr & 0x08 (UI hint: there is a later sibling at same level)
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

fn usage(prog: &str) -> ! {
    eprintln!(
        "Usage: {prog} <file | -> \
         [--json] [--dump] [--offsets] [--validate] \
         [--enc utf8|latin1|ascii] [--text] [--canon]"
    );
    std::process::exit(2);
}

fn decode_note(bytes: &[u8], enc: &str) -> String {
    match enc {
        "utf8" => String::from_utf8_lossy(bytes).to_string(),
        "latin1" => bytes.iter().map(|&b| b as char).collect::<String>(),
        "ascii" => bytes.iter().map(|&b| (b & 0x7f) as char).collect::<String>(),
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

    if buf.len() >= 3 && buf[0..3] == MAGIC { i += 3; }
    if buf.len() >= i + 6 && buf[i..i + 6] == PREAMBLE { i += 6; }

    while i < buf.len() {
        // explicit EOF sentinels
        if i == buf.len() - 1 && buf[i] == 0x1a { break; }
        if i + 2 < buf.len() && buf[i] == 0xff && buf[i + 1] == 0xff && buf[i + 2] == 0x1a { break; }

        // Find next 0xFF; heading text may be zero-length.
        let mut k = i;
        while k < buf.len() && buf[k] != 0xff { k += 1; }
        if k >= buf.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "unterminated heading text"));
        }

        // Must have at least 4 bytes after the terminator for attr+marker+delta.
        if k + 4 >= buf.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "truncated record header"));
        }
        let attr  = buf[k + 1];
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
        let collapsed = mark1 == M_COLLAPSED;
        let delta = i16::from_le_bytes([buf[k + 4], buf[k + 5]]);

        let off_text = i;
        let len_text = k - i;
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

        if (attr & 0x80) != 0 {
            if i + 2 > buf.len() {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "truncated note length"));
            }
            off_note_len = Some(i);
            let nlen = u16::from_le_bytes([buf[i], buf[i + 1]]) as usize;
            i += 2;
            if i + nlen > buf.len() {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "truncated note bytes"));
            }
            off_note = Some(i);
            note_len = nlen;
            note = Some(decode_note(&buf[i..i + nlen], note_enc));
            i += nlen;
        }

        let flags = Flags {
            has_note: (attr & 0x80) != 0,
            selected: (attr & 0x20) != 0,
            has_next_sibling: (attr & 0x08) != 0,
        };

        out.push(Rec {
            text, delta, attr, collapsed, note, flags,
            off_text, len_text, off_terminator, off_attr, off_marker, off_delta, off_note_len, off_note,
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
        flags: Flags { has_note: false, selected: false, has_next_sibling: false },
        synthetic: true,
        children: Vec::new(),
    };

    let mut path: Vec<usize> = Vec::new(); // indexes from root to current parent/node
    let mut level: i32 = 0;

    for r in recs {
        level += r.delta as i32;
        if level < 0 { level = 0; }

        // shrink to target level
        while (path.len() as i32) > level { path.pop(); }
        // if we jumped more than +1, create dummy intermediates
        while (path.len() as i32) < level {
            let dummy = Node {
                text: String::new(),
                note: None,
                collapsed: false,
                flags: Flags { has_note: false, selected: false, has_next_sibling: false },
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
    // Re-walk the path to get a fresh mutable reference to the parent.
    let mut ptr: *mut Node = root as *mut Node;
    unsafe {
        for &idx in path.iter() {
            ptr = &mut (*ptr).children[idx] as *mut Node;
        }
        (*ptr).children.push(child);
        let new_idx = (*ptr).children.len() - 1;
        path.push(new_idx); // make the new node current
    }
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
        let sel  = if n.flags.selected { "*" } else { " " };
        out.push_str(&format!("{prefix}{fold}{sel} {}\n", n.text));

        if let Some(note) = &n.note {
            for line in note.replace("\r\n", "\n").lines() {
                out.push_str(&format!("{prefix}    > {}\n", line));
            }
        }
        if !n.collapsed && !n.children.is_empty() {
            out.push_str(&render_indented(&n.children, &(prefix.to_string() + "    ")));
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
        let nxt = if r.flags.has_next_sibling { 'N' } else { ' ' };
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
            r.off_text, r.len_text,
            r.off_terminator,
            r.off_attr, r.attr,
            r.off_marker, m,
            r.off_delta,
            match r.off_note_len { Some(o)=>format!(" nlen[{:#06x}]", o), None=>"".into() },
            match r.off_note     { Some(o)=>format!(" note[{:#06x}]", o), None=>"".into() },
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
            '"'  => out.push_str("\\\""),
            _    => out.push(ch),
        }
    }
    out
}

// Format attr as 8 characters (bits 7..0):
// Known bits use letters (lowercase=0, UPPER=1):
//   0x80 -> n/N (note present)
//   0x20 -> c/C (cursor/selected)
//   0x08 -> m/M (more siblings at same level)
// Unknown bits print '0' or '1'.
fn fmt_attr_bits(attr: u8) -> String {
    let mut s = String::with_capacity(8);
    for i in (0..8).rev() {
        let mask = 1u8 << i;
        let set = (attr & mask) != 0;
        let ch = match mask {
            0x80 => if set { 'N' } else { 'n' },
            0x20 => if set { 'C' } else { 'c' },
            0x08 => if set { 'M' } else { 'm' },
            _ => if set { '1' } else { '0' },
        };
        s.push(ch);
    }
    s
}

/// Offset-free, insertion-stable, bit-complete dump:
/// [attr,abcdefgh] [mark,%04x] [delta,%04x] [textLen,%04x] "text"
/// If a note exists, emit on following lines:
///   [noteLen=%04x]
///   <note text...>   (CRLF normalized to LF)
fn render_canon(recs: &[Rec]) -> String {
    let mut out = String::new();
    for r in recs {
        // raw marker (u16 LE): FFFF=expanded, FFFE=collapsed
        let mark_le: u16 = if r.collapsed { 0xFFFE } else { 0xFFFF };
        // raw delta (two's complement of i16)
        let delta_raw: u16 = r.delta as u16;
        // raw length from file
        let text_len_raw: u16 = r.len_text as u16;

        out.push_str(&format!(
            "[attr,{}] [mark,{:04x}] [delta,{:04x}] [textLen,{:04x}] \"{}\"\n",
            fmt_attr_bits(r.attr),
            mark_le,
            delta_raw,
            text_len_raw,
            escape_headline(&r.text)
        ));

        if r.flags.has_note {
            out.push_str(&format!("[noteLen={:04x}]\n", r.note_len as u16));
            if let Some(note) = &r.note {
                let note_norm = note.replace("\r\n", "\n");
                out.push_str(&note_norm);
                if !note_norm.ends_with('\n') {
                    out.push('\n');
                }
            }
        }
    }
    out
}

/// Validate derived invariants and print warnings to stderr
fn validate(recs: &[Rec]) {
    // compute levels
    let mut levels = Vec::with_capacity(recs.len());
    let mut lvl = 0i32;
    for r in recs {
        lvl += r.delta as i32;
        if lvl < 0 { lvl = 0; }
        levels.push(lvl);
    }

    // 0x08 = has any later sibling at the same level (before subtree ends)
    for i in 0..recs.len() {
        let my = levels[i];
        let mut has_later_sibling = false;
        for j in (i + 1)..recs.len() {
            if levels[j] < my { break; }          // left this subtree
            if levels[j] == my { has_later_sibling = true; break; }
        }
        let bit = (recs[i].attr & 0x08) != 0;
        if has_later_sibling != bit {
            eprintln!(
              "WARN: rec #{:03} sibling bit mismatch (attr={}, expected={}) at attr[{:#06x}]",
              i, bit, has_later_sibling, recs[i].off_attr
            );
        }
    }

    // selection bit sanity
    let selected: Vec<_> = recs.iter().enumerate().filter(|(_,r)| r.flags.selected).map(|(i,_)| i).collect();
    if selected.len() > 1 {
        eprintln!("WARN: multiple selections set at indices {:?}", selected);
    }
}

fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let mut file: Option<String> = None;
    let mut out_json = false;
    let mut do_dump = false;
    let mut do_offsets = false;
    let mut do_validate = false;
    let mut plain_text = false;
    let mut canon = false;
    let mut enc = String::from("latin1");

    while let Some(a) = args.next() {
        match a.as_str() {
            "--json" => out_json = true,
            "--dump" => do_dump = true,
            "--offsets" => do_offsets = true,
            "--validate" => do_validate = true,
            "--text" => plain_text = true,
            "--canon" => canon = true,
            "--enc" => {
                if let Some(v) = args.next() { enc = v; } else {
                    usage(&env::args().next().unwrap_or_else(|| "otl".into()));
                }
            }
            _ => {
                if file.is_none() { file = Some(a); }
                else { usage(&env::args().next().unwrap_or_else(|| "otl".into())); }
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
        validate(&recs);
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
        print!("{}", render_canon(&recs));
    } else {
        print!("{}", render_indented(&tree, ""));
    }

    Ok(())
}
