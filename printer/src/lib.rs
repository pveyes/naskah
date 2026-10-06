extern crate parser;

mod js;

use parser::ast::Program;
use parser::{parse, parse_checked, ParseError};

/// JavaScript plus, for every `tulis(...)`, the line it was written on.
pub struct Transpiled {
    pub js: String,
    /// `(js_line, naskah_line)`, both 1-based. If a JS line holds several
    /// `tulis` calls, the first one is recorded.
    pub call_sites: Vec<(usize, usize)>,
    /// `(js_line, naskah_line)` for the first JS line of every statement, so a
    /// runtime error can be traced back to the line that was written.
    pub statement_lines: Vec<(usize, usize)>,
    /// What is wrong with the program, if anything. `js` then holds a comment saying so.
    pub error: Option<ParseError>,
}

pub fn transpile(s: &str) -> Transpiled {
    transpile_with(s, parse)
}

/// Like `transpile`, but a name that is used and never made is a syntax error. This is
/// what the playground uses, so a misspelled name is caught before the program runs.
pub fn transpile_checked(s: &str) -> Transpiled {
    transpile_with(s, parse_checked)
}

fn transpile_with(s: &str, read: fn(&str) -> Result<Program, ParseError>) -> Transpiled {
    match read(s) {
        Ok(ast) => extract_call_sites(&js::print(ast)),
        Err(e) => Transpiled {
            js: format!("// Salah sintaks di {}", e),
            call_sites: vec![],
            statement_lines: vec![],
            error: Some(e),
        },
    }
}

pub fn to_js(s: String) -> String {
    transpile(&s).js
}

/// Strip the line markers the printer left behind and collect them per JS line.
fn extract_call_sites(marked: &str) -> Transpiled {
    let mut js = String::new();
    let mut call_sites = Vec::new();
    let mut statement_lines = Vec::new();
    let is_marker = |c: char| c == js::SITE_START || c == js::STATEMENT_START;

    for (index, line) in marked.split('\n').enumerate() {
        if index > 0 {
            js.push('\n');
        }
        let (mut site_done, mut statement_done) = (false, false);
        let mut rest = line;
        while let Some(start) = rest.find(is_marker) {
            js.push_str(&rest[..start]);
            let opener = rest[start..].chars().next().unwrap();
            let (closer, done, list) = if opener == js::SITE_START {
                (js::SITE_END, &mut site_done, &mut call_sites)
            } else {
                (js::STATEMENT_END, &mut statement_done, &mut statement_lines)
            };
            let after = &rest[start + opener.len_utf8()..];
            let end = after.find(closer).expect("unterminated line marker");
            if !*done {
                list.push((index + 1, after[..end].parse().unwrap()));
                *done = true;
            }
            rest = &after[end + closer.len_utf8()..];
        }
        js.push_str(rest);
    }

    Transpiled { js, call_sites, statement_lines, error: None }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ext_single() {
        let js = to_js(String::from("misal x = kosong;\n"));
        assert_eq!(js, String::from("let x = null;\n"));
    }

    #[test]
    fn ext_multi() {
        let js = to_js(String::from("misal x = kosong;\nmisal y = benar;\n"));
        assert_eq!(js, String::from("let x = null;\nlet y = true;\n"));
    }

    fn js(src: &str) -> String {
        to_js(String::from(src))
    }

    #[test]
    fn asking_and_reading_numbers() {
        assert_eq!(
            js("misal umur = bilangan(tanya(\"Umur?\"))"),
            "let umur = __bilangan(prompt(\"Umur?\"));\n"
        );
        assert_eq!(
            js("misal umur = tanya(\"Umur?\", Tipe.Angka)"),
            "let umur = prompt(\"Umur?\", __tipe.Angka);\n"
        );
        // a name that merely contains the word is left alone
        assert_eq!(js("misal bilangan2 = 1"), "let bilangan2 = 1;\n");
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
            "function jumlah(a, b) {\n  return __tambah(a, b);\n}\nwhile (true) {\n  break;\n}\n"
        );
    }

    #[test]
    fn saat_without_braces() {
        assert_eq!(
            js("pilih x {\nsaat 1 / 2 tulis(\"a\")\nlain tulis(\"b\")\n}"),
            "if (x === 1 || x === 2) {\n  console.log(\"a\");\n} else {\n  console.log(\"b\");\n}\n"
        );
    }

    #[test]
    fn repeat_until() {
        assert_eq!(
            js("misal x = 0\nulang {\nx = x + 1\n} sampai x >= 5\ntulis(x)"),
            "let x = 0;\ndo {\n  x = __tambah(x, 1);\n} while (!(x >= 5));\nconsole.log(x);\n"
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
        assert_eq!(js("x = a bukan b;"), "x = a !== b;\n");
        assert_eq!(js("x = a adalah b dan c;"), "x = a === b && c;\n");
    }

    #[test]
    fn school_symbols_multiply_and_divide() {
        assert_eq!(js("x = 6 × 7;"), "x = 6 * 7;\n");
        assert_eq!(js("x = 10 ÷ 4 + 1;"), "x = 10 / 4 + 1;\n");
        assert_eq!(js("x = 1 + 2 × 3;"), "x = 1 + 2 * 3;\n");
        // `/` after `saat` means "atau", but `÷` is always a division
        assert_eq!(
            js("pilih x {\nsaat 8 ÷ 2 tulis(\"a\")\n}"),
            js("pilih x {\nsaat (8 / 2) tulis(\"a\")\n}")
        );
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
        assert_eq!(js("x = 0xff + 1,5 + 0b11;"), "x = 255 + 1.5 + 3;\n");
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
        assert_eq!(js("x = [] adalah {};"), "x = [] === {};\n");
        assert_eq!(
            js("x = daftar.panjang + daftar[0];"),
            "x = __tambah(daftar.length, daftar[0]);\n"
        );
        assert_eq!(js("daftar.tambah(4);"), "daftar.push(4);\n");
        assert_eq!(js("x = daftar.gabung(\", \").balik();"), "x = daftar.join(\", \").reverse();\n");
        assert_eq!(js("a[0] = b.c = a[1];"), "a[0] = b.c = a[1];\n");
        assert_eq!(js("f(1)(2);"), "f(1)(2);\n");
    }

    #[test]
    fn member_operands_get_parentheses() {
        assert_eq!(js("x = (a + b).panjang;"), "x = __tambah(a, b).length;\n");
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
            js("untuk setiap x dalam [1, 2] {\nlanjut;\n}"),
            "for (let x of [1, 2]) {\n  continue;\n}\n"
        );
    }

    #[test]
    fn switch_is_an_if_chain() {
        assert_eq!(
            js("pilih x {\nsaat 1 / 2 {\ntulis(\"a\");\n}\nsaat 3 {\n}\nlain {\nberhenti;\n}\n}"),
            "if (x === 1 || x === 2) {\n  console.log(\"a\");\n} else if (x === 3) {\n} else {\n  break;\n}\n"
        );
    }

    #[test]
    fn switch_on_an_expression_evaluates_it_once() {
        assert_eq!(
            js("pilih f() {\nsaat 1 {\n}\n}"),
            "{\n  const _pilih0 = f();\n  if (_pilih0 === 1) {\n  }\n}\n"
        );
        // nested switches get distinct names
        assert_eq!(
            js("pilih f() {\nsaat 1 {\npilih g() {\nsaat 2 {\n}\n}\n}\n}"),
            "{\n  const _pilih0 = f();\n  if (_pilih0 === 1) {\n    {\n      const _pilih2 = g();\n      if (_pilih2 === 2) {\n      }\n    }\n  }\n}\n"
        );
    }

    #[test]
    fn call_sites_map_js_lines_to_naskah_lines() {
        let t = transpile("misal x = 1;\ntulis(x);\n\njika x {\n  tulis(\"a\");\n}\n");
        assert_eq!(t.js, "let x = 1;\nconsole.log(x);\nif (x) {\n  console.log(\"a\");\n}\n");
        assert_eq!(t.call_sites, vec![(2, 2), (4, 5)]);
    }

    #[test]
    fn call_sites_use_the_first_call_on_a_line() {
        let t = transpile("\ntulis(tulis(1));");
        assert_eq!(t.js, "console.log(console.log(1));\n");
        assert_eq!(t.call_sites, vec![(1, 2)]);
    }

    #[test]
    fn only_tulis_has_a_call_site() {
        let t = transpile("f(1);\ntanya(\"x\");");
        assert_eq!(t.js, "f(1);\nprompt(\"x\");\n");
        assert!(t.call_sites.is_empty());
    }

    #[test]
    fn text_templates() {
        assert_eq!(
            js(r#"s = "Halo, {nama}! {a + 1}";"#),
            "s = `Halo, ${__teks(nama)}! ${__teks(__tambah(a, 1))}`;\n"
        );
        assert_eq!(js(r#"s = "{xs.panjang} item";"#), "s = `${__teks(xs.length)} item`;\n");
        // nested strings and calls inside an interpolation
        assert_eq!(
            js(r#"s = "a {f("x")} b";"#),
            "s = `a ${__teks(f(\"x\"))} b`;\n"
        );
        // templates inside templates
        assert_eq!(js(r#"s = "{"[{x}]"}";"#), "s = `${`[${__teks(x)}]`}`;\n");
    }

    #[test]
    fn template_text_is_escaped_for_javascript() {
        // a backtick in the text must not end the literal
        assert_eq!(js(r#"s = "a`b {c}";"#), "s = `a\\`b ${__teks(c)}`;\n");
        // `\{` is a literal brace, and `${` in the output must not start an interpolation
        assert_eq!(js(r#"s = "$\{x} {y}";"#), "s = `\\${x} ${__teks(y)}`;\n");
        assert_eq!(js(r#"s = "\{x} {y}";"#), "s = `{x} ${__teks(y)}`;\n");
        // a plain string keeps its escape
        assert_eq!(js(r#"s = "\{x}";"#), "s = \"\\{x}\";\n");
        assert_eq!(js(r#"s = "a } b";"#), "s = \"a } b\";\n");
    }

    #[test]
    fn async_and_await() {
        assert_eq!(
            js("fungsi ambil() {\ntunggu tunda(100);\nhasilkan 1;\n}\nmisal x = tunggu ambil();"),
            "async function ambil() {\n  await new Promise((resolve) => setTimeout(resolve, 100));\n  return 1;\n}\nlet x = await ambil();\n"
        );
        assert_eq!(js("x = (tunggu a).b;"), "x = (await a).b;\n");
        assert_eq!(js("x = tunggu a + b;"), "x = __tambah(await a, b);\n");
        assert_eq!(js("x = tunggu (a + b);"), "x = await __tambah(a, b);\n");
        // tunda is only the built-in sleep when called with one argument
        assert_eq!(js("tunda(1, 2);"), "tunda(1, 2);\n");
    }

    #[test]
    fn function_values_are_arrow_functions() {
        assert_eq!(
            js("xs.ubah(fungsi (x) {\nhasilkan x * 2;\n});"),
            "xs.map((x) => {\n  return x * 2;\n});\n"
        );
        assert_eq!(
            js("jika benar {\nxs.saring(fungsi (x) {\nhasilkan x;\n});\n}"),
            "if (true) {\n  xs.filter((x) => {\n    return x;\n  });\n}\n"
        );
        assert_eq!(js("f(fungsi () { });"), "f(() => {\n});\n");
        assert_eq!(
            js("f(fungsi () {\ntunggu g();\n});"),
            "f(async () => {\n  await g();\n});\n"
        );
        assert_eq!(
            js("misal g = fungsi (a, b) {\nhasilkan a;\n};"),
            "let g = (a, b) => {\n  return a;\n};\n"
        );
        // nested function values keep their own indentation
        assert_eq!(
            js("f(fungsi () {\ng(fungsi () {\nlanjut;\n});\n});"),
            "f(() => {\n  g(() => {\n    continue;\n  });\n});\n"
        );
    }

    #[test]
    fn list_helpers_and_errors_use_javascript_names() {
        assert_eq!(js("x = xs.urut().cari(f);"), "x = xs.sort().find(f);\n");
        assert_eq!(js("x = e.pesan;"), "x = __pesan(e);\n");
        // writing to it is just the property
        assert_eq!(js("e.pesan = \"baru\";"), "e.message = \"baru\";\n");
    }

    #[test]
    fn try_catch_finally_and_throw() {
        assert_eq!(
            js("coba {\nlempar Galat(\"x\");\n} tangkap galat {\ntulis(galat.pesan);\n} akhirnya {\n}"),
            "try {\n  throw new Error(\"x\");\n} catch (galat) {\n  console.log(__pesan(galat));\n} finally {\n}\n"
        );
        assert_eq!(js("coba {\n} tangkap {\n}"), "try {\n} catch {\n}\n");
        assert_eq!(js("lempar \"teks\";"), "throw \"teks\";\n");
        assert_eq!(js("x = Galat(\"y\");"), "x = new Error(\"y\");\n");
    }

    #[test]
    fn classes() {
        let src = "Kucing(nama) turunan Hewan(nama, 1) {
.umur = 1
ambil() {
tunggu g()
}
suara() {
hasilkan .umur
}
}
misal k = Kucing(\"Tom\")";
        assert_eq!(
            js(src),
            "class Kucing extends Hewan {
  constructor(nama) {
    super(nama, 1);
    this.umur = 1;
  }
  async ambil() {
    await g();
  }
  suara() {
    return this.umur;
  }
}
let k = new Kucing(\"Tom\");
"
        );
        // a base class with no statements has no constructor
        assert_eq!(js("Hewan(x) {\n}"), "class Hewan {\n}\n");
        // a derived one always has, since it has to call the parent's
        assert_eq!(
            js("Kucing(nama) turunan Hewan(nama) {\n}"),
            "class Kucing extends Hewan {\n  constructor(nama) {\n    super(nama);\n  }\n}\n"
        );
        assert_eq!(
            js("A() turunan B() {\n}"),
            "class A extends B {\n  constructor() {\n    super();\n  }\n}\n"
        );
        // `..nama` is the parent's version
        assert_eq!(
            js("Kucing() turunan Hewan() {\nsuara() {\nhasilkan ..suara() + \"!\"\n}\n}"),
            "class Kucing extends Hewan {\n  constructor() {\n    super();\n  }\n  suara() {\n    return __tambah(super.suara(), \"!\");\n  }\n}\n"
        );
        // inside a block the class is indented with it
        assert_eq!(
            js("jika benar {\nHewan() {\nm() {\n}\n}\n}"),
            "if (true) {\n  class Hewan {\n    m() {\n    }\n  }\n}\n"
        );
        // `.nama` is this.nama everywhere in a class, also in callbacks and texts
        assert_eq!(
            js("Hewan() {\nm() {\nxs.ubah(fungsi (x) {\nhasilkan x + .base\n})\ntulis(\"{.nama}\")\n}\n}"),
            "class Hewan {\n  m() {\n    xs.map((x) => {\n      return __tambah(x, this.base);\n    });\n    console.log(`${__teks(this.nama)}`);\n  }\n}\n"
        );
    }

    #[test]
    fn a_block_can_make_a_value() {
        assert_eq!(
            js("Hewan(nama) {\n  .nama = { misal x = 5; hasilkan nama + x }\n}"),
            "class Hewan {
  constructor(nama) {
    this.nama = (() => {
      let x = 5;
      return __tambah(nama, x);
    })();
  }
}
"
        );
        assert_eq!(
            js("misal v = {\n  misal a = 1\n  hasilkan a + 1\n}"),
            "let v = (() => {\n  let a = 1;\n  return __tambah(a, 1);\n})();\n"
        );
        // a block that waits makes the function around it async
        assert_eq!(
            js("fungsi f() {\nmisal v = { hasilkan tunggu g() }\n}"),
            "async function f() {\n  let v = await (async () => {\n    return await g();\n  })();\n}\n"
        );
        assert_eq!(
            js("misal v = 1 + { hasilkan tunggu g() }\n"),
            "let v = __tambah(1, await (async () => {\n  return await g();\n})());\n"
        );
        // an object stays an object
        assert_eq!(js("misal o = { a: 1 }"), "let o = { a: 1 };\n");
        // a statement that starts with a block expression is an ordinary block
        assert_eq!(js("{\nlanjut\n}"), "{\n  continue;\n}\n");
    }

    #[test]
    fn semicolons_are_optional() {
        assert_eq!(
            js("misal x = 1\ntulis(x)\njika x {\nberhenti\n}"),
            "let x = 1;\nconsole.log(x);\nif (x) {\n  break;\n}\n"
        );
        assert_eq!(js("misal x = 1; tulis(x);"), "let x = 1;\nconsole.log(x);\n");
    }

    #[test]
    fn capital_letters_build_objects() {
        assert_eq!(js("x = hewan(1);"), "x = hewan(1);\n");
        assert_eq!(js("x = Hewan(1);"), "x = new Hewan(1);\n");
        assert_eq!(js("x = Date();"), "x = new Date();\n");
        assert_eq!(js("x = Math.max(1, 2);"), "x = Math.max(1, 2);\n");
        assert_eq!(
            js("Masalah(pesan) turunan Galat(pesan) {\n}"),
            "class Masalah extends Error {\n  constructor(pesan) {\n    super(pesan);\n  }\n}\n"
        );
    }

    #[test]
    fn tunggu_inside_a_template_makes_the_function_async() {
        assert_eq!(
            js("fungsi f() {\ntulis(\"{tunggu g()}\");\n}"),
            "async function f() {\n  console.log(`${__teks(await g())}`);\n}\n"
        );
    }

    #[test]
    fn new_with_member_callee() {
        assert_eq!(js("x = a.B(1, 2).c;"), "x = new a.B(1, 2).c;\n");
    }

    #[test]
    fn call_sites_inside_function_values_and_templates() {
        let t = transpile("xs.ubah(fungsi (x) {\ntulis(x);\n});\ntulis(\"a {x}\");");
        assert_eq!(
            t.js,
            "xs.map((x) => {\n  console.log(x);\n});\nconsole.log(`a ${__teks(x)}`);\n"
        );
        assert_eq!(t.call_sites, vec![(2, 2), (4, 4)]);
    }

    #[test]
    fn statement_lines_map_every_statement_to_its_naskah_line() {
        let t = transpile("misal x = 1\n\njika x {\n  tulis(x)\n\n  x = 2\n}\nfungsi f() {\n  hasilkan 1\n}\n");
        assert_eq!(
            t.js,
            "let x = 1;\nif (x) {\n  console.log(x);\n  x = 2;\n}\nfunction f() {\n  return 1;\n}\n"
        );
        assert_eq!(t.statement_lines, vec![(1, 1), (2, 3), (3, 4), (4, 6), (6, 8), (7, 9)]);
    }

    #[test]
    fn statement_lines_inside_classes() {
        let t = transpile("Hewan(n) {\n  .n = n\n  m() {\n    hasilkan 1\n  }\n}");
        assert_eq!(t.js, "class Hewan {\n  constructor(n) {\n    this.n = n;\n  }\n  m() {\n    return 1;\n  }\n}\n");
        assert_eq!(t.statement_lines, vec![(1, 1), (3, 2), (6, 4)]);

        // the call to the parent has no line of its own, the statements after it do
        let t = transpile("Kucing(n) turunan Hewan(n) {\n  .x = 1\n}");
        assert_eq!(t.js, "class Kucing extends Hewan {\n  constructor(n) {\n    super(n);\n    this.x = 1;\n  }\n}\n");
        assert_eq!(t.statement_lines, vec![(1, 1), (4, 2)]);
    }

    #[test]
    fn statement_lines_survive_switch_and_function_values() {
        let t = transpile("pilih f() {\n  saat 1 {\n    tulis(1)\n  }\n}\nxs.ubah(fungsi (x) {\n  hasilkan x\n})");
        assert_eq!(
            t.js,
            "{\n  const _pilih0 = f();\n  if (_pilih0 === 1) {\n    console.log(1);\n  }\n}\nxs.map((x) => {\n  return x;\n});\n"
        );
        // the switch is one statement, the cases inside it are real ones
        assert_eq!(t.statement_lines, vec![(1, 1), (4, 3), (7, 6), (8, 7)]);
    }

    #[test]
    fn a_mistake_is_kept_as_data() {
        let t = transpile("misal x = ;");
        let e = t.error.expect("a mistake");
        assert_eq!((e.line, e.col), (1, 11));
        assert!(e.message.starts_with("ekspresi tidak lengkap"));
        assert!(transpile("misal x = 1").error.is_none());
        assert!(transpile_checked("tulis(nmaa)").error.is_some());
    }

    #[test]
    fn javascript_reserved_words_are_safe_names() {
        assert_eq!(
            js("misal delete = 1\nmisal new = 2\ntulis(delete * new)"),
            "let delete$ = 1;\nlet new$ = 2;\nconsole.log(delete$ * new$);\n"
        );
        assert_eq!(
            js("fungsi default(class) {\nhasilkan class\n}\ndefault(1)"),
            "function default$(class$) {\n  return class$;\n}\ndefault$(1);\n"
        );
        assert_eq!(js("untuk in dari 1 sampai 3 {\n}"), "for (let in$ = 1; in$ <= 3; in$++) {\n}\n");
        assert_eq!(js("untuk setiap with dalam xs {\n}"), "for (let with$ of xs) {\n}\n");
        assert_eq!(js("coba {\n} tangkap catch {\n}"), "try {\n} catch (catch$) {\n}\n");
        assert_eq!(js("misal f = fungsi (this) { hasilkan this }"), "let f = (this$) => {\n  return this$;\n};\n");
        // inside texts too
        assert_eq!(js("misal if = 1\nmisal s = \"{if}\""), "let if$ = 1;\nlet s = `${__teks(if$)}`;\n");
        // property names, methods and object keys may be reserved words in JavaScript
        assert_eq!(js("x = o.delete\no.default = 1"), "x = o.delete;\no.default = 1;\n");
        assert_eq!(js("misal o = { class: 1 }"), "let o = { class: 1 };\n");
        assert_eq!(
            js("Hewan() {\ndelete() {\n}\n}"),
            "class Hewan {\n  delete() {\n  }\n}\n"
        );
        // two leading underscores are for the helpers, so they could not be shadowed
        assert_eq!(js("misal __teks = 1\n__teks = 2"), "let __teks$ = 1;\n__teks$ = 2;\n");
        // the names the program normally uses are untouched
        assert_eq!(js("misal letak = 1\nmisal baru = letak"), "let letak = 1;\nlet baru = letak;\n");
    }

    #[test]
    fn bare_block_is_indented() {
        assert_eq!(
            js("jika x {\n{\nlanjut;\n}\n}"),
            "if (x) {\n  {\n    continue;\n  }\n}\n"
        );
    }
}
