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
            js("fungsi jumlah(a, b) {\nhasilkan a + b;\n}\nselama benar {\nberhenti;\n}\n"),
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

    #[test]
    fn lists_objects_and_members() {
        assert_eq!(
            js("misal o = { nama: \"Budi\", panjang: 3, \"a b\": [1, 2] };"),
            "let o = { nama: \"Budi\", length: 3, \"a b\": [1, 2] };\n"
        );
        assert_eq!(js("x = [] == {};"), "x = [] === {};\n");
        assert_eq!(
            js("x = daftar.panjang + daftar[0];"),
            "x = daftar.length + daftar[0];\n"
        );
        assert_eq!(js("daftar.tambah(4);"), "daftar.push(4);\n");
        assert_eq!(js("x = daftar.gabung(\", \").balik();"), "x = daftar.join(\", \").reverse();\n");
        assert_eq!(js("a[0] = b.c = a[1];"), "a[0] = b.c = a[1];\n");
        assert_eq!(js("f(1)(2);"), "f(1)(2);\n");
    }

    #[test]
    fn member_operands_get_parentheses() {
        assert_eq!(js("x = (a + b).panjang;"), "x = (a + b).length;\n");
        assert_eq!(js("x = (5).foo;"), "x = (5).foo;\n");
        assert_eq!(js("x = (-a)[0];"), "x = (-a)[0];\n");
        // a statement starting with `{` would be parsed as a block by JS
        assert_eq!(js("({ a: 1 }).a;"), "({ a: 1 }.a);\n");
    }

    #[test]
    fn for_loops() {
        assert_eq!(
            js("untuk i dari 1 sampai 3 {\ntulis(i);\n}"),
            "for (let i = 1; i <= 3; i++) {\n  console.log(i);\n}\n"
        );
        assert_eq!(
            js("untuk i dari 10 sampai 0 langkah -2 {\n}"),
            "for (let i = 10; i >= 0; i += -2) {\n}\n"
        );
        assert_eq!(
            js("untuk i dari 0 sampai 9 langkah 3 {\n}"),
            "for (let i = 0; i <= 9; i += 3) {\n}\n"
        );
        assert_eq!(
            js("untuk setiap x dalam [1, 2] {\nlanjut;\n}"),
            "for (let x of [1, 2]) {\n  continue;\n}\n"
        );
    }

    #[test]
    fn switch_is_an_if_chain() {
        assert_eq!(
            js("pilih x {\nkalau 1, 2 {\ntulis(\"a\");\n}\nkalau 3 {\n}\nlain {\nberhenti;\n}\n}"),
            "if (x === 1 || x === 2) {\n  console.log(\"a\");\n} else if (x === 3) {\n} else {\n  break;\n}\n"
        );
    }

    #[test]
    fn switch_on_an_expression_evaluates_it_once() {
        assert_eq!(
            js("pilih f() {\nkalau 1 {\n}\n}"),
            "{\n  const _pilih0 = f();\n  if (_pilih0 === 1) {\n  }\n}\n"
        );
        // nested switches get distinct names
        assert_eq!(
            js("pilih f() {\nkalau 1 {\npilih g() {\nkalau 2 {\n}\n}\n}\n}"),
            "{\n  const _pilih0 = f();\n  if (_pilih0 === 1) {\n    {\n      const _pilih2 = g();\n      if (_pilih2 === 2) {\n      }\n    }\n  }\n}\n"
        );
    }

    #[test]
    fn bare_block_is_indented() {
        assert_eq!(
            js("jika x {\n{\nlanjut;\n}\n}"),
            "if (x) {\n  {\n    continue;\n  }\n}\n"
        );
    }
}
