use super::number::number_literal;
use ast::Literal;
use std::str;

named!(
    boolean<bool>,
    alt!(map!(tag!("benar"), |_| true) | map!(tag!("salah"), |_| false))
);

named!(
  pub boolean_literal<Literal>,
  map!(boolean, |b| Literal::Boolean(b))
);

named!(pub null_literal<Literal>, map!(tag!("kosong"), |_| Literal::Null));

named!(
    string_literal<Literal>,
    do_parse!(
        tag!("\"")
            >> s: map_res!(take_until!("\""), str::from_utf8)
            >> tag!("\"")
            >> (Literal::String(String::from(s)))
    )
);

named!(
  pub literal<Literal>,
  alt_complete!(boolean_literal | number_literal | string_literal | null_literal)
);

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn boolean() {
        assert_eq!(
            literal(&b"benar"[..]),
            Ok((&b""[..], Literal::Boolean(true)))
        );

        assert_eq!(
            literal(&b"salah"[..]),
            Ok((&b""[..], Literal::Boolean(false)))
        );
    }

    #[test]
    fn string() {
        assert_eq!(
            literal(&b"\"p23u08rfwi\""[..]),
            Ok((&b""[..], Literal::String(String::from("p23u08rfwi"))))
        );
    }

    #[test]
    fn number() {
        assert_eq!(literal(&b"2 "[..]), Ok((&b" "[..], Literal::Number(2))));
    }

    #[test]
    fn null() {
        assert_eq!(literal(&b"kosong"[..]), Ok((&b""[..], Literal::Null)));
    }
}
