use ast::Literal;
use nom::{is_digit, is_hex_digit};
use std::i64;
use std::str;

named!(sign, recognize!(opt!(one_of!("+-"))));

#[allow(dead_code)]
fn is_bin_digit(chr: u8) -> bool {
    chr == b'0' || chr == b'1'
}

named!(dec_digits, take_while!(is_digit));
named!(hex_digits, take_while!(is_hex_digit));
named!(bin_digits, take_while1!(is_bin_digit));

named!(bin_literal, recognize!(do_parse!(sign >> bin_digits >> ())));

named!(
    decimal_literal,
    recognize!(do_parse!(sign >> dec_digits >> ()))
);

named!(hex_literal, recognize!(do_parse!(sign >> hex_digits >> ())));

named!(
    binary<i64>,
    map_res!(map_res!(bin_literal, str::from_utf8), |s| {
        i64::from_str_radix(s, 2)
    })
);

named!(
    decimal<i64>,
    map_res!(map_res!(decimal_literal, str::from_utf8), |s| {
        i64::from_str_radix(s, 10)
    })
);

named!(
    hexadecimal<i64>,
    map_res!(map_res!(hex_literal, str::from_utf8), |s| {
        i64::from_str_radix(s, 16)
    })
);

named!(
    integer<i64>,
    alt_complete!(
        preceded!(tag!("0b"), binary)
            | preceded!(tag!("0x"), hexadecimal)
            | preceded!(opt!(tag!("0d")), decimal)
    )
);

named!(
  pub number_literal<Literal>,
  map!(integer, |d| Literal::Number(d))
);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bin_literal() {
        assert_eq!(bin_literal(&b"0 "[..]), Ok((&b" "[..], &b"0"[..])));
        assert_eq!(bin_literal(&b"1 "[..]), Ok((&b" "[..], &b"1"[..])));
        assert_eq!(bin_literal(&b"10 "[..]), Ok((&b" "[..], &b"10"[..])));
    }

    #[test]
    fn test_decimal_literal() {
        assert_eq!(decimal_literal(&b"0 "[..]), Ok((&b" "[..], &b"0"[..])));
        assert_eq!(decimal_literal(&b"10 "[..]), Ok((&b" "[..], &b"10"[..])));
    }

    #[test]
    fn test_hex_literal() {
        assert_eq!(hex_literal(&b"0 "[..]), Ok((&b" "[..], &b"0"[..])));
        assert_eq!(hex_literal(&b"f "[..]), Ok((&b" "[..], &b"f"[..])));
        assert_eq!(hex_literal(&b"10 "[..]), Ok((&b" "[..], &b"10"[..])));
    }

    #[test]
    fn test_binary() {
        assert_eq!(binary(&b"0 "[..]), Ok((&b" "[..], 0)));
        assert_eq!(binary(&b"1 "[..]), Ok((&b" "[..], 1)));
        assert_eq!(binary(&b"10 "[..]), Ok((&b" "[..], 2)));
    }

    #[test]
    fn test_decimal() {
        assert_eq!(decimal(&b"0 "[..]), Ok((&b" "[..], 0)));
        assert_eq!(decimal(&b"10 "[..]), Ok((&b" "[..], 10)));
    }

    #[test]
    fn test_hexadecimal() {
        assert_eq!(hexadecimal(&b"0 "[..]), Ok((&b" "[..], 0)));
        assert_eq!(hexadecimal(&b"f "[..]), Ok((&b" "[..], 15)));
        assert_eq!(hexadecimal(&b"10 "[..]), Ok((&b" "[..], 16)));
    }

    #[test]
    fn test_integer() {
        assert_eq!(integer(&b"0 "[..]), Ok((&b" "[..], 0)));
        assert_eq!(integer(&b"100 "[..]), Ok((&b" "[..], 100)));
        assert_eq!(integer(&b"0xf "[..]), Ok((&b" "[..], 15)));
        assert_eq!(integer(&b"0b10 "[..]), Ok((&b" "[..], 2)));
        assert_eq!(integer(&b"0d10 "[..]), Ok((&b" "[..], 10)));
    }

    #[test]
    fn test_number_literal() {
        assert_eq!(
            number_literal(&b"2 "[..]),
            Ok((&b" "[..], Literal::Number(2)))
        )
    }
}
