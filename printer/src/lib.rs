extern crate parser;

mod js;

use parser::parse;

pub fn to_js(s: String) -> String {
    let naskah_ast = parse(&s);
    match naskah_ast {
        Ok(ast) => js::print(ast),
        Err(_) => String::from("salah sintaks"),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ext_single() {
        let js = to_js(String::from("variabel x = null;\n"));
        assert_eq!(js, String::from("var x = null;\n"));
    }

    #[test]
    fn ext_multi() {
        let js = to_js(String::from("variabel x = null;\nvariabel y = benar;\n"));
        assert_eq!(js, String::from("var x = null;\nvar y = true;\n"));
    }
}
