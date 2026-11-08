//! Nur für den Browser (WebAssembly): Dateien zum Herunterladen anbieten.

use wasm_bindgen::JsCast;

/// Bietet die Daten als Datei zum Herunterladen an. Der Browser (oder Safari auf dem iPhone) fragt dann, wohin
/// sie sollen.
pub fn download(name: &str, mime: &str, data: &[u8]) -> Result<(), String> {
    let js_err = |e: wasm_bindgen::JsValue| format!("{e:?}");
    let bytes = js_sys::Uint8Array::from(data);
    let parts = js_sys::Array::of1(&bytes);
    let options = web_sys::BlobPropertyBag::new();
    options.set_type(mime);
    let blob =
        web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options).map_err(js_err)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(js_err)?;
    let document = web_sys::window()
        .and_then(|w| w.document())
        .ok_or("Kein Browserfenster")?;
    let link: web_sys::HtmlAnchorElement = document
        .create_element("a")
        .map_err(js_err)?
        .dyn_into()
        .map_err(|_| "Kein Link".to_string())?;
    link.set_href(&url);
    link.set_download(name);
    link.click();
    // Der Download startet sofort, danach wird die Adresse nicht mehr gebraucht.
    let _ = web_sys::Url::revoke_object_url(&url);
    Ok(())
}
