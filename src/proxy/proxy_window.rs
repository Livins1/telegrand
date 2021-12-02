use glib::clone;
use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use crate::config::APP_ID;

use super::proxy_handle_dialog::ProxyHandleDialog;
mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use std::cell::{Cell};


    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/proxy-window.ui")]
    pub struct ProxyWindow {
        pub client_id: Cell<i32>,
        #[template_child]
        pub proxy_enable_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub proxy_add_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProxyWindow {
        const NAME: &'static str = "ProxyWindow";
        type Type = super::ProxyWindow;
        type ParentType = adw::PreferencesWindow;
        
        /* fn new() -> Self {
            Self {
                client_id: Cell::default(),
                proxy_enable_switch: TemplateChild::default(),
                proxy_add_button: TemplateChild::default(),
                proxy_handle_dialog: TemplateChild::default(),

            }
        } */


        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass)
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProxyWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            // If the system supports color schemes, load the 'Follow system colors'
            // switch state, otherwise make that switch insensitive
            /* let style_manager = adw::StyleManager::default();
            if let Some(style_manager) = style_manager {
                if style_manager.system_supports_color_schemes() {
                    let settings = gio::Settings::new(APP_ID);
                    let follow_system_colors = settings.string("color-scheme") == "default";
                    self.follow_system_colors_switch
                        .set_active(follow_system_colors);
                } else {
                    self.follow_system_colors_switch.set_sensitive(false);
                }
            } */

            obj.setup_bindings();
        }
    }
    impl WidgetImpl for ProxyWindow {}
    impl WindowImpl for ProxyWindow {}
    impl AdwWindowImpl for ProxyWindow {}
    impl PreferencesWindowImpl for ProxyWindow {}
}

glib::wrapper! {
    pub struct ProxyWindow(ObjectSubclass<imp::ProxyWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window, adw::PreferencesWindow;
}

impl Default for ProxyWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl ProxyWindow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ProxyWindow")
    }

    pub fn create_proxy_window(&self, client: i32) {
        let self_ = imp::ProxyWindow::from_instance(self);
        self_.client_id.set(client);
    }

    fn setup_bindings(&self) {
        let self_ = imp::ProxyWindow::from_instance(self);

        self_.proxy_enable_switch.connect_active_notify(|switch| {
            // println!("Proxy enable switch: {}", switch.is_active())
        });


        self_.proxy_add_button.connect_clicked(clone!(@weak self as app => move |_| {
            app.show_add_proxy_dialog();
        }));
    }

    fn show_add_proxy_dialog(&self) {
        let dialog = ProxyHandleDialog::new();
        dialog.set_transient_for(Some(self));
        dialog.present();
    }
}
