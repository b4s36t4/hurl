use wasm_bindgen::prelude::*;

use std::panic;

#[wasm_bindgen]
pub struct HurlFmt {
    hurl_content: String,
}

#[allow(dead_code)]
#[wasm_bindgen]
pub struct RunOptions {
    pub verbose: bool,
    pub very_verbose: bool,
    // pub variables: HashMap<String, Value>,
    #[wasm_bindgen(getter_with_clone)]
    pub variable_file: String,
}

#[wasm_bindgen]
impl HurlFmt {
    pub fn new(hurl_content: String) -> Self {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        Self { hurl_content }
    }

    #[allow(unused_variables)]
    pub fn run_hurl(&self, options: RunOptions) -> String {
        let _hurl_content = self.hurl_content.clone();

        let base_logger = BaseLogger::new(false, options.verbose);


        // let hurl_result = hurl::run(&hurl_content);
        // hurl_result.to_string()
        "".to_string()
    }
}
