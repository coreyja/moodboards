use core::panic;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{HtmlElement, MouseEvent, Request, RequestInit, RequestMode, Response};

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
async fn start() -> Result<(), JsValue> {
    log("Hello from Rust with a new script!");

    let document = web_sys::window().unwrap().document().unwrap();

    let new_h1 = document.create_element("h1").unwrap();

    new_h1.set_text_content(Some("A new h1 appeared"));

    document.body().unwrap().append_child(&new_h1)?;

    let cb = Closure::<dyn FnMut(MouseEvent)>::new(|event: MouseEvent| {
        let event_target = event.target().unwrap();
        let t = event_target.dyn_ref::<HtmlElement>().unwrap().clone();
        let Some(url) = t.get_attribute("cja-click") else {
            return;
        };
        let method = t
            .get_attribute("cja-method")
            .unwrap_or_else(|| "GET".to_string());

        let Some(replace_id) = t.get_attribute("cja-replace-id") else {
            return;
        };

        log(&format!("Mouse event: {:?}", event));

        let mut opts = RequestInit::new();
        opts.method(&method);
        opts.mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(&url, &opts).unwrap();

        let window = web_sys::window().unwrap();
        spawn_local(async move {
            let resp_value = JsFuture::from(window.fetch_with_request(&request))
                .await
                .unwrap();

            assert!(resp_value.is_instance_of::<Response>());
            let resp: Response = resp_value.dyn_into().unwrap();
            let body: String = JsFuture::from(resp.text().unwrap())
                .await
                .unwrap()
                .as_string()
                .unwrap();

            log(&format!("Response: {:?}", body));
            log(&format!("id: {:?}", replace_id));
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id(&replace_id)
                .unwrap()
                .set_outer_html(&body);
        });
    });
    let cb_ref = cb.as_ref().unchecked_ref();

    document
        .body()
        .unwrap()
        .add_event_listener_with_callback("click", cb_ref)?;

    std::mem::forget(cb);

    Ok(())
}
