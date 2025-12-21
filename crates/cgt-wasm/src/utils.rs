use std::fmt::Display;
use wasm_bindgen::JsValue;

/// Convert any error that implements Display into a JsValue for JavaScript consumption
pub fn map_error<E: Display>(error: E) -> JsValue {
    JsValue::from_str(&error.to_string())
}
