use ast::LiteralValue;

named!(
  pub boolean_literal<LiteralValue>,
  alt_complete!(
    map!(tag!("benar"), |_| LiteralValue::Boolean(true)) |
    map!(tag!("salah"), |_| LiteralValue::Boolean(false))
  )
);
