use crate::local_image::LocalImage;

pub(crate) enum UiResult {
    Selected(LocalImage),
    Cancelled,
}

pub enum EventResult {
    Continue,
    Cancel,
    Exit(Option<UiResult>),
}
