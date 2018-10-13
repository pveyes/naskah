use ast::LiteralValue;
use nom::{is_digit, is_hex_digit};
use std::i64;
use std::str;

named!(sign, recognize!(opt!(one_of!("+-"))));

#[allow(dead_code)]
fn is_bin_digit(chr: u8) -> bool {
  chr == b'0' || chr == b'1'
}

named!(digits, take_while!(is_digit));
named!(hex_digits, take_while!(is_hex_digit));
named!(bin_digits, take_while!(is_bin_digit));

named!(
  integer_literal2,
  recognize!(do_parse!(sign >> bin_digits >> ()))
);

named!(
  integer_literal10,
  recognize!(do_parse!(sign >> digits >> ()))
);

named!(
  integer_literal16,
  recognize!(do_parse!(sign >> hex_digits >> ()))
);

named!(
  binary<i64>,
  map_res!(map_res!(integer_literal2, str::from_utf8), |s| {
    i64::from_str_radix(s, 2)
  })
);

named!(
  decimal<i64>,
  map_res!(map_res!(integer_literal10, str::from_utf8), |s| {
    i64::from_str_radix(s, 10)
  })
);

named!(
  hexadecimal<i64>,
  map_res!(map_res!(integer_literal16, str::from_utf8), |s| {
    i64::from_str_radix(s, 16)
  })
);

named!(
  integer<i64>,
  alt!(
    preceded!(tag!("0b"), binary)
      | preceded!(tag!("0x"), hexadecimal)
      | preceded!(opt!(tag!("0d")), decimal)
  )
);

named!(
  pub number_literal<LiteralValue>,
  map!(integer, |d| LiteralValue::Number(d))
);

#[allow(unused_macros)]
macro_rules! assert_parser {
  ($parser:expr, $input:expr, $result:expr) => {
    assert_eq!($parser($input.as_bytes()), Ok((&b";"[..], $result)));
  };
}

#[test]
fn test_binary() {
  assert_parser!(integer, "0b0;", 0);
  assert_parser!(integer, "0b1;", 1);
  assert_parser!(integer, "0b01;", 1);
  assert_parser!(integer, "0b10;", 2);
}

#[test]
fn test_decimal() {
  assert_parser!(integer, "15;", 15);
  assert_parser!(integer, "0d15;", 15);
}

#[test]
fn test_hexadecimal() {
  assert_parser!(integer, "0xf;", 15);
  assert_parser!(integer, "0x11;", 17);
}
