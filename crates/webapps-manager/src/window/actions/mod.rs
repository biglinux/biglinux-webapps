mod about;
mod browse;
mod import_export;
mod remove_all;
mod remove_multiple;

use super::context::WindowContext;

pub(super) fn install_window_actions(context: &WindowContext) {
    about::install(context);
    import_export::install_import_action(context);
    import_export::install_export_action(context);
    browse::install(context);
    remove_multiple::install(context);
    remove_all::install(context);
}
