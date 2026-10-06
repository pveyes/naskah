extern crate parser;

mod js;

use parser::parse;

pub fn to_js(s: String) -> String {
    let naskah_ast = parse(&s);
    match naskah_ast {
        Ok(ast) => js::print(ast),
        Err(e) => format!("// Salah sintaks di {}", e),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ext_single() {
        let js = to_js(String::from("misal x = null;\n"));
        assert_eq!(js, String::from("let x = null;\n"));
    }

    #[test]
    fn ext_multi() {
        let js = to_js(String::from("misal x = null;\nmisal y = benar;\n"));
        assert_eq!(js, String::from("let x = null;\nlet y = true;\n"));
    }

    fn js(src: &str) -> String {
        to_js(String::from(src))
    }

    #[test]
    fn const_and_builtins() {
        assert_eq!(
            js("konstan nama = \"Budi\";\ntulis(\"halo\", nama);\n"),
            "const nama = \"Budi\";\nconsole.log(\"halo\", nama);\n"
        );
        assert_eq!(js("misal x = tanya(\"umur?\");"), "let x = prompt(\"umur?\");\n");
    }

    #[test]
    fn functions_and_loops() {
        assert_eq!(
            js("fungsi jumlah(a, b) {\nkembali a + b;\n}\nselama benar {\nberhenti;\n}\n"),
            "function jumlah(a, b) {\n  return a + b;\n}\nwhile (true) {\n  break;\n}\n"
        );
    }

    #[test]
    fn if_else_chain() {
        assert_eq!(
            js("jika a benar {\nlanjut;\n} lain jika b {\n} lain {\n}"),
            "if (a === true) {\n  continue;\n} else if (b) {\n} else {\n}\n"
        );
    }

    #[test]
    fn logic_operators() {
        assert_eq!(js("x = a dan b atau c;"), "x = a && b || c;\n");
        assert_eq!(js("x = a dan (b atau c);"), "x = a && (b || c);\n");
        assert_eq!(js("x = bukan a == b;"), "x = !(a === b);\n");
        assert_eq!(js("x = bukan a dan b;"), "x = !a && b;\n");
    }

    #[test]
    fn parentheses_follow_precedence() {
        assert_eq!(js("x = 1 + 2 * 3;"), "x = 1 + 2 * 3;\n");
        assert_eq!(js("x = (1 + 2) * 3;"), "x = (1 + 2) * 3;\n");
        assert_eq!(js("x = 1 - (2 - 3);"), "x = 1 - (2 - 3);\n");
        assert_eq!(js("x = (1 - 2) - 3;"), "x = 1 - 2 - 3;\n");
        assert_eq!(js("x = 2 ^ 3 ^ 2;"), "x = 2 ** 3 ** 2;\n");
        assert_eq!(js("x = (2 ^ 3) ^ 2;"), "x = (2 ** 3) ** 2;\n");
        // JS forbids a unary operand directly left of `**`
        assert_eq!(js("x = (-2) ^ 2;"), "x = (-2) ** 2;\n");
        assert_eq!(js("x = -2 ^ 2;"), "x = -(2 ** 2);\n");
        assert_eq!(js("x = - -y;"), "x = -(-y);\n");
        assert_eq!(js("x = 1 > 2 + 3;"), "x = 1 > 2 + 3;\n");
    }

    #[test]
    fn numbers() {
        assert_eq!(js("x = 0xff + 1.5 + 0b11;"), "x = 255 + 1.5 + 3;\n");
    }

    #[test]
    fn comments_are_dropped() {
        assert_eq!(js("// catatan\nmisal x = 1; // satu\n"), "let x = 1;\n");
    }

    #[test]
    fn syntax_error_is_reported_as_comment() {
        assert_eq!(
            js("misal x = ;"),
            "// Salah sintaks di baris 1, kolom 11: ekspresi tidak lengkap, ditemukan `;`"
        );
    }
}
