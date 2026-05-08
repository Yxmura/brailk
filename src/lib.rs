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

// GREEK_LOWER: single-char Greek codes
const GREEK_LOWER_KEYS: [char; 25] = [
    'a', 'b', 'g', 'd', 'e', 'z', 'ä', 'h', 'ô', 'q', 'i', 'k',
    'l', 'm', 'n', 'x', 'o', 'p', 'r', 's', 't', 'y', 'f', 'c', 'j',
];
const GREEK_LOWER_VALS: [&str; 25] = [
    "\\alpha", "\\beta", "\\gamma", "\\delta", "\\epsilon", "\\zeta",
    "\\eta", "\\eta", "\\theta", "\\theta", "\\iota", "\\kappa",
    "\\lambda", "\\mu", "\\nu", "\\xi", "\\omicron", "\\pi",
    "\\rho", "\\sigma", "\\tau", "\\upsilon", "\\phi", "\\chi", "\\chi",
];

const GREEK_UPPER_KEYS: [char; 11] = [
    'G', 'D', 'O', 'L', 'X', 'P', 'S', 'Y', 'F', 'W', 'V',
];
const GREEK_UPPER_VALS: [&str; 11] = [
    "\\Gamma", "\\Delta", "\\Theta", "\\Lambda",
    "\\Xi", "\\Pi", "\\Sigma", "\\Upsilon",
    "\\Phi", "\\Psi", "\\Omega",
];

// Dollar codes (sorted by length desc for longest match)
const DOLLAR_KEYS: [&str; 65] = [
    "<=>", "nt//", "ntldr", "pllr", "pldr", "pldl", "ntelm", "ntdlv",
    "pllb", "plrb", "plsb", "plso", "dlv=", "bdlv=", "nt=", "nt<", "nt>", "nt|",
    "%%", "krd", "+-", "-+", "==", "<=", ">=", "!=", "//", "elm", "belm",
    "dlv", "bdlv", "dsn", "unie", "vrs", "plb", "plo", "ldr", "slg", "cgr",
    "par", "rhk", "4kt", "3hk", "hk", "rh", "bol",
    "**", "na", "en", "of", "asa", "inf", "int", "N", "Z", "Q", "R", "C",
    "O", "A", "E", "d", "*", "/", "=",
];

const DOLLAR_VALS: [&str; 65] = [
    "\\iff", "\\nparallel", "\\not\\perp",
    "\\leftrightarrow", "\\implies", "\\impliedby",
    "\\notin", "\\not\\subset",
    "\\nwarrow", "\\nearrow", "\\nearrow", "\\searrow",
    "\\subseteq", "\\supseteq",
    "\\neq", "\\nless", "\\ngtr", "\\nmid",
    "\\%\\%", "\\#", "\\pm", "\\mp",
    "\\equiv", "\\leq", "\\geq", "\\neq", "\\parallel",
    "\\in", "\\ni", "\\subset", "\\supset", "\\cap", "\\cup", "\\setminus",
    "\\uparrow", "\\downarrow", "\\perp", "\\sim", "\\simeq",
    "\\Diamond", "\\square", "\\square", "\\triangle", "\\angle", "\\measuredangle",
    "\\bullet",
    "\\ast", "\\circ", "\\land", "\\lor", "\\iff", "\\infty", "\\int",
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

    // Collect sub/superscript content
    fn collect_sub_or_super(&mut self) -> String {
        // Search for closing #
        let mut search = self.pos;
        let mut depth = 0;
        let mut has_hash = false;
        while search < self.input.len() {
            let chunk = &self.input[search..];
            if chunk.starts_with(b"$br") || chunk.starts_with(b"$wt") {
                depth += 1;
                search += 3;
                continue;
            }
            if self.input[search] == b'#' {
                if depth == 0 { has_hash = true; break; }
                depth -= 1;
            }
            if (self.input[search] == b'^' || self.input[search] == b'_') && depth == 0 && search > self.pos {
                break;
            }
            search += 1;
        }

        if has_hash && search > self.pos {
            let mut collected = String::new();
            let mut d = 0;
            while self.pos < self.input.len() {
                if self.peek(0) == b'#' && d == 0 {
                    self.pos += 1;
                    break;
                }
                if self.remaining().starts_with(b"$br") || self.remaining().starts_with(b"$wt") {
                    d += 1;
                    collected.push_str(&String::from_utf8_lossy(&self.input[self.pos..self.pos + 3]));
                    self.pos += 3;
                    continue;
                }
                if self.peek(0) == b'#' && d > 0 {
                    d -= 1;
                    collected.push('#');
                    self.pos += 1;
                    continue;
                }
                collected.push(self.peek(0) as char);
                self.pos += 1;
            }
            let mut sub = Converter::new(collected.as_bytes(), true);
            sub.convert_inner();
            self.errors.extend(sub.errors);
            sub.output
        } else if self.peek(0) == b'{' {
            self.pos += 1;
            let mut depth = 1;
            let mut collected = String::new();
            while self.pos < self.input.len() && depth > 0 {
                let c = self.peek(0) as char;
                self.pos += 1;
                if c == '{' { depth += 1; }
                else if c == '}' { depth -= 1; if depth > 0 { collected.push(c); } continue; }
                if depth > 0 { collected.push(c); }
            }
            let mut sub = Converter::new(collected.as_bytes(), true);
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

    fn handle_greek(&mut self) {
        // μ or µ consumed already
        if self.pos >= self.input.len() {
            self.emit("\\mu");
            return;
        }
        let key = self.peek(0) as char;
        self.pos += 1;

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

    fn handle_dollar(&mut self) {
        self.pos += 1; // skip $
        let needs_lead = self.output.ends_with(|c: char| c.is_alphanumeric());

        // Check for $br (fraction)
        if self.starts_with("br") {
            if needs_lead { self.emit(" "); }
            self.pos += 2;
            if self.peek(0) == b' ' { self.pos += 1; }
            let num = self.collect_until_fraction_slash();
            let den = self.collect_until_hash();
            let mut num_cvt = Converter::new(num.as_bytes(), true);
            let mut den_cvt = Converter::new(den.as_bytes(), true);
            num_cvt.convert_inner();
            den_cvt.convert_inner();
            self.errors.extend(num_cvt.errors);
            self.errors.extend(den_cvt.errors);
            self.emit("\\frac{");
            self.emit(&num_cvt.output);
            self.emit("}{");
            self.emit(&den_cvt.output);
            self.emit("}");
            return;
        }

        // $wt (root)
        if self.starts_with("wt") {
            if needs_lead { self.emit(" "); }
            self.pos += 2;
            if self.peek(0) == b' ' || self.peek(0) == 0xA8 { self.pos += 1; }
            let rad = self.collect_until_hash_or_single();
            let mut rad_cvt = Converter::new(rad.as_bytes(), true);
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
                let mut inner = Converter::new(raw.as_bytes(), true);
                inner.convert_inner();
                self.errors.extend(inner.errors);
                self.emit(latex);
                self.emit("{");
                self.emit(&inner.output);
                self.emit("}");
                return;
            }
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

    fn collect_until_fraction_slash(&mut self) -> String {
        let mut depth = 0;
        let mut collected = String::new();
        while self.pos < self.input.len() {
            if self.remaining().starts_with(b"$br") || self.remaining().starts_with(b"$wt") {
                depth += 1;
                collected.push_str(&String::from_utf8_lossy(&self.input[self.pos..self.pos + 3]));
                self.pos += 3;
                continue;
            }
            let c = self.peek(0);
            if depth == 0 && c == b'/' {
                self.pos += 1;
                return collected;
            }
            if c == b'#' && depth > 0 {
                depth -= 1;
            }
            collected.push(c as char);
            self.pos += 1;
        }
        collected
    }

    fn collect_until_hash(&mut self) -> String {
        let mut depth = 0;
        let mut collected = String::new();
        while self.pos < self.input.len() {
            if self.remaining().starts_with(b"$br") || self.remaining().starts_with(b"$wt") {
                depth += 1;
                collected.push_str(&String::from_utf8_lossy(&self.input[self.pos..self.pos + 3]));
                self.pos += 3;
                continue;
            }
            let c = self.peek(0);
            if c == b'#' {
                if depth == 0 {
                    self.pos += 1;
                    return collected;
                }
                depth -= 1;
            }
            collected.push(c as char);
            self.pos += 1;
        }
        collected
    }

    fn collect_until_hash_or_single(&mut self) -> String {
        let mut search = self.pos;
        let mut depth = 0;
        let mut found_hash = false;
        while search < self.input.len() {
            let chunk = &self.input[search..];
            if chunk.starts_with(b"$br") || chunk.starts_with(b"$wt") {
                depth += 1;
                search += 3;
                continue;
            }
            if self.input[search] == b'#' {
                if depth == 0 { found_hash = true; break; }
                depth -= 1;
            }
            search += 1;
        }
        if found_hash {
            self.collect_until_hash()
        } else {
            let remaining = String::from_utf8_lossy(&self.input[self.pos..]).to_string();
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
            let c = self.peek(0) as char;
            if c.is_alphabetic() {
                if self.lookahead_math_func(self.pos).is_some() {
                    break;
                }
                self.pos += 1;
            } else if c == ' ' {
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
            if c == 0xBC || c == 0xB5 {
                self.pos += 1;
                self.handle_greek();
                continue;
            }
            if c == 0xC2 && self.pos + 1 < self.input.len() && self.input[self.pos + 1] == 0xB5 {
                self.pos += 2;
                self.handle_greek();
                continue;
            }

            // ¨ — spatievuller
            if c == 0xA8 || (c == 0xC2 && self.pos + 1 < self.input.len() && self.input[self.pos + 1] == 0xA8) {
                if c == 0xC2 { self.pos += 2; } else { self.pos += 1; }
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
                let mut idx_cvt = Converter::new(idx_raw.as_bytes(), true);
                idx_cvt.convert_inner();
                self.errors.extend(idx_cvt.errors);

                if self.starts_with("$wt") {
                    self.pos += 3;
                    if self.peek(0) == b' ' || self.peek(0) == 0xA8 { self.pos += 1; }
                    let rad_raw = self.collect_until_hash_or_single();
                    let mut rad_cvt = Converter::new(rad_raw.as_bytes(), true);
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

            // Operators (excluding = which is handled as \approx)
            if c == b'=' {
                if !self.compact && !self.output.ends_with(' ') {
                    self.emit(" ");
                }
                self.emit("\\approx");
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

            // / → \div (from dollar codes)
            if c == b'/' {
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
                if self.pos + 1 < self.input.len() && self.peek(1) != b' ' {
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
        assert_eq!(cvt.output, "x + y \\approx 3");
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
}
