use serde::Serialize;
use std::env;
use std::fs;
use std::io::{self, Read};

#[derive(Debug, Clone)]
struct Rec {
    text: String,
    delta: i16,          // relative level change
    attr: u8,            // raw attribute flags
    collapsed: bool,     // marker FE FF vs FF FF
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
    children: Vec<Node>,
}

const MAGIC: [u8; 3] = [0x1a, 0x93, 0x1a];
const PREAMBLE: [u8; 6] = [0xff, 0x00, 0xff, 0xff, 0xff, 0xff];
const M_EXPANDED: u8 = 0xff;
const M_COLLAPSED: u8 = 0xfe;

fn decode_note(bytes: &[u8], enc: &str) -> String {
    match enc {
        "utf8" => String::from_utf8_lossy(bytes).to_string(),
        "latin1" => bytes.iter().map(|&b| b as char).collect::<String>(),
        "ascii" => bytes.iter().map(|&b| (b & 0x7f) as char).collect::<String>(),
        _ => String::from_utf8_lossy(bytes).to_string(),
    }
}

fn parse_otl(buf: &[u8], note_enc: &str) -> io::Result<Vec<Rec>> {
    let mut i = 0usize;
    let mut out = Vec::<Rec>::new();

    if buf.len() >= 3 && buf[0..3] == MAGIC { i += 3; }
    if buf.len() >= i + 6 && buf[i..i + 6] == PREAMBLE { i += 6; }

    while i < buf.len() {
        // stop on trailing FF FF 1A
        if i + 2 < buf.len() && buf[i] == 0xff && buf[i + 1] == 0xff && buf[i + 2] == 0x1a {
            break;
        }

        // find start of printable ASCII heading
        while i < buf.len() && !(buf[i] >= 0x20 && buf[i] <= 0x7e) {
            i += 1;
        }
        if i >= buf.len() { break; }

        // heading text up to 0xFF terminator
        let start = i;
        while i < buf.len() && (0x20..=0x7e).contains(&buf[i]) {
            i += 1;
        }
        if i >= buf.len() || buf[i] != 0xff {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "unterminated heading text"));
        }
        let text = String::from_utf8_lossy(&buf[start..i]).to_string();

        // attr + marker (FF FF expanded, FE FF collapsed)
        if i + 4 >= buf.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "truncated record header"));
        }
        let attr = buf[i + 1];
        let mark1 = buf[i + 2];
        let mark2 = buf[i + 3];
        if mark2 != 0xff || (mark1 != M_EXPANDED && mark1 != M_COLLAPSED) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "bad marker after heading"));
        }
        let collapsed = mark1 == M_COLLAPSED;

        // delta (i16 LE)
        let delta = i16::from_le_bytes([buf[i + 4], buf[i + 5]]);
        i += 6;

        // optional note
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
        text: "<ROOT>".to_string(),
        note: None,
        collapsed: false,
        flags: Flags { has_note: false, selected: false, has_next_sibling: false },
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
                text: "<LEVEL>".to_string(),
                note: None,
                collapsed: false,
                flags: Flags { has_note: false, selected: false, has_next_sibling: false },
                children: Vec::new(),
            };
            push_child(&mut root, &mut path, dummy);
        }

        let node = Node {
            text: r.text.clone(),
            note: r.note.clone(),
            collapsed: r.collapsed,
            flags: r.flags.clone(),
            children: Vec::new(),
        };
        push_child(&mut root, &mut path, node);
    }

    root.children
}

fn push_child(root: &mut Node, path: &mut Vec<usize>, child: Node) {
    // Walk down following path to get a mutable reference to the parent
    // (we must re-borrow afresh to keep Rust borrow checker happy)
    let mut ptr: *mut Node = root as *mut Node;
    unsafe {
        for &idx in path.iter() {
            ptr = &mut (*ptr).children[idx] as *mut Node;
        }
        (*ptr).children.push(child);
        let new_idx = (*ptr).children.len() - 1;
        path.push(new_idx); // make the new node the current context
    }
}

fn render_indented(nodes: &[Node], prefix: &str) -> String {
    let mut out = String::new();
    for n in nodes {
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

fn render_plain_all(nodes: &[Node], depth: usize) -> String {
    let mut out = String::new();
    for n in nodes {
        if n.text != "<LEVEL>" {
            let indent = " ".repeat(depth * 2);
            out.push_str(&indent);
            out.push_str(&n.text);
            out.push('\n');

            if let Some(note) = &n.note {
                let note_indent = " ".repeat((depth + 1) * 2);
                for line in note.replace("\r\n", "\n").lines() {
                    out.push_str(&note_indent);
                    out.push_str(line);
                    out.push('\n');
                }
            }
        }
        // Always recurse (ignore collapsed state)
        out.push_str(&render_plain_all(&n.children, depth + 1));
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

fn usage(prog: &str) -> ! {
    eprintln!("Usage: {prog} <file | -> [--json] [--dump] [--enc utf8|latin1|ascii] [--text]");
    std::process::exit(2);
}

fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let mut file: Option<String> = None;
    let mut out_json = false;
    let mut plain_text = false;
    let mut do_dump = false;
    let mut enc = String::from("latin1");

    while let Some(a) = args.next() {
        match a.as_str() {
            "--json" => out_json = true,
            "--dump" => do_dump = true,
            "--enc" => {
                if let Some(v) = args.next() { enc = v; } else {
                    usage(&env::args().next().unwrap_or_else(|| "otl".into()));
                }
            }
            "--text" => plain_text = true,
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
        if !out_json { return Ok(()); }
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
