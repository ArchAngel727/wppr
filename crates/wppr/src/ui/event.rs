use crate::local_image::LocalImage;

pub enum UiResult {
    Selected(LocalImage),
    Cancelled,
}

pub enum EventResult {
    Continue,
    Cancel,
    Exit(Option<UiResult>),
}
