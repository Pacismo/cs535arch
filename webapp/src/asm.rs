use libasm::{compile, Input as AsmInput};
use wasm_bindgen::{prelude::wasm_bindgen, JsError};

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Input {
    filename: String,
    content: String,
}

#[wasm_bindgen]
impl Input {
    #[wasm_bindgen(constructor)]
    pub fn new(filename: &str, content: &str) -> Input {
        Input {
            filename: filename.to_owned(),
            content: content.to_owned(),
        }
    }
}

#[wasm_bindgen]
pub fn assemble_files(input: Vec<Input>) -> Result<Vec<u8>, JsError> {
    compile(input.iter().map(|Input { filename, content }| AsmInput {
        path: filename,
        data: content,
    }))
    .map_err(|e| JsError::from(e))
}
