use std::cmp::Ordering;
use std::io;

#[derive(Default)]
struct Nodes {
    all: Vec<String>,
}

impl Nodes {
    pub fn add(&mut self, n: &str) {
        let n = n.trim();
        if self.all.iter().all(|x| x != n) {
            self.all.push(n.to_owned());
        }
    }
    pub fn len(&self) -> usize {
        self.all.len()
    }
    pub fn index_of(&self, n: &str) -> usize {
        self.all
            .iter()
            .enumerate()
            .find_map(|(i, v)| if v == n { Some(i) } else { None })
            .unwrap()
    }
    pub fn print(&self, height: usize) {
        fn p(n: &str, height: usize, delim: &str) {
            println!(
                r#"  *+[F]{{\txt{{{}}}}} \ar@{{-}}[{}] {}"#,
                n,
                "d".repeat(height + 1),
                delim,
            );
        }
        for n in &self.all[..self.len() - 1] {
            p(n, height, "&");
        }
        p(&self.all[self.all.len() - 1], height, r#"\\"#);
    }
}

enum Item {
    Arrow {
        from: String,
        to: String,
        text: String,
    },
    Note {
        from: Option<String>,
        to: Option<String>,
        text: String,
        line: usize,
    },
    Group {
        text: String,
        lines: usize,
    },
}

impl Item {
    pub fn txt(t: &str) -> String {
        t.trim().replace('\\', "\\backslash").replace('_', "\\_").replace('\n', " \\\\\n")
    }
    pub fn print(&self, nodes: &Nodes) {
        match &self {
            Self::Arrow { from, to, text } => {
                let start = nodes.index_of(from);
                let end = nodes.index_of(to);
                let (l, c, d) = if start > end {
                    (start - end, "l", '_')
                } else {
                    (end - start, "r", '^')
                };
                println!(
                    r#"    {} \ar[{}]{}{{\txt{{{}}}}} {} \\"#,
                    "&".repeat(start),
                    c.repeat(l),
                    d,
                    Self::txt(text),
                    "&".repeat(nodes.len() - start - 1)
                );
            }
            Self::Note {
                from,
                to,
                text,
                line,
            } => {
                let start = from.as_ref().map(|x| nodes.index_of(x)).unwrap_or(0);
                let end = to
                    .as_ref()
                    .map(|x| nodes.index_of(x))
                    .unwrap_or_else(|| nodes.len() - 1);
                match start.cmp(&end) {
                    Ordering::Equal => {
                        println!(
                            r#"    {} *+[F.:<3pt>]{{\txt{{{}}}}} {} \\"#,
                            "&".repeat(start),
                            Self::txt(text),
                            "&".repeat(nodes.len() - end),
                        );
                    }
                    Ordering::Less => {
                        let middle = (end - start) / 2;
                        println!(
                            r#"    {} \save [].[{}] *+[F.:<3pt>]\frm{{}} \restore {} \txt{{{}}} {} \\"#,
                            "&".repeat(start),
                            "r".repeat(end - start),
                            "&".repeat(middle - start),
                            Self::txt(text),
                            "&".repeat(nodes.len() - middle - 1),
                        );
                    }
                    Ordering::Greater => {
                        panic!("unsupported note ordering on line {}", line);
                    }
                }
            }
            Self::Group { text, lines } => {
                if *lines == 0 {
                    println!("% empty group: {}", text);
                } else {
                    println!(
                        r#"    \save [].[{}] {{\txt{{{}}}}} \restore"#,
                        "r".repeat(nodes.len() - 1),
                        Self::txt(text),
                    );
                    println!(
                        r#"      \save [].[{}{}] *+[F-,]\frm{{}} \restore {} \\"#,
                        "d".repeat(*lines),
                        "r".repeat(nodes.len() - 1),
                        "&".repeat(nodes.len() - 1),
                    );
                }
            }
        }
    }
}

#[derive(Default)]
struct Items {
    title: String,
    options: String,
    nodes: Nodes,
    all: Vec<Item>,
}

impl Items {
    pub fn arrow(&mut self, a: &str, b: &str, text: String) {
        let a = a.trim();
        let b = b.trim();
        self.nodes.add(a);
        self.nodes.add(b);
        self.all.push(Item::Arrow {
            from: a.to_owned(),
            to: b.to_owned(),
            text,
        });
    }

    pub fn note(&mut self, a: Option<&str>, b: Option<&str>, text: String, line: usize) {
        let from = a.map(|x| x.trim().to_owned());
        let to = b.map(|x| x.trim().to_owned());
        self.all.push(Item::Note {
            from,
            to,
            text,
            line,
        });
    }

    pub fn group(&mut self, text: String) {
        self.all.push(Item::Group { text, lines: 0 })
    }

    pub fn end_group(&mut self) {
        let count = self.all.len();
        for i in (0..count).rev() {
            if let Item::Group { lines, .. } = &mut self.all[i] {
                *lines = count - i - 1;
                break;
            }
        }
    }

    pub fn add_text(&mut self, t: &str) {
        if let Some(Item::Arrow { text, .. } | Item::Note { text, .. } | Item::Group { text, .. }) =
            self.all.last_mut()
        {
            text.push('\n');
            text.push_str(t);
        }
    }

    pub fn len(&self) -> usize {
        self.all.len()
    }

    pub fn print(&self) {
        println!("{}", self.title);
        println!(r#"\[ \xymatrix {} {{"#, self.options);

        self.nodes.print(self.len());
        for item in &self.all {
            item.print(&self.nodes);
        }
        println!(r#"  {} \\"#, "&".repeat(self.nodes.len() - 1));
        println!(r#"}} \]"#);
    }
}

fn main() {
    let mut items = Items::default();
    let mut line = 1;

    loop {
        let mut t = String::new();
        if io::stdin().read_line(&mut t).unwrap() == 0 {
            break;
        }
        let s = t.trim();
        if s.starts_with('#') {
            continue;
        }
        if s.is_empty() {
            items.add_text(s);
            continue;
        }
        if let Some((label, text)) = s.split_once(':') {
            let label = label.trim();
            let text = text.trim().to_owned();

            if label == "xypic" {
                items.options = text;
            } else if label == "title" {
                items.title = text;
            } else if label == "note" {
                items.note(None, None, text, line);
            } else if let Some(x) = label.strip_prefix("note ") {
                if let Some((a, b)) = x.split_once(',') {
                    items.note(Some(a), Some(b), text, line);
                } else {
                    items.note(Some(x), Some(x), text, line);
                }
            } else if let Some((a, b)) = label.split_once("->") {
                items.arrow(a, b, text);
            } else if let Some((a, b)) = label.split_once("<-") {
                items.arrow(b, a, text);
            } else if label == "group" {
                items.group(text);
            } else {
                println!("% skip {}: {}", label, text);
            }
        } else if s == "end" {
            items.end_group();
        } else {
            items.add_text(s);
        }

        line += 1;
    }

    // \[ \xymatrix @C+10em {
    //     *+[F]{\txt{Client}} \ar@{-}[ddd] & *+[F]{\txt{Server}} \ar@{-}[ddd] \\
    //     \ar[r]^{\txt{hi there}} & \\
    //     & \ar[l]_{\txt{hi yourself}} \\
    //     & \\
    //   } \]

    items.print();
}