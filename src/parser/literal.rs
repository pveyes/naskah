use super::boolean::boolean_literal;
use super::number::number_literal;
use ast::LiteralValue;
use std::str;

named!(
  null<LiteralValue>,
  map!(tag!("kosong"), |_| LiteralValue::Null)
);

named!(
  string_literal<LiteralValue>,
  do_parse!(
    tag!("\"")
      >> s: map_res!(take_until!("\""), str::from_utf8)
      >> (LiteralValue::String(String::from(s)))
  )
);

named!(
  pub literal<LiteralValue>,
  alt_complete!(boolean_literal | number_literal | string_literal | null)
);

#[cfg(test)]
mod test {
  use super::*;
  #[test]
  fn boolean() {
    assert_eq!(
      literal(&b"benar"[..]),
      Ok((&b""[..], LiteralValue::Boolean(true)))
    );

    assert_eq!(
      literal(&b"salah"[..]),
      Ok((&b""[..], LiteralValue::Boolean(false)))
    );
  }

  #[test]
  fn string() {
    assert_eq!(
      literal(&b"\"p23u08rfwi\""[..]),
      Ok((&b"\""[..], LiteralValue::String(String::from("p23u08rfwi"))))
    );
  }

  #[test]
  fn number() {
    assert_eq!(
      literal(&b"2 "[..]),
      Ok((&b" "[..], LiteralValue::Number(2)))
    );
  }

  #[test]
  fn null() {
    assert_eq!(literal(&b"kosong"[..]), Ok((&b""[..], LiteralValue::Null)));
  }
}
