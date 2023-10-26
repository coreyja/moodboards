use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    log("Hello from Rust with a new script!");

    let new_h1 = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("h1")
        .unwrap();

    new_h1.set_text_content(Some("A new h1 appeared"));

    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .body()
        .unwrap()
        .append_child(&new_h1)?;

    Ok(())
}
