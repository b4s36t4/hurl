use std::panic;
use wasm_bindgen::prelude::*;

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

    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn error(a: &str);
}

#[wasm_bindgen]
pub struct BaseLogger {
    verbose: bool,
}

#[wasm_bindgen]
impl BaseLogger {
    /// Creates a new base logger using `color` and `verbose`.
    pub fn new(verbose: bool) -> BaseLogger {
        BaseLogger { verbose }
    }

    /// Prints an informational `message` on standard error.
    pub fn info(&self, message: &str) {
        log(&format!("{message}"));
    }

    /// Prints a debug `message` on standard error if the logger is in verbose mode.
    pub fn debug(&self, message: &str) {
        if !self.verbose {
            return;
        }
        let mut s = String::new();
        s.push_str("*");
        if !message.is_empty() {
            s.push_str(&format!(" {message}"));
        }
        log(&format!("{}", s.to_string()));
    }

    /// Prints an error `message` on standard error.
    pub fn error(&self, message: &str) {
        let mut s = String::new();
        s.push_str("error");
        s.push_str(": ");
        s.push_str(message);
        log(&format!("{}", s.to_string()));
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
#[wasm_bindgen]
pub struct RunOptions {
    pub verbose: bool,
    pub very_verbose: bool,
    #[wasm_bindgen(getter_with_clone)]
    pub variable_file: String,
}

#[wasm_bindgen]
pub struct HurlFmt {
    options: RunOptions,
}

#[wasm_bindgen]
impl HurlFmt {
    pub fn new(options: RunOptions) -> Self {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        Self { options }
    }

    #[allow(unused_variables)]
    pub fn run_hurl(&self, code: &str) -> String {
        let _hurl_content = code;

        // let base_logger = BaseLogger::new(self.options.verbose);

        // base_logger.info("started running!!");
        // let hurl_result = hurl::run(&hurl_content);
        // hurl_result.to_string()
        "".to_string()
    }
}
