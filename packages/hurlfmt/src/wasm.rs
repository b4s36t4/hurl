use crate::{curl, format};
use wasm_bindgen::prelude::*;

use std::panic;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[wasm_bindgen]
pub enum InputFormat {
    Hurl,
    Curl,
}

impl std::fmt::Display for InputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputFormat::Hurl => write!(f, "Hurl"),
            InputFormat::Curl => write!(f, "Curl"),
        }
    }
}

#[wasm_bindgen]
pub struct HurlFmt {
    pub input_format: InputFormat,
}

#[wasm_bindgen]
impl HurlFmt {
    pub fn new(input_format: InputFormat) -> Self {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        Self { input_format }
    }

    pub fn parse_hurl_file(&self, content: &str) -> String {
        parse_hurl_file(content, self.input_format)
    }

    pub fn parse_curl_file(&self, content: &str) -> String {
        parse_curl_file(content)
    }
}

#[allow(unreachable_patterns)]
#[wasm_bindgen]
pub fn parse_hurl_file(content: &str, input_format: InputFormat) -> String {
    log(&format!("Running parse_hurl_file {}", input_format));
    let input = match input_format {
        InputFormat::Hurl => content.to_string(),
        InputFormat::Curl => curl::parse(&content)
            .map_err(|error| error.to_string())
            .unwrap(),
        _ => panic!("Invalid input format"),
    };
    let hurl_file = hurl_core::parser::parse_hurl_file(&input).unwrap();
    log("Done parse_hurl_file");
    let formatted = format::format_json(&hurl_file);
    formatted
}

#[wasm_bindgen]
pub fn parse_curl_file(content: &str) -> String {
    let input = curl::parse(&content).unwrap();
    input
}

#[wasm_bindgen]
pub fn test_wasm() {
    log("Running test_wasm");
    log("Done test_wasm");
}
