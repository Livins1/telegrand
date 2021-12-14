use super::proxy_handle_dialog::ProxyHandleDialog;
use crate::utils::do_async;
use glib::clone;
use gtk::NoSelection;
use gtk::SignalListItemFactory;
use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use crate::proxy::proxy_object::ProxyObject;
use crate::proxy::proxy_row::ProxyRow;

use tdgrand::{enums, functions, types};

use adw::prelude::*;
use adw::subclass::prelude::*;
mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::Cell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/proxy-window.ui")]
    pub struct ProxyWindow {
        pub client_id: Cell<i32>,
        #[template_child]
        pub proxy_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub proxy_handle_dialog: TemplateChild<ProxyHandleDialog>,
        #[template_child]
        pub proxy_enable_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub proxy_add_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub proxy_list: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProxyWindow {
        const NAME: &'static str = "ProxyWindow";
        type Type = super::ProxyWindow;
        type ParentType = adw::PreferencesWindow;

        /* fn new() -> Self {
            Self {
                proxy_stack: TemplateChild::default(),
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
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_int(
                    "client-id",
                    "Client Id",
                    "The telegram client id",
                    std::i32::MIN,
                    std::i32::MAX,
                    0,
                    glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "client-id" => {
                    let client_id = value.get().unwrap();
                    self.client_id.set(client_id);
                }
                _ => unimplemented!(),
            }
        }
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
            // obj.setup_factory();
            obj.init_exits_proxies();
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

impl ProxyWindow {
    pub fn new(client_id: i32) -> Self {
        glib::Object::new(&[("client-id", &client_id)]).expect("Failed to create ProxyWindow")
    }

    fn setup_bindings(&self) {
        let self_ = imp::ProxyWindow::from_instance(self);
        /* self_.proxy_enable_switch.connect_active_notify(|switch| {
            // println!("Proxy enable switch: {}", switch.is_active())
        }); */

        self_
            .proxy_add_button
            .connect_clicked(clone!(@weak self as app => move |_| {
                app.show_proxy_handle_dialog();
            }));

        self_
            .proxy_stack
            .connect_visible_child_notify(clone!(@weak self as  app => move |_| {
                app.update_actions_for_visible_page()
            }));
    }

    fn update_actions_for_visible_page(&self) {
        let self_ = imp::ProxyWindow::from_instance(self);

        let visible_page = self_.proxy_stack.visible_child_name().unwrap();
        println!("{}", visible_page.as_str());
        match visible_page.as_str() {
            "main-page" => {
                self.init_exits_proxies();
            }
            "proxy-handle" => {
                self.set_default_height(300);
            }
            _ => {}
        };
    }

    fn init_exits_proxies(&self) {
        let client_id = self.client_id();
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move { functions::GetProxies::new().send(client_id).await },
            clone!(@weak self as obj => move |result| async move {
               // let self_ = imp::ProxyWindow::from_instance(&obj);
                match result {
                    Err(_err) => {}
                    Ok(proxies) => {
                        obj.set_proxies(&proxies);
                    },
                }
            }),
        );
    }

    fn set_proxies(&self, proxies: &enums::Proxies) {
        // let model = self.model();
        let self_ = imp::ProxyWindow::from_instance(self);

        match proxies {
            enums::Proxies::Proxies(proxies) => {
                let mut last_check_button: Option<gtk::CheckButton> = None;

                while let Some(child) = self_.proxy_list.last_child() {
                    self_.proxy_list.remove(&child);
                }

                for proxy in proxies.proxies.clone().into_iter() {
                    let proxy_row = ProxyRow::new();
                    let proxy_object = ProxyObject::new();
                    proxy_object.set_proxy(proxy);
                    proxy_row.bind(&proxy_object);
                    if let Some(button) = &last_check_button {
                        proxy_row.check_button_set_group(Some(&button))
                    }
                    last_check_button = Some(proxy_row.check_button());
                    self_.proxy_list.append(&proxy_row);
                }
            }
        };
    }

    fn show_proxy_handle_dialog(&self) {
        let self_ = imp::ProxyWindow::from_instance(self);
        self_.proxy_handle_dialog.set_client_id(self.client_id());
        /* let dialog = ProxyHandleDialog::new();
        dialog.set_transient_for(Some(&self.transient_for().unwrap()));
        dialog.present();
        // self.close(); */
        // self.set_height_request(200);
        // self.set_default_height(200);
        self_.proxy_stack.set_visible_child_name("proxy-handle");
        // self.set_visible(false);
    }

    pub fn client_id(&self) -> i32 {
        let self_ = imp::ProxyWindow::from_instance(self);
        self_.client_id.get()
    }

    pub fn cast_to_main_stack(&self) {
        let self_ = imp::ProxyWindow::from_instance(self);
        self_.proxy_stack.set_visible_child_name("main-page");
    }

    /* fn setup_factory(&self) {
        let factory = SignalListItemFactory::new();

        factory.connect_setup(move |_, list_item| {
            let proxy_row = ProxyRow::new();
            list_item.set_child(Some(&proxy_row));
        });

        factory.connect_bind(move |_, list_item| {
            let proxy_object = list_item
                .item()
                .expect("The item has to exist.")
                .downcast::<ProxyObject>()
                .expect("The item has to be an `ProxyObject`.");

            let proxy_row = list_item
                .child()
                .expect("The child has to exist.")
                .downcast::<ProxyRow>()
                .expect("The child has to be a `ProxyRow`.");

            proxy_row.bind(&proxy_object);
        });

        factory.connect_unbind(move |_, list_item| {
            let proxy_row = list_item
                .child()
                .expect("The child has to exist.")
                .downcast::<ProxyRow>()
                .expect("The child has to be a `ProxyRow`.");

            proxy_row.unbind();
        });

        let self_ = imp::ProxyWindow::from_instance(self);
        // self_.proxy_list.set_factory(Some(&factory));
    } */
}
