use ast::Identifier;
use std::str;

named!(
  pub identifier<Identifier>,
  map!(
    map_res!(re_bytes_find!(r"^[a-zA-Z_]\w*"), str::from_utf8),
    |name| Identifier {
      name: String::from(name)
    }
  )
);

#[cfg(test)]
mod test {
  use super::*;
  #[test]
  fn alphanumeric() {
    assert_eq!(
      identifier(&b"x"[..]),
      Ok((
        &b""[..],
        Identifier {
          name: String::from("x")
        }
      ))
    );

    assert_eq!(
      identifier(&b"y2"[..]),
      Ok((
        &b""[..],
        Identifier {
          name: String::from("y2")
        }
      ))
    );

    assert_eq!(
      identifier(&b"x_y_2"[..]),
      Ok((
        &b""[..],
        Identifier {
          name: String::from("x_y_2")
        }
      ))
    );
  }

  #[test]
  fn can_start_with_underscore() {
    assert_eq!(
      identifier(&b"__x"[..]),
      Ok((
        &b""[..],
        Identifier {
          name: String::from("__x")
        }
      ))
    );
  }

  #[test]
  fn cannot_start_with_number() {
    assert_ne!(
      identifier(&b"2x"[..]),
      Ok((
        &b""[..],
        Identifier {
          name: String::from("2x")
        }
      ))
    );
  }
}
