#[macro_use]
extern crate nom;
extern crate regex;

mod ast;
mod parser;
mod printer;

use parser::parse;
use printer::js::print;

use std::ffi::CString;
use std::mem;
use std::os::raw::{c_char, c_void};

fn transpile_to_js(s: String) -> String {
    let naskah_ast = parse(&s);
    match naskah_ast {
        Ok(ast) => print(ast),
        Err(_) => String::from("salah sintaks"),
    }
}

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut c_void {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    return ptr as *mut c_void;
}

#[no_mangle]
pub extern "C" fn dealloc_str(ptr: *mut c_char) {
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

#[no_mangle]
pub extern "C" fn transpile(ptr: *mut c_char) -> *mut c_char {
    let input: String;

    unsafe {
        input = CString::from_raw(ptr).into_string().unwrap();
    }

    let script = transpile_to_js(input);
    let c_script = CString::new(script).unwrap();
    c_script.into_raw()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ext_single() {
        let js = transpile_to_js(String::from("misal x = null;\n"));
        assert_eq!(js, String::from("var x = null;\n"));
    }

    #[test]
    fn ext_multi() {
        let js = transpile_to_js(String::from("misal x = null;\nmisal y = benar;\n"));
        assert_eq!(js, String::from("var x = null;\nvar y = true;\n"));
    }
}
