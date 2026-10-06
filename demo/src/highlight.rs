//! Tiny tokenizer used to colour the playground editor and output.

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Lang {
    Naskah,
    JavaScript,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    Plain,
    Keyword,
    Literal,
    Number,
    Str,
    Function,
    Operator,
    Comment,
}

impl Kind {
    pub fn class(self) -> &'static str {
        match self {
            Kind::Plain => "",
            Kind::Keyword => "tok-keyword",
            Kind::Literal => "tok-literal",
            Kind::Number => "tok-number",
            Kind::Str => "tok-string",
            Kind::Function => "tok-function",
            Kind::Operator => "tok-operator",
            Kind::Comment => "tok-comment",
        }
    }
}

impl Lang {
    fn keywords(self) -> &'static [&'static str] {
        match self {
            Lang::Naskah => &[
                "misal", "konstan", "jika", "lain", "selama", "ulang", "berhenti", "lanjut",
                "fungsi", "hasilkan", "dan", "atau", "bukan", "untuk", "setiap", "dari", "sampai",
                "langkah", "dalam", "pilih", "kalau", "coba", "tangkap", "akhirnya", "lempar", "nanti",
                "tunggu", "kelas", "turunan", "buat", "baru", "ini", "induk",
            ],
            Lang::JavaScript => &[
                "var", "let", "const", "if", "else", "for", "while", "break", "continue",
                "function", "return", "of", "try", "catch", "finally", "throw", "async", "await", "class",
                "extends", "constructor", "new", "this", "super",
            ],
        }
    }

    fn literals(self) -> &'static [&'static str] {
        match self {
            Lang::Naskah => &["benar", "salah", "kosong"],
            Lang::JavaScript => &["true", "false", "null"],
        }
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '$'
}

fn is_ident_part(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

/// Index just past the string starting at `start`. An unterminated string
/// stops at the end of the line, so one stray quote does not colour the rest.
/// `{...}` in a Naskah string and `${...}` in a JS template hold code, which
/// may contain strings of its own.
fn string_end(lang: Lang, src: &str, start: usize) -> usize {
    let bytes = src.as_bytes();
    let quote = bytes[start];
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                i += 1 + src[i + 1..].chars().next().map_or(0, |c| c.len_utf8());
            }
            q if q == quote => return i + 1,
            b'{' if quote == b'"' && lang == Lang::Naskah => i = braces_end(lang, src, i),
            b'$' if quote == b'`' && bytes.get(i + 1) == Some(&b'{') => {
                i = braces_end(lang, src, i + 1)
            }
            b'\n' if quote != b'`' => return i,
            _ => i += 1,
        }
    }
    bytes.len()
}

/// Index just past the `}` matching the `{` at `start`.
fn braces_end(lang: Lang, src: &str, start: usize) -> usize {
    let bytes = src.as_bytes();
    let mut depth = 0;
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                i += 1;
                if depth == 0 {
                    return i;
                }
            }
            b'"' | b'`' | b'\'' => i = string_end(lang, src, i),
            b'\n' => return i,
            _ => i += 1,
        }
    }
    bytes.len()
}

/// Split `src` into contiguous tokens. Concatenating the text of every
/// token always yields `src` back.
pub fn tokenize(lang: Lang, src: &str) -> Vec<(Kind, &str)> {
    let mut tokens = Vec::new();
    let mut chars = src.char_indices().peekable();

    while let Some(&(start, c)) = chars.peek() {
        let mut end = start + c.len_utf8();
        let kind;

        if c.is_whitespace() {
            kind = Kind::Plain;
            while let Some(&(i, c)) = chars.peek() {
                if !c.is_whitespace() {
                    break;
                }
                end = i + c.len_utf8();
                chars.next();
            }
        } else if c == '/' && src[start..].starts_with("//") {
            kind = Kind::Comment;
            while let Some(&(i, d)) = chars.peek() {
                if d == '\n' {
                    break;
                }
                end = i + d.len_utf8();
                chars.next();
            }
        } else if c == '"' || c == '\'' || (c == '`' && lang == Lang::JavaScript) {
            kind = Kind::Str;
            end = string_end(lang, src, start);
            while let Some(&(i, _)) = chars.peek() {
                if i >= end {
                    break;
                }
                chars.next();
            }
        } else if c.is_ascii_digit() {
            kind = Kind::Number;
            while let Some(&(i, d)) = chars.peek() {
                if !(d.is_ascii_alphanumeric() || d == '.') {
                    break;
                }
                end = i + d.len_utf8();
                chars.next();
            }
        } else if is_ident_start(c) {
            chars.next();
            while let Some(&(i, d)) = chars.peek() {
                if !is_ident_part(d) {
                    break;
                }
                end = i + d.len_utf8();
                chars.next();
            }
            let word = &src[start..end];
            kind = if lang.keywords().contains(&word) {
                Kind::Keyword
            } else if lang.literals().contains(&word) {
                Kind::Literal
            } else if src[end..].starts_with('(') {
                Kind::Function
            } else {
                Kind::Plain
            };
        } else {
            kind = Kind::Operator;
            chars.next();
        }

        tokens.push((kind, &src[start..end]));
    }

    tokens
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn roundtrip() {
        let src = "misal x = 2 + 2;\njika x benar {\n  menang(\"a\\\"b\");\n}\n";
        let joined: String = tokenize(Lang::Naskah, src).iter().map(|t| t.1).collect();
        assert_eq!(joined, src);
    }

    #[test]
    fn naskah_tokens() {
        let toks = tokenize(Lang::Naskah, "misal x = benar;");
        assert_eq!(toks[0], (Kind::Keyword, "misal"));
        assert!(toks.contains(&(Kind::Literal, "benar")));
    }

    #[test]
    fn js_tokens() {
        let toks = tokenize(Lang::JavaScript, "var y = x === 0x1F; f('s')");
        assert_eq!(toks[0], (Kind::Keyword, "var"));
        assert!(toks.contains(&(Kind::Number, "0x1F")));
        assert!(toks.contains(&(Kind::Function, "f")));
        assert!(toks.contains(&(Kind::Str, "'s'")));
    }

    #[test]
    fn comments() {
        let toks = tokenize(Lang::Naskah, "x / 2; // bagi dua\ny");
        assert!(toks.contains(&(Kind::Comment, "// bagi dua")));
        assert!(toks.contains(&(Kind::Operator, "/")));
        assert_eq!(toks.last(), Some(&(Kind::Plain, "y")));
    }

    #[test]
    fn new_keywords() {
        let toks = tokenize(Lang::Naskah, "fungsi f() { hasilkan a dan bukan b; }");
        for word in &["fungsi", "hasilkan", "dan", "bukan"] {
            assert!(toks.contains(&(Kind::Keyword, *word)), "{}", word);
        }
        let toks = tokenize(Lang::Naskah, "kelas A turunan B { buat() { induk(); ini.x = baru C(); } }");
        for word in &["kelas", "turunan", "buat", "induk", "ini", "baru"] {
            assert!(toks.contains(&(Kind::Keyword, *word)), "{}", word);
        }
        let toks = tokenize(Lang::Naskah, "coba { lempar x; } tangkap e { } akhirnya { } nanti fungsi f() { tunggu g(); }");
        for word in &["coba", "lempar", "tangkap", "akhirnya", "nanti", "tunggu"] {
            assert!(toks.contains(&(Kind::Keyword, *word)), "{}", word);
        }
        let toks = tokenize(Lang::Naskah, "untuk setiap x dalam y { } pilih z { kalau 1 { } lain { } }");
        for word in &["untuk", "setiap", "dalam", "pilih", "kalau", "lain"] {
            assert!(toks.contains(&(Kind::Keyword, *word)), "{}", word);
        }
    }

    #[test]
    fn interpolations_stay_inside_the_string() {
        let src = "tulis(\"a {f(\"}\")} b\", 1);";
        let toks = tokenize(Lang::Naskah, src);
        assert!(toks.contains(&(Kind::Str, "\"a {f(\"}\")} b\"")), "{:?}", toks);
        assert!(toks.contains(&(Kind::Number, "1")));
        // an escaped brace opens nothing
        let toks = tokenize(Lang::Naskah, "\"\\{a\" 2");
        assert!(toks.contains(&(Kind::Str, "\"\\{a\"")), "{:?}", toks);
    }

    #[test]
    fn javascript_templates() {
        let src = "x = `a ${f(`b ${c}`)} d`; y";
        let toks = tokenize(Lang::JavaScript, src);
        assert!(toks.contains(&(Kind::Str, "`a ${f(`b ${c}`)} d`")), "{:?}", toks);
        assert_eq!(toks.last(), Some(&(Kind::Plain, "y")));
        let joined: String = toks.iter().map(|t| t.1).collect();
        assert_eq!(joined, src);
    }

    #[test]
    fn unterminated_string_and_unicode() {
        let src = "\"abc é";
        let joined: String = tokenize(Lang::Naskah, src).iter().map(|t| t.1).collect();
        assert_eq!(joined, src);
    }
}
