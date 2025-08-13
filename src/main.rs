use serde::Serialize;
use std::env;
use std::fs;
use std::io::{self, Read};

#[derive(Debug, Clone)]
struct Rec {
    text: String,
    delta: i16,      // relative level change
    attr: u8,        // raw attribute flags
    collapsed: bool, // marker FE FF vs FF FF
    note: Option<String>,
    flags: Flags,
}

#[derive(Debug, Clone, Serialize)]
struct Flags {
    has_note: bool,        // attr & 0x80
    selected: bool,        // attr & 0x20 (observed)
    has_next_sibling: bool // attr & 0x08 (observed)
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
        "Usage: {prog} <file | -> [--json] [--dump] [--enc utf8|latin1|ascii] [--text]"
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

// Decode heading bytes: char = b&0x7F; if high bit set, append a space.
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

        if k + 4 >= buf.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "truncated record header"));
        }
        let attr  = buf[k + 1];
        let mark1 = buf[k + 2];
        let mark2 = buf[k + 3];

        // Stray 0xFF? (marker must be FE/FF followed by FF)
        if mark2 != 0xff || (mark1 != M_EXPANDED && mark1 != M_COLLAPSED) {
            i = k + 1;
            continue;
        }

        // Valid record
        let text = decode_heading(&buf[i..k]);
        let collapsed = mark1 == M_COLLAPSED;
        let delta = i16::from_le_bytes([buf[k + 4], buf[k + 5]]);
        i = k + 6;

        // Optional note
        let mut note: Option<String> = None;
        if (attr & 0x80) != 0 {
            if i + 2 > buf.len() {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "truncated note length"));
            }
            let nlen = u16::from_le_bytes([buf[i], buf[i + 1]]) as usize;
            i += 2;
            if i + nlen > buf.len() {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "truncated note bytes"));
            }
            note = Some(decode_note(&buf[i..i + nlen], note_enc));
            i += nlen;
        }

        let flags = Flags {
            has_note: (attr & 0x80) != 0,
            selected: (attr & 0x20) != 0,
            has_next_sibling: (attr & 0x08) != 0,
        };

        out.push(Rec { text, delta, attr, collapsed, note, flags });
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
        let nlen = r.note.as_ref().map(|t| t.len()).unwrap_or(0);
        s.push_str(&format!(
            "{:>3}  L={:>2}  d={:>2}  attr=0x{:02x}  {} {} {}  note={}  {}\n",
            idx, lvl, r.delta, r.attr, c, sel, nxt, nlen, r.text
        ));
    }
    s
}

fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let mut file: Option<String> = None;
    let mut out_json = false;
    let mut do_dump = false;
    let mut plain_text = false;
    let mut enc = String::from("latin1");

    while let Some(a) = args.next() {
        match a.as_str() {
            "--json" => out_json = true,
            "--dump" => do_dump = true,
            "--text" => plain_text = true,
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
    if do_dump {
        print!("{}", dump_recs(&recs));
        if !out_json && !plain_text {
            return Ok(());
        }
    }

    let tree = build_tree(&recs);
    if out_json {
        println!("{}", serde_json::to_string_pretty(&tree).unwrap());
    } else if plain_text {
        print!("{}", render_plain_all(&tree, 0));
    } else {
        print!("{}", render_indented(&tree, ""));
    }

    Ok(())
}
