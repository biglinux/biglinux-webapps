use big_app_kit::desktop;
use gettextrs::gettext;
use gtk::glib::variant::ToVariant;
use gtk4 as gtk;
use webkit6 as webkit;
use webkit6::prelude::*;

pub(super) fn setup_context_menu(webview: &webkit::WebView) {
    use gtk::gio;

    let ctx_group = gio::SimpleActionGroup::new();
    let action = desktop::install_string_action(&ctx_group, "open-link-browser", |uri| {
        let _ = gio::AppInfo::launch_default_for_uri(&uri, gio::AppLaunchContext::NONE);
    });
    webview.insert_action_group("ctx", Some(&ctx_group));

    let action_ref = action;
    webview.connect_context_menu(move |_wv, menu, hit| {
        if let Some(uri) = hit.link_uri() {
            menu.append(&webkit::ContextMenuItem::new_separator());
            let target = uri.to_variant();
            let item = webkit::ContextMenuItem::from_gaction(
                &action_ref,
                &gettext("Open Link in Browser"),
                Some(&target),
            );
            menu.append(&item);
        }
        false
    });
}
