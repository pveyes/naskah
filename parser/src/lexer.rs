use std::fmt;

/// A syntax error with the position where it was found (1-based).
#[derive(PartialEq, Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "baris {}, kolom {}: {}", self.line, self.col, self.message)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Tok {
    Ident(String),
    Number(f64),
    /// Raw text between the quotes, escape sequences left as written.
    Str(String),
    Punct(&'static str),
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tok: Tok,
    pub line: usize,
    pub col: usize,
}

const PUNCT2: [&str; 4] = ["==", "!=", ">=", "<="];
const PUNCT1: [&str; 21] = [
    "+", "-", "*", "/", "%", "^", ">", "<", "=", "(", ")", "{", "}", ",", ";", "[", "]", ".", ":", "×", "÷",
];

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_ident_part(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

pub(crate) enum StringError {
    /// The closing `"` is missing.
    Unterminated,
    /// A `{` inside the text has no matching `}`.
    Brace,
}

/// `chars[start]` is the opening `"`. Returns the index just past the closing one.
/// `{ ... }` inside the text is an interpolation and may hold strings of its own.
pub(crate) fn scan_string(chars: &[char], start: usize) -> std::result::Result<usize, StringError> {
    let mut i = start + 1;
    loop {
        match chars.get(i) {
            None | Some('\n') => return Err(StringError::Unterminated),
            Some('\\') => i += 2,
            Some('"') => return Ok(i + 1),
            Some('{') => i = scan_braces(chars, i)?,
            Some(_) => i += 1,
        }
    }
}

/// `chars[start]` is a `{`. Returns the index just past the matching `}`.
pub(crate) fn scan_braces(chars: &[char], start: usize) -> std::result::Result<usize, StringError> {
    let mut depth = 0;
    let mut i = start;
    loop {
        match chars.get(i) {
            None | Some('\n') => return Err(StringError::Brace),
            Some('"') => i = scan_string(chars, i).map_err(|_| StringError::Brace)?,
            Some('{') => {
                depth += 1;
                i += 1;
            }
            Some('}') => {
                depth -= 1;
                i += 1;
                if depth == 0 {
                    return Ok(i);
                }
            }
            Some(_) => i += 1,
        }
    }
}

pub fn lex(src: &str) -> Result<Vec<Token>, ParseError> {
    let chars: Vec<char> = src.chars().collect();
    let mut tokens = Vec::new();
    let (mut i, mut line, mut col) = (0, 1, 1);

    macro_rules! err {
        ($line:expr, $col:expr, $($arg:tt)*) => {
            return Err(ParseError { message: format!($($arg)*), line: $line, col: $col })
        };
    }

    while i < chars.len() {
        let c = chars[i];
        let (start_line, start_col) = (line, col);

        if c == '\n' {
            i += 1;
            line += 1;
            col = 1;
            continue;
        }
        if c.is_whitespace() {
            i += 1;
            col += 1;
            continue;
        }
        // comment until end of line
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
                col += 1;
            }
            continue;
        }

        let tok = if c.is_ascii_digit() {
            let begin = i;
            let radix = match (c, chars.get(i + 1)) {
                ('0', Some('b')) => 2,
                ('0', Some('x')) => 16,
                ('0', Some('d')) => 10,
                _ => 0,
            };
            let value = if radix != 0 && chars.get(i + 2).map_or(false, |d| d.is_digit(radix)) {
                i += 2;
                let digits_start = i;
                while i < chars.len() && chars[i].is_digit(radix) {
                    i += 1;
                }
                let digits: String = chars[digits_start..i].iter().collect();
                match u64::from_str_radix(&digits, radix) {
                    Ok(n) => n as f64,
                    Err(_) => err!(start_line, start_col, "angka terlalu besar"),
                }
            } else {
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let next_is_digit = |at: usize| chars.get(at).map_or(false, |d| d.is_ascii_digit());
                if chars.get(i) == Some(&'.') && next_is_digit(i + 1) {
                    err!(
                        start_line,
                        start_col,
                        "{}",
                        "angka desimal ditulis dengan koma, misalnya 1,5 dan bukan 1.5"
                    );
                }
                // `1,5` is one number: a comma between digits with no space is a decimal point
                if chars.get(i) == Some(&',') && next_is_digit(i + 1) {
                    i += 1;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                    // `1,2,3` could be a list or a number: do not guess
                    if chars.get(i) == Some(&',') && next_is_digit(i + 1) {
                        let mut end = i;
                        while end < chars.len() && (chars[end].is_ascii_digit() || chars[end] == ',') {
                            end += 1;
                        }
                        let written: String = chars[begin..end].iter().collect();
                        err!(
                            start_line,
                            start_col,
                            "angka `{}` bisa dibaca dua cara: beri spasi setelah koma pemisah (1, 2, 3), atau tulis desimal dengan satu koma saja (1,5)",
                            written
                        );
                    }
                }
                let text: String = chars[begin..i].iter().collect();
                text.replace(',', ".").parse::<f64>().unwrap()
            };
            if chars.get(i).map_or(false, |&d| is_ident_part(d)) {
                err!(start_line, start_col, "angka tidak valid");
            }
            col += i - begin;
            Tok::Number(value)
        } else if is_ident_start(c) {
            let begin = i;
            while i < chars.len() && is_ident_part(chars[i]) {
                i += 1;
            }
            col += i - begin;
            Tok::Ident(chars[begin..i].iter().collect())
        } else if c == '"' {
            let end = match scan_string(&chars, i) {
                Ok(end) => end,
                Err(StringError::Unterminated) => {
                    err!(start_line, start_col, "{}", "teks tidak ditutup dengan `\"`")
                }
                Err(StringError::Brace) => err!(
                    start_line,
                    start_col,
                    "{}",
                    "tanda `{` di dalam teks tidak ditutup, tulis `\\{` untuk kurung biasa"
                ),
            };
            let raw: String = chars[i + 1..end - 1].iter().collect();
            col += end - i;
            i = end;
            Tok::Str(raw)
        } else {
            let two: String = chars[i..chars.len().min(i + 2)].iter().collect();
            let one = c.to_string();
            if let Some(p) = PUNCT2.iter().find(|p| **p == two) {
                i += 2;
                col += 2;
                Tok::Punct(p)
            } else if let Some(p) = PUNCT1.iter().find(|p| **p == one) {
                i += 1;
                col += 1;
                Tok::Punct(p)
            } else {
                err!(start_line, start_col, "karakter `{}` tidak dikenal", c)
            }
        };

        tokens.push(Token { tok, line: start_line, col: start_col });
    }

    tokens.push(Token { tok: Tok::Eof, line, col });
    Ok(tokens)
}
