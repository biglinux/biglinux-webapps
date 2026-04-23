mod fields;
mod lifecycle;
mod media;

pub(super) use fields::{
    setup_browser_handler, setup_category_handler, setup_name_handler, setup_profile_handlers,
    setup_url_handler,
};
pub(super) use lifecycle::{setup_cancel_handler, setup_destroy_handler, setup_save_handler};
pub(super) use media::{setup_detection_handler, setup_favicon_picker, setup_icon_picker};
