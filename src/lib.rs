use std::alloc::{alloc, dealloc, Layout};
use std::ffi::CString;
use std::os::raw::c_char;

// ====== Helpers ======

fn is_alpha(c: u8) -> bool {
    c.is_ascii_alphabetic()
}

fn is_digit(c: u8) -> bool {
    c.is_ascii_digit()
}

fn is_alnum(c: u8) -> bool {
    is_alpha(c) || is_digit(c)
}

fn is_operator(c: u8) -> bool {
    matches!(c, b'+' | b'-' | b'<' | b'>' | b'&' | b'|')
}

// ====== Constants ======

const MATH_FUNCS: [&str; 38] = [
    "arccos", "arcsin", "arctan", "arg", "cos", "cosh", "cot", "coth",
    "csc", "ctg", "deg", "det", "dim", "exp", "gcd", "hom", "inf",
    "ker", "lg", "lim", "liminf", "limsup", "ln", "log", "max", "min",
    "mod", "Pr", "sec", "sin", "sinh", "sup", "tan", "tanh",
    "tg", "tr", "Re", "Im",
];

// GREEK_LOWER: single-char Greek codes from manual §4.16
const GREEK_LOWER_KEYS: [char; 24] = [
    'a', 'b', 'g', 'd', 'e', 'z', 'ä', 'ô', 'i', 'k', 'l', 'm',
    'n', 'x', 'o', 'p', 'r', 's', 't', 'y', 'f', 'h', 'ç', 'ö',
];
const GREEK_LOWER_VALS: [&str; 24] = [
    "\\alpha", "\\beta", "\\Gamma", "\\delta", "\\epsilon", "\\zeta",
    "\\eta", "\\theta", "\\iota", "\\kappa", "\\lambda", "\\mu",
    "\\nu", "\\xi", "\\omicron", "\\pi", "\\rho", "\\sigma",
    "\\tau", "\\upsilon", "\\phi", "\\chi", "\\psi", "\\omega",
];

const GREEK_UPPER_KEYS: [char; 24] = [
    'A', 'B', 'G', 'D', 'E', 'Z', 'Ä', 'Ô', 'I', 'K', 'L', 'M',
    'N', 'X', 'O', 'P', 'R', 'S', 'T', 'Y', 'F', 'H', 'C', 'Ö',
];
const GREEK_UPPER_VALS: [&str; 24] = [
    "A", "B", "\\Gamma", "\\Delta", "E", "Z", "H", "\\Theta",
    "I", "K", "\\Lambda", "M", "N", "\\Xi", "O", "\\prod", "R",
    "\\sum", "T", "Y", "\\Phi", "X", "\\Psi", "\\Omega",
];

// Dollar codes (sorted by length desc for longest match)
const DOLLAR_KEYS: [&str; 68] = [
    "<=>", "nt//", "ntldr", "pllr", "pldr", "pldl", "ntelm", "ntdlv",
    "pllb", "plrb", "plsb", "plso", "dlv=", "bdlv=", "nt=", "nt<", "nt>", "nt|",
    "%%", "krd", "+-", "-+", "==", "<=", ">=", "!=", "//", "elm", "belm",
    "dlv", "bdlv", "dsn", "unie", "vrs", "plb", "plo", "ldr", "slg", "cgr",
    "par", "rhk", "4kt", "3hk", "hk", "rh", "bol",
    "**", "na", "en", "of", "asa", "inf", "int", "plr", "pll", "nt",
    "N", "Z", "Q", "R", "C", "O", "A", "E", "d", "*", "/", "=",
];

const DOLLAR_VALS: [&str; 68] = [
    "\\iff", "\\nparallel", "\\not\\perp",
    "\\leftrightarrow", "\\implies", "\\impliedby",
    "\\notin", "\\not\\subset",
    "\\nwarrow", "\\nearrow", "\\nearrow", "\\searrow",
    "\\subseteq", "\\supseteq",
    "\\neq", "\\nless", "\\ngtr", "\\nmid",
    "\\%\\%", "\\#", "\\pm", "\\mp",
    "\\equiv", "\\leq", "\\geq", "\\neq", "\\parallel",
    "\\in", "\\ni", "\\subset", "\\supset", "\\cap", "\\cup", "\\setminus",
    "\\uparrow", "\\downarrow", "\\perp", "\\sim", "\\cong",
    "\\Diamond", "\\square", "\\square", "\\triangle", "\\angle", "\\measuredangle",
    "\\bullet",
    "\\ast", "\\circ", "\\land", "\\lor", "\\iff", "\\infty", "\\int",
    "\\to", "\\leftarrow", "\\neg",
    "\\mathbb{N}", "\\mathbb{Z}", "\\mathbb{Q}", "\\mathbb{R}", "\\mathbb{C}",
    "\\emptyset", "\\forall", "\\exists", "\\partial",
    "\\times", "\\div", "\\approx",
];

// ====== Converter ======

struct Error {
    type_: String,
    message: String,
    position: usize,
    context: String,
}

struct Converter<'a> {
    input: &'a [u8],
    pos: usize,
    output: String,
    errors: Vec<Error>,
    compact: bool,
}

impl<'a> Converter<'a> {
    fn new(input: &'a [u8], compact: bool) -> Self {
        Converter {
            input,
            pos: 0,
            output: String::with_capacity(input.len() * 2),
            errors: Vec::new(),
            compact,
        }
    }

    fn peek(&self, offset: usize) -> u8 {
        let idx = self.pos + offset;
        if idx < self.input.len() { self.input[idx] } else { 0 }
    }

    fn remaining(&self) -> &'a [u8] {
        &self.input[self.pos..]
    }

    fn starts_with(&self, s: &str) -> bool {
        self.remaining().starts_with(s.as_bytes())
    }

    fn emit(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn emit_char(&mut self, c: char) {
        self.output.push(c);
    }

    fn error(&mut self, type_: &str, msg: &str, pos: usize) {
        let ctx_start = if pos > 20 { pos - 20 } else { 0 };
        let ctx_end = std::cmp::min(self.input.len(), ctx_start + 40);
        let context = String::from_utf8_lossy(&self.input[ctx_start..ctx_end]).to_string();
        self.errors.push(Error {
            type_: type_.to_string(),
            message: msg.to_string(),
            position: pos,
            context,
        });
    }

    fn collect_while<F>(&mut self, f: F) -> String
    where
        F: Fn(u8) -> bool,
    {
        let start = self.pos;
        while self.pos < self.input.len() && f(self.peek(0)) {
            self.pos += 1;
        }
        String::from_utf8_lossy(&self.input[start..self.pos]).to_string()
    }

    fn try_math_func(&mut self) -> Option<&'static str> {
        let rem = self.remaining();
        for func in MATH_FUNCS {
            if rem.starts_with(func.as_bytes()) {
                let after = self.pos + func.len();
                if after >= self.input.len() || !is_alpha(self.input[after]) {
                    self.pos += func.len();
                    return Some(func);
                }
            }
        }
        None
    }

    fn lookahead_math_func(&self, pos: usize) -> Option<&'static str> {
        if pos >= self.input.len() { return None; }
        let rem = &self.input[pos..];
        for func in MATH_FUNCS {
            if rem.starts_with(func.as_bytes()) {
                let after = pos + func.len();
                if after >= self.input.len() || !is_alpha(self.input[after]) {
                    return Some(func);
                }
            }
        }
        None
    }

    fn index_has_closer(&self, mut search: usize, stop_slash: bool) -> bool {
        let mut depth = 0;
        while search < self.input.len() {
            if self.input[search..].starts_with(b"$br") || self.input[search..].starts_with(b"$wt") {
                depth += 1;
                search += 3;
                continue;
            }
            let c = self.input[search];
            if c == b'^' || c == b'_' {
                let width = if search + 1 < self.input.len() && self.input[search + 1] == c { 2 } else { 1 };
                depth += 1;
                search += width;
                continue;
            }
            if c == b'#' {
                if depth == 0 { return true; }
                depth -= 1;
            } else if stop_slash && depth == 0 && c == b'/' {
                return false;
            }
            search += 1;
        }
        false
    }

    fn top_level_hash_ahead(&self) -> bool {
        let mut search = self.pos;
        let mut stack: Vec<u8> = Vec::new();
        while search < self.input.len() {
            if self.input[search..].starts_with(b"$br") || self.input[search..].starts_with(b"$wt") {
                stack.push(b'b');
                search += 3;
                continue;
            }
            let c = self.input[search];
            if c == b'^' || c == b'_' {
                let width = if search + 1 < self.input.len() && self.input[search + 1] == c { 2 } else { 1 };
                stack.push(b'i');
                search += width;
                continue;
            }
            if c == b'#' {
                if stack.is_empty() { return true; }
                stack.pop();
            }
            search += 1;
        }
        false
    }

    // Collect sub/superscript content
    fn collect_sub_or_super(&mut self) -> String {
        if self.top_level_hash_ahead() {
            let collected = self.collect_until_hash();
            let mut sub = Converter::new(&collected, true);
            sub.convert_inner();
            self.errors.extend(sub.errors);
            sub.output
        } else if self.peek(0) == b'{' {
            self.pos += 1;
            let mut depth = 1;
            let mut collected = Vec::new();
            while self.pos < self.input.len() && depth > 0 {
                let c = self.peek(0);
                self.pos += 1;
                if c == b'{' { depth += 1; }
                else if c == b'}' { depth -= 1; if depth > 0 { collected.push(c); } continue; }
                if depth > 0 { collected.push(c); }
            }
            let mut sub = Converter::new(&collected, true);
            sub.convert_inner();
            self.errors.extend(sub.errors);
            sub.output
        } else {
            if self.pos < self.input.len() {
                let c = self.peek(0) as char;
                self.pos += 1;
                c.to_string()
            } else {
                String::new()
            }
        }
    }

    fn handle_sub(&mut self) {
        self.pos += 1;
        let content = self.collect_sub_or_super();
        self.output.push('_');
        self.output.push('{');
        self.output.push_str(&content);
        self.output.push('}');
    }

    fn handle_super(&mut self) {
        self.pos += 1;
        let content = self.collect_sub_or_super();
        self.output.push('^');
        self.output.push('{');
        self.output.push_str(&content);
        self.output.push('}');
    }

    fn next_char(&self) -> (char, usize) {
        let b = self.peek(0);
        let width = if b < 0x80 { 1 } else if b & 0xE0 == 0xC0 { 2 } else if b & 0xF0 == 0xE0 { 3 } else if b & 0xF8 == 0xF0 { 4 } else { 1 };
        if self.pos + width <= self.input.len() {
            if let Ok(s) = std::str::from_utf8(&self.input[self.pos..self.pos + width]) {
                if let Some(c) = s.chars().next() {
                    return (c, width);
                }
            }
        }
        (b as char, 1)
    }

    fn handle_greek(&mut self) {
        // μ or µ consumed already
        if self.pos >= self.input.len() {
            self.emit("\\mu");
            return;
        }
        let (key, width) = self.next_char();
        self.pos += width;

        // Check single-char Greek (lowercase)
        for i in 0..GREEK_LOWER_KEYS.len() {
            if GREEK_LOWER_KEYS[i] == key {
                self.emit(GREEK_LOWER_VALS[i]);
                return;
            }
        }
        // Check uppercase
        for i in 0..GREEK_UPPER_KEYS.len() {
            if GREEK_UPPER_KEYS[i] == key {
                self.emit(GREEK_UPPER_VALS[i]);
                return;
            }
        }
        self.error("OnbekendGrieks", &format!("Onbekende Griekse code '{}'", key), self.pos - 1);
        self.emit("\\mu");
        self.emit_char(key);
    }

    fn skip_space_filler(&mut self) {
        if self.peek(0) == b' ' || self.peek(0) == 0xA8 {
            self.pos += 1;
        } else if self.remaining().starts_with(&[0xC2, 0xA8]) {
            self.pos += 2;
        }
    }

    fn handle_shape(&mut self) -> bool {
        let kind = if self.starts_with("st") {
            "st"
        } else if self.starts_with("mat") {
            "mat"
        } else if self.starts_with("det") {
            "det"
        } else if self.starts_with("com") {
            "com"
        } else {
            return false;
        };
        self.pos += kind.len();
        while self.peek(0) == b'.' || is_digit(self.peek(0)) {
            self.pos += 1;
        }
        if self.peek(0) == b' ' { self.pos += 1; }
        let mut raw = Vec::new();
        while self.pos < self.input.len() {
            if self.remaining().starts_with(b"$#") {
                self.pos += 2;
                break;
            }
            raw.push(self.peek(0));
            self.pos += 1;
        }
        let text = String::from_utf8_lossy(&raw);
        let rows: Vec<&str> = text.split('\n').filter(|r| !r.trim().is_empty()).collect();
        let mut out = String::new();
        if kind == "com" {
            out.push_str("\\binom{");
            for (i, row) in rows.iter().enumerate() {
                if i > 0 { out.push_str("}{"); }
                let mut cvt = Converter::new(row.trim().as_bytes(), true);
                cvt.convert_inner();
                self.errors.extend(cvt.errors);
                out.push_str(&cvt.output);
            }
            out.push('}');
        } else {
            let env = match kind {
                "st" => "cases",
                "mat" => "pmatrix",
                _ => "vmatrix",
            };
            out.push_str("\\begin{");
            out.push_str(env);
            out.push('}');
            for (i, row) in rows.iter().enumerate() {
                if i > 0 { out.push_str("\\\\"); }
                let cells: Vec<&str> = if kind == "st" { vec![row] } else { row.split_whitespace().collect() };
                for (j, cell) in cells.iter().enumerate() {
                    if j > 0 { out.push('&'); }
                    let cleaned: String = cell.chars().filter(|c| !c.is_whitespace()).collect();
                    let mut cvt = Converter::new(cleaned.as_bytes(), true);
                    cvt.convert_inner();
                    self.errors.extend(cvt.errors);
                    out.push_str(&cvt.output);
                }
            }
            out.push_str("\\end{");
            out.push_str(env);
            out.push('}');
        }
        self.emit(&out);
        true
    }

    fn handle_dollar(&mut self) {
        self.pos += 1; // skip $
        let needs_lead = self.output.ends_with(|c: char| c.is_alphanumeric());

        // Check for $br (fraction)
        if self.starts_with("br") {
            if needs_lead { self.emit(" "); }
            self.pos += 2;
            self.skip_space_filler();
            let num = self.collect_until_fraction_slash();
            let den = self.collect_until_hash();
            let mut num_cvt = Converter::new(&num, true);
            let mut den_cvt = Converter::new(&den, true);
            num_cvt.convert_inner();
            den_cvt.convert_inner();
            self.errors.extend(num_cvt.errors);
            self.errors.extend(den_cvt.errors);
            self.emit("\\frac{");
            self.emit(&unwrap_text(&num_cvt.output));
            self.emit("}{");
            self.emit(&unwrap_text(&den_cvt.output));
            self.emit("}");
            return;
        }

        // $wt (root)
        if self.starts_with("wt") {
            if needs_lead { self.emit(" "); }
            self.pos += 2;
            self.skip_space_filler();
            let rad = self.collect_until_hash_or_single();
            let mut rad_cvt = Converter::new(&rad, true);
            rad_cvt.convert_inner();
            self.errors.extend(rad_cvt.errors);
            self.emit("\\sqrt{");
            self.emit(&rad_cvt.output);
            self.emit("}");
            return;
        }

        // Line forms: $bgb, $strb, $slgb, $hkb, $plrb, $pllb
        let line_forms = [
            ("bgb", "\\overline"), ("strb", "\\overline"), ("slgb", "\\tilde"),
            ("hkb", "\\hat"), ("plrb", "\\vec"), ("pllb", "\\overleftarrow"),
        ];
        for (code, latex) in &line_forms {
            if self.starts_with(code) {
                if needs_lead { self.emit(" "); }
                self.pos += code.len();
                if self.peek(0) == b' ' { self.pos += 1; }
                let raw = self.collect_until_hash();
                let mut inner = Converter::new(&raw, true);
                inner.convert_inner();
                self.errors.extend(inner.errors);
                self.emit(latex);
                self.emit("{");
                self.emit(&inner.output);
                self.emit("}");
                return;
            }
        }

        if self.handle_shape() {
            return;
        }

        // Dollar codes
        for i in 0..DOLLAR_KEYS.len() {
            if self.starts_with(DOLLAR_KEYS[i]) {
                self.pos += DOLLAR_KEYS[i].len();
                if self.output.ends_with(|c: char| c.is_alphanumeric()) {
                    self.emit(" ");
                }
                self.emit(DOLLAR_VALS[i]);
                if self.pos < self.input.len() && is_alpha(self.input[self.pos]) {
                    self.emit(" ");
                }
                return;
            }
        }

        self.error("OnbekendeDollar", &format!("Onbekende $-code op positie {}", self.pos), self.pos - 1);
        self.emit("\\$");
    }

    fn collect_until(&mut self, stop_slash: bool) -> Vec<u8> {
        let mut collected = Vec::new();
        let mut stack: Vec<u8> = Vec::new();
        while self.pos < self.input.len() {
            if self.remaining().starts_with(b"$br") || self.remaining().starts_with(b"$wt") {
                stack.push(b'b');
                collected.extend_from_slice(&self.input[self.pos..self.pos + 3]);
                self.pos += 3;
                continue;
            }
            let c = self.peek(0);
            if c == b'^' || c == b'_' {
                let width = if self.peek(1) == c { 2 } else { 1 };
                if self.index_has_closer(self.pos + width, stop_slash) {
                    stack.push(b'i');
                }
                collected.extend_from_slice(&self.input[self.pos..self.pos + width]);
                self.pos += width;
                continue;
            }
            if c == b'#' {
                if stack.is_empty() {
                    self.pos += 1;
                    return collected;
                }
                stack.pop();
                collected.push(c);
                self.pos += 1;
                continue;
            }
            if stop_slash && stack.is_empty() && c == b'/' {
                self.pos += 1;
                if self.peek(0) == b'/' { self.pos += 1; }
                return collected;
            }
            collected.push(c);
            self.pos += 1;
        }
        collected
    }

    fn collect_until_fraction_slash(&mut self) -> Vec<u8> {
        self.collect_until(true)
    }

    fn collect_until_hash(&mut self) -> Vec<u8> {
        self.collect_until(false)
    }

    fn collect_until_hash_or_single(&mut self) -> Vec<u8> {
        if self.top_level_hash_ahead() {
            self.collect_until_hash()
        } else {
            let remaining = self.input[self.pos..].to_vec();
            self.pos = self.input.len();
            remaining
        }
    }

    fn handle_alpha(&mut self) {
        if let Some(func) = self.try_math_func() {
            self.emit("\\");
            self.emit(func);
            return;
        }

        let start = self.pos;
        while self.pos < self.input.len() {
            let c = self.peek(0);
            if is_alpha(c) {
                if self.lookahead_math_func(self.pos).is_some() {
                    break;
                }
                self.pos += 1;
            } else if c == b' ' {
                self.pos += 1;
            } else {
                break;
            }
        }

        let text = String::from_utf8_lossy(&self.input[start..self.pos]).to_string();
        if text.is_empty() { return; }

        let words: Vec<&str> = text.split(' ').filter(|w| !w.is_empty()).collect();
        if words.iter().all(|w| w.len() == 1) {
            if self.compact {
                self.emit(&words.join(""));
            } else if words.len() > 1 {
                self.emit(&words.join(" \\cdot "));
            } else {
                self.emit(words[0]);
            }
            let trailing = text.len() - text.trim_end().len();
            if trailing > 0 {
                for _ in 0..trailing { self.emit(" "); }
            }
        } else {
            self.emit("\\text{");
            self.emit(&text);
            self.emit("}");
        }
    }

    fn simple_fraction_right_len(&self, pos: usize) -> usize {
        let mut n = 0;
        while pos + n < self.input.len() && is_digit(self.input[pos + n]) { n += 1; }
        if n > 0 {
            let end = pos + n;
            if end >= self.input.len() || self.input[end] != b'.' { return n; }
            return 0;
        }
        if pos < self.input.len() && is_alpha(self.input[pos]) {
            let end = pos + 1;
            if end >= self.input.len() { return 1; }
            let next = self.input[end];
            if !is_alnum(next) && next != b'_' && next != b'^' { return 1; }
        }
        0
    }

    fn simple_fraction_left_len(&self) -> usize {
        let mut start = self.pos;
        while start > 0 && is_digit(self.input[start - 1]) { start -= 1; }
        if start < self.pos {
            let token = String::from_utf8_lossy(&self.input[start..self.pos]);
            if self.output.ends_with(&*token) { return self.pos - start; }
            return 0;
        }
        if self.pos > 0 && is_alpha(self.input[self.pos - 1])
            && (self.pos < 2 || !is_alnum(self.input[self.pos - 2]))
            && self.output.ends_with(self.input[self.pos - 1] as char)
        {
            return 1;
        }
        0
    }

    fn starts_simple_fraction(&self, pos: usize) -> bool {
        let mut left = 0;
        while pos + left < self.input.len() {
            let b = self.input[pos + left];
            if is_digit(b) || (left == 0 && is_alpha(b)) { left += 1; } else { break; }
        }
        if left == 0 { return false; }
        let slash = pos + left;
        slash < self.input.len() && self.input[slash] == b'/'
            && self.simple_fraction_right_len(slash + 1) > 0
    }

    fn convert_inner(&mut self) {
        while self.pos < self.input.len() {
            let c = self.peek(0);

            // Skip spaces at top level
            if c == b' ' {
                // Preserve single space between tokens
                if !self.output.is_empty() && !self.output.ends_with(' ') {
                    self.emit(" ");
                }
                self.pos += 1;
                continue;
            }

            if c == b'\n' {
                self.pos += 1;
                self.emit(" \\\\ ");
                continue;
            }

            if c == b'#' {
                self.pos += 1;
                continue;
            }

            // μ|µ for Greek
            if c == 0xCE && self.pos + 1 < self.input.len() && self.input[self.pos + 1] == 0xBB {
                // λ in UTF-8 — pass through as \lambda
                self.emit("\\lambda");
                self.pos += 2;
                continue;
            }
            if self.pos + 1 < self.input.len()
                && ((c == 0xCE && self.input[self.pos + 1] == 0xBC)
                    || (c == 0xC2 && self.input[self.pos + 1] == 0xB5))
            {
                self.pos += 2;
                self.handle_greek();
                continue;
            }

            // ¨ — spatievuller
            if c == 0xA8 || (c == 0xC2 && self.pos + 1 < self.input.len() && self.input[self.pos + 1] == 0xA8) {
                let after = if c == 0xC2 { self.pos + 2 } else { self.pos + 1 };
                self.pos = after;
                if after < self.input.len() && self.input[after] == b'=' { continue; }
                if self.starts_simple_fraction(after) { continue; }
                self.emit(" ");
                continue;
            }

            // $ codes
            if c == b'$' {
                self.handle_dollar();
                continue;
            }

            // ^^ left superscript
            if c == b'^' && self.pos + 1 < self.input.len() && self.peek(1) == b'^' {
                self.pos += 2;
                let idx_raw = self.collect_until_hash();
                let mut idx_cvt = Converter::new(&idx_raw, true);
                idx_cvt.convert_inner();
                self.errors.extend(idx_cvt.errors);

                if self.starts_with("$wt") {
                    self.pos += 3;
                    self.skip_space_filler();
                    let rad_raw = self.collect_until_hash_or_single();
                    let mut rad_cvt = Converter::new(&rad_raw, true);
                    rad_cvt.convert_inner();
                    self.errors.extend(rad_cvt.errors);
                    self.emit("\\sqrt[");
                    self.emit(&idx_cvt.output);
                    self.emit("]{");
                    self.emit(&rad_cvt.output);
                    self.emit("}");
                } else if self.remaining().starts_with(b"log") {
                    let nxt = self.pos + 3;
                    if nxt >= self.input.len() || !is_alpha(self.input[nxt]) {
                        self.pos += 3;
                        self.emit("\\log_{");
                        self.emit(&idx_cvt.output);
                        self.emit("}");
                    } else {
                        self.error("LinksSuperscript", "Links superscript zonder geldig doel", self.pos - 2);
                        self.emit("{}^{");
                        self.emit(&idx_cvt.output);
                        self.emit("}");
                    }
                } else {
                    self.error("LinksSuperscript", "Links superscript zonder geldig doel", self.pos - 2);
                    self.emit("{}^{");
                    self.emit(&idx_cvt.output);
                    self.emit("}");
                }
                continue;
            }

            if c == b'^' {
                self.handle_super();
                continue;
            }

            if c == b'_' {
                self.handle_sub();
                continue;
            }

            // Letters
            if is_alpha(c) {
                self.handle_alpha();
                continue;
            }

            // Digits
            if is_digit(c) {
                let s = self.collect_while(|x| is_digit(x));
                self.emit(&s);
                continue;
            }

            // Equality ($= is \approx)
            if c == b'=' {
                if !self.compact && !self.output.ends_with(' ') {
                    self.emit(" ");
                }
                self.emit("=");
                if !self.compact {
                    self.emit(" ");
                }
                self.pos += 1;
                continue;
            }

            if is_operator(c) {
                if !self.compact && !self.output.ends_with(' ') {
                    self.emit(" ");
                }
                self.emit_char(c as char);
                if !self.compact && self.pos + 1 < self.input.len() && (is_alnum(self.input[self.pos + 1]) || self.input[self.pos + 1] == b'(' || self.input[self.pos + 1] == b'[') {
                    self.emit(" ");
                }
                self.pos += 1;
                continue;
            }

            // * → \times (from dollar codes)
            if c == b'*' {
                if !self.compact && !self.output.ends_with(' ') {
                    self.emit(" ");
                }
                self.emit("\\times");
                if !self.compact {
                    self.emit(" ");
                }
                self.pos += 1;
                continue;
            }

            // / → simple fraction or \div
            if c == b'/' {
                let left_len = self.simple_fraction_left_len();
                if left_len > 0 {
                    let right_len = self.simple_fraction_right_len(self.pos + 1);
                    if right_len > 0 {
                        let left_start = self.pos - left_len;
                        let left = String::from_utf8_lossy(&self.input[left_start..self.pos]).to_string();
                        let right = String::from_utf8_lossy(&self.input[self.pos + 1..self.pos + 1 + right_len]).to_string();
                        for _ in 0..left_len { self.output.pop(); }
                        self.emit("\\frac{");
                        self.emit(&left);
                        self.emit("}{");
                        self.emit(&right);
                        self.emit("}");
                        self.pos += 1 + right_len;
                        continue;
                    }
                }
                self.emit("\\div");
                self.pos += 1;
                continue;
            }

            // Parentheses/brackets
            if c == b'(' || c == b')' || c == b'[' || c == b']' {
                self.emit_char(c as char);
                self.pos += 1;
                continue;
            }

            if c == b'.' {
                self.emit_char('.');
                self.pos += 1;
                continue;
            }

            if c == b',' {
                self.emit_char(',');
                if self.pos + 1 < self.input.len() && self.peek(1) != b' ' && !is_digit(self.peek(1)) {
                    self.emit(" ");
                }
                self.pos += 1;
                continue;
            }

            if c == b'!' || c == b'?' || c == b'\'' {
                self.emit_char(c as char);
                self.pos += 1;
                continue;
            }

            // Default: pass through
            self.emit_char(c as char);
            self.pos += 1;
        }
    }

    fn to_json(&self, _display: bool) -> String {
        let mut json = String::new();
        json.push('{');
        json.push_str("\"output\":\"");
        // Escape JSON string
        for c in self.output.chars() {
            match c {
                '"' => json.push_str("\\\""),
                '\\' => json.push_str("\\\\"),
                '\n' => json.push_str("\\n"),
                '\r' => json.push_str("\\r"),
                '\t' => json.push_str("\\t"),
                other => json.push(other),
            }
        }
        json.push('"');

        json.push_str(",\"errors\":[");
        for (i, err) in self.errors.iter().enumerate() {
            if i > 0 { json.push(','); }
            json.push_str("{\"type\":\"");
            json.push_str(&escaped_json(&err.type_));
            json.push_str("\",\"message\":\"");
            json.push_str(&escaped_json(&err.message));
            json.push_str("\",\"position\":");
            json.push_str(&err.position.to_string());
            json.push_str(",\"context\":\"");
            json.push_str(&escaped_json(&err.context));
            json.push_str("\"}");
        }
        json.push_str("]}");
        json
    }
}

fn unwrap_text(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find("\\text{") {
        out.push_str(&rest[..i]);
        let after = i + 6;
        if let Some(end) = rest[after..].find('}') {
            out.push_str(&rest[after..after + end]);
            rest = &rest[after + end + 1..];
        } else {
            out.push_str(&rest[i..]);
            return out;
        }
    }
    out.push_str(rest);
    out
}

fn escaped_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

// ====== WASM Exports ======

#[no_mangle]
pub extern "C" fn convert(input: *const u8, len: usize, display: bool) -> *mut c_char {
    if input.is_null() || len == 0 {
        let result = "{\"output\":\"\",\"errors\":[]}";
        return CString::new(result).unwrap().into_raw();
    }

    let input_slice = unsafe { std::slice::from_raw_parts(input, len) };
    let mut cvt = Converter::new(input_slice, false);
    cvt.convert_inner();

    if display {
        let mut wrapped = String::from("$$");
        wrapped.push_str(&cvt.output);
        wrapped.push_str("$$");
        cvt.output = wrapped;
    } else {
        let mut wrapped = String::from("$");
        wrapped.push_str(&cvt.output);
        wrapped.push('$');
        cvt.output = wrapped;
    }

    let json = cvt.to_json(display);
    CString::new(json).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}

// ====== Memory allocator exports ======

#[no_mangle]
pub extern "C" fn allocate(size: usize) -> *mut u8 {
    let layout = Layout::from_size_align(size, 1).unwrap();
    unsafe { alloc(layout) }
}

#[no_mangle]
pub extern "C" fn deallocate(ptr: *mut u8, size: usize) {
    let layout = Layout::from_size_align(size, 1).unwrap();
    unsafe { dealloc(ptr, layout) }
}

// ====== Tests ======

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_expr() {
        let input = b"x + y = 3";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "x + y = 3");
        assert!(cvt.errors.is_empty());
    }

    #[test]
    fn test_math_func() {
        let input = b"sin(x)";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\sin(x)");
    }

    #[test]
    fn test_fraction() {
        let input = b"$br x+1/x-1#";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\frac{x+1}{x-1}");
    }

    #[test]
    fn test_sqrt() {
        let input = b"$wt x+1#";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\sqrt{x+1}");
    }

    #[test]
    fn test_greek() {
        let input = b"\xce\xbb"; // λ
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\lambda");
    }

    #[test]
    fn test_greek_mu_prefix() {
        let input = b"\xce\xbca"; // μa (Greek mu)
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\alpha");

        let input = b"\xc2\xb5a"; // µa (micro sign)
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\alpha");
    }

    #[test]
    fn test_nested_greek_mu() {
        let input = b"$br 32/ $wt12+1*\xc2\xb5a";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\frac{32}{\\sqrt{12+1\\times\\alpha}}");

        let input = b"$br 32/ $wt12+1*\xce\xbca";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\frac{32}{\\sqrt{12+1\\times\\alpha}}");
    }

    #[test]
    fn test_fraction_with_unclosed_index() {
        let input = b"$br x^2/3#";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\frac{x^{2}}{3}");
    }

    #[test]
    fn test_simple_fraction_with_trailing_symbol() {
        let input = b"3/25b";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\frac{3}{25}b");
    }

    #[test]
    fn test_superscript() {
        let input = b"x^2#";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "x^{2}");
    }

    #[test]
    fn test_subscript() {
        let input = b"x_1#";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "x_{1}");
    }

    #[test]
    fn test_dollar_codes() {
        let input = b"$R";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\mathbb{R}");
    }

    #[test]
    fn test_text_detection() {
        let input = b"hello world";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\text{hello world}");
    }

    #[test]
    fn test_greek_pi_code() {
        let input = b"\xc2\xb5p";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\pi");
    }

    #[test]
    fn test_greek_capital_codes() {
        let input = b"\xc2\xb5G\xc2\xb5D";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\Gamma\\Delta");
    }

    #[test]
    fn test_mu_applies_to_next_letter_only() {
        let input = b"\xc2\xb5ax";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\alphax");
    }

    #[test]
    fn test_number_sets() {
        let input = b"$N$Z$Q$R$C";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\mathbb{N}\\mathbb{Z}\\mathbb{Q}\\mathbb{R}\\mathbb{C}");
    }

    #[test]
    fn test_math_function_names() {
        let input = b"lim log arcsin cosh det Pr Re Im";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\lim \\log \\arcsin \\cosh \\det \\Pr \\Re \\Im");
    }

    #[test]
    fn test_function_with_greek_argument() {
        let input = b"sin \xc2\xb5a";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\sin \\alpha");
    }

    #[test]
    fn test_fraction_with_terms() {
        let input = b"$br 24+35/12*8#";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\frac{24+35}{12\\times8}");
    }

    #[test]
    fn test_fraction_with_spatievuller() {
        let input = b"$br tan\xc2\xa8\xc2\xb5a-1/sin\xc2\xa8\xc2\xb5a#";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\frac{\\tan \\alpha-1}{\\sin \\alpha}");
    }

    #[test]
    fn test_fraction_with_parentheses() {
        let input = b"$br (a+b)/(a-b)#";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\frac{(a+b)}{(a-b)}");
    }

    #[test]
    fn test_root_of_single_digit() {
        let input = b"$wt 9";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\sqrt{9}");
    }

    #[test]
    fn test_root_with_numeric_index() {
        let input = b"^^3#$wt 8";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\sqrt[3]{8}");
    }

    #[test]
    fn test_nested_roots() {
        let input = b"$wt 2+$wt 3##";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\sqrt{2+\\sqrt{3}}");
    }

    #[test]
    fn test_exponent_without_closer() {
        let input = b"x^2y";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "x^{2}y");
    }

    #[test]
    fn test_optional_closer_subscripts() {
        let input = b"x_1#, x_2";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "x_{1}, x_{2}");
    }

    #[test]
    fn test_nested_sub_and_superscripts() {
        let input = b"a_i+1#^j+1#(x)";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "a_{i+1}^{j+1}(x)");
    }

    #[test]
    fn test_left_super_log() {
        let input = b"^^a#log x";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\log_{a} x");
    }

    #[test]
    fn test_integral_with_limits() {
        let input = b"$int_a#^b#f(x)dx";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\int_{a}^{b}f(x)\\text{dx}");
    }

    #[test]
    fn test_line_form_arc() {
        let input = b"$bgb XY#";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\overline{\\text{XY}}");
    }

    #[test]
    fn test_line_form_wavy() {
        let input = b"$slgb p+q#";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\tilde{p+q}");
    }

    #[test]
    fn test_line_form_angle() {
        let input = b"$hkb AB#";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\hat{\\text{AB}}");
    }

    #[test]
    fn test_line_form_vector() {
        let input = b"$plrb AB#";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\vec{\\text{AB}}");
    }

    #[test]
    fn test_line_form_left_arrow() {
        let input = b"$pllb AB#";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\overleftarrow{\\text{AB}}");
    }

    #[test]
    fn test_formula_line_break() {
        let input = b"a+b\nc+d";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "a + b \\\\ c + d");
    }

    #[test]
    fn test_percent_and_promille() {
        let input = b"50%$%%";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "50%\\%\\%");
    }

    #[test]
    fn test_relation_codes() {
        let input = b"$<=$>=$==";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\leq \\geq \\equiv");
    }

    #[test]
    fn test_negation_codes() {
        let input = b"$nt=$nt<$nt>$nt|$ntelm$ntdlv$nt//$ntldr";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\neq \\nless \\ngtr \\nmid \\notin \\not\\subset \\nparallel \\not\\perp");
    }

    #[test]
    fn test_arrow_codes() {
        let input = b"$pllr$pldr$pldl$asa$plb$plo";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\leftrightarrow \\implies \\impliedby \\iff \\uparrow \\downarrow");
    }

    #[test]
    fn test_set_operation_codes() {
        let input = b"$elm$belm$dlv$bdlv$dsn$unie$vrs";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\in \\ni \\subset \\supset \\cap \\cup \\setminus");
    }

    #[test]
    fn test_geometry_codes() {
        let input = b"$3hk$4kt$rhk$par$hk$rh$O$bol";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\triangle \\square \\square \\Diamond \\angle \\measuredangle \\emptyset \\bullet");
    }

    #[test]
    fn test_analysis_codes() {
        let input = b"$int$krd$slg$ldr$inf";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\int \\#\\sim \\perp \\infty");
    }

    #[test]
    fn test_logic_codes() {
        let input = b"$en$of";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\land \\lor");
    }

    #[test]
    fn test_approx_code() {
        let input = b"\xc2\xb5p$=3";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\pi \\approx3");
    }

    #[test]
    fn test_triangle_similarity() {
        let input = b"$3hk ABC$slg$3hk DEF";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\triangle \\text{ABC}\\sim \\triangle \\text{DEF}");
    }

    #[test]
    fn test_equality_sign_is_not_approx() {
        let input = b"x = y";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "x = y");
    }

    #[test]
    fn test_simple_fraction() {
        let input = b"1/2";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\frac{1}{2}");
    }

    #[test]
    fn test_mixed_number() {
        let input = b"2\xc2\xa81/4";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "2\\frac{1}{4}");
    }

    #[test]
    fn test_greek_gamma_capital_code() {
        let input = b"\xc2\xb5g";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\Gamma");
    }

    #[test]
    fn test_greek_psi_omega_chi_codes() {
        let input = b"\xc2\xb5\xc3\xa7\xc2\xb5\xc3\xb6\xc2\xb5h";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\psi\\omega\\chi");
    }

    #[test]
    fn test_summation_symbol_with_limits() {
        let input = b"\xc2\xb5S_i\xc2\xa8=1#^i\xc2\xa8=6#t_i";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\sum_{i=1}^{i=6}t_{i}");
    }

    #[test]
    fn test_missing_arrow_codes() {
        let input = b"$plr$pll";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\to \\leftarrow");
    }

    #[test]
    fn test_congruence_code() {
        let input = b"$cgr";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\cong");
    }

    #[test]
    fn test_nested_fraction_with_double_bar() {
        let input = b"$br$br\xc2\xa81/2+3#//5#";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\frac{\\frac{1}{2+3}}{5}");
    }

    #[test]
    fn test_compound_nested_fraction() {
        let input = b"$br a+$br\xc2\xa82bc/d-e#+f//g/h-3#";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\frac{a+\\frac{2bc}{d-e}+f}{\\frac{g}{h}-3}");
    }

    #[test]
    fn test_fraction_of_indexed_roots() {
        let input = b"$br t/$wt\xc2\xa8a_p#^n##+$wt\xc2\xa8a_q#^n###";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\frac{t}{\\sqrt{a_{p}^{n}}+\\sqrt{a_{q}^{n}}}");
    }

    #[test]
    fn test_indexed_root_with_exponent() {
        let input = b"^^6#$wt a^2n+4##";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\sqrt[6]{a^{2n+4}}");
    }

    #[test]
    fn test_nested_subscript_with_exponent() {
        let input = b"a_x^2#+y#+b";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "a_{x^{2}+y} + b");
    }

    #[test]
    fn test_exponent_with_fraction() {
        let input = b"x^1/2#y";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "x^{\\frac{1}{2}}y");
    }

    #[test]
    fn test_decimal_comma() {
        let input = b"(2,5; 6,20)";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "(2,5; 6,20)");
    }

    #[test]
    fn test_negation_code() {
        let input = b"$nt";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\neg");
    }

    #[test]
    fn test_matrix_shape() {
        let input = b"$mat2.2\na  c\nb  d$#";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\begin{pmatrix}a&c\\\\b&d\\end{pmatrix}");
    }

    #[test]
    fn test_system_shape() {
        let input = b"$st2\n2x+3y =14\nx-2y =0$#";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\begin{cases}2x+3y=14\\\\x-2y=0\\end{cases}");
    }

    #[test]
    fn test_determinant_shape() {
        let input = b"$det2.2\na  c\nb  d$#";
        let mut cvt = Converter::new(input, false);
        cvt.convert_inner();
        assert_eq!(cvt.output, "\\begin{vmatrix}a&c\\\\b&d\\end{vmatrix}");
    }
}
