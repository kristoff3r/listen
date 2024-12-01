use leptos::prelude::document;
use web_sys::wasm_bindgen::JsCast;

pub fn get_element_by_id<T>(id: &str) -> Option<T>
where
    T: JsCast,
{
    let element = document()
        .get_element_by_id(id)?
        .dyn_into::<T>()
        .ok()
        .expect("wrong type");
    Some(element)
}
