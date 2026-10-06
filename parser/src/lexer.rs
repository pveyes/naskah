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
const PUNCT1: [&str; 19] = [
    "+", "-", "*", "/", "%", "^", ">", "<", "=", "(", ")", "{", "}", ",", ";", "[", "]", ".", ":",
];

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_ident_part(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
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
                if chars.get(i) == Some(&'.') && chars.get(i + 1).map_or(false, |d| d.is_ascii_digit())
                {
                    i += 1;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                }
                let text: String = chars[begin..i].iter().collect();
                text.parse::<f64>().unwrap()
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
            let begin = i;
            i += 1;
            let mut escaped = false;
            loop {
                match chars.get(i) {
                    None | Some('\n') => err!(start_line, start_col, "teks tidak ditutup dengan `\"`"),
                    Some(&d) => {
                        i += 1;
                        if escaped {
                            escaped = false;
                        } else if d == '\\' {
                            escaped = true;
                        } else if d == '"' {
                            break;
                        }
                    }
                }
            }
            col += i - begin;
            Tok::Str(chars[begin + 1..i - 1].iter().collect())
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
