use ast::LiteralValue;
use nom::{digit, hex_digit};
use std::i64;
use std::str;

named!(sign, recognize!(opt!(one_of!("+-"))));

fn is_bin_digit(byte: u8) -> bool {
  byte == b'0' || byte == b'1'
}

named!(bin_digit, take_while1!(is_bin_digit));

named!(
  integer_literal2,
  recognize!(do_parse!(sign >> bin_digit >> ()))
);

named!(
  integer_literal10,
  recognize!(do_parse!(sign >> digit >> ()))
);

named!(
  integer_literal16,
  recognize!(do_parse!(sign >> hex_digit >> ()))
);

named!(
  integer2<i64>,
  map_res!(map_res!(integer_literal2, str::from_utf8), |s| {
    i64::from_str_radix(s, 2)
  })
);

named!(
  integer10<i64>,
  map_res!(map_res!(integer_literal10, str::from_utf8), |s| {
    i64::from_str_radix(s, 10)
  })
);

named!(
  integer16<i64>,
  map_res!(map_res!(integer_literal16, str::from_utf8), |s| {
    i64::from_str_radix(s, 16)
  })
);

named!(
  integer<i64>,
  alt!(
    preceded!(tag!("0b"), integer2)
      | preceded!(opt!(tag!("0d")), integer10)
      | preceded!(tag!("0x"), integer16)
  )
);

named!(
  pub number_literal<LiteralValue>,
  map!(integer, |d| LiteralValue::Number(d))
);

macro_rules! assert_parser {
  ($parser:expr, $input:expr, $result:expr) => {
    assert_eq!($parser($input.as_bytes()), Ok((&b""[..], $result)));
  };
}

#[test]
fn it_works() {
  assert_parser!(integer, "0xf", 15)
}
