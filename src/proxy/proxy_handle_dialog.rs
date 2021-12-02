use glib::clone;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use adw::prelude::*;
use tdgrand::{functions,types};
use crate::utils::{do_async};


#[derive(Debug, Eq, PartialEq, Clone, Copy, glib::GEnum, num_derive::FromPrimitive)]
#[repr(u32)]
#[genum(type_name = "ProxyTypes")]
pub enum ProxyTypes {
    #[genum(name = "Socks5", nick = "socks5")]
    Socks5 = 0,
    #[genum(name = "Http", nick = "http")]
    Http,
}

impl Default for ProxyTypes {
    fn default() -> Self {
        Self::Socks5
    }
}

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    // use once_cell::sync::{Lazy, OnceCell};
    use std::cell::{Cell};

    
    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/proxy-handle-dialog.ui")]
    pub struct ProxyHandleDialog {
        pub client_id: Cell<i32>,
        #[template_child]
        pub proxy_types: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub proxy_address_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub proxy_port_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub proxy_auth_username_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub proxy_auth_passwd_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub proxy_save_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProxyHandleDialog {
        const NAME: &'static str = "ProxyHandleDialog";
        type Type = super::ProxyHandleDialog;
        type ParentType = adw::Window;
        
        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("proxy.save-proxy", None,move |widget,_,_| {
                widget.add_save_proxy();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProxyHandleDialog {
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

    impl WidgetImpl for ProxyHandleDialog {}
    impl WindowImpl for ProxyHandleDialog {}
    impl AdwWindowImpl for ProxyHandleDialog {}

}

glib::wrapper! {
    pub struct ProxyHandleDialog(ObjectSubclass<imp::ProxyHandleDialog>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}


fn is_non_ascii_digit(c:char) -> bool {
    !c.is_ascii_digit()
}


impl ProxyHandleDialog {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ProxyHandleDialog")
    }


    fn setup_bindings(&self) {

        let self_ = imp::ProxyHandleDialog::from_instance(self);

        // port validator
        self_.proxy_port_entry.connect_text_notify(clone!(@weak self as app => move |widget| {
            println!("test");
            let text = widget.text();
            if text.contains(is_non_ascii_digit) {
                widget.set_text(&text.replace(is_non_ascii_digit, ""))
            }
        }));

        self_.proxy_types.connect_selected_item_notify(clone!(@weak self as app => move |proxy_types| {
                if let Some(select_item) = proxy_types.selected_item() {
                    match select_item.downcast::<adw::EnumListItem>().unwrap().nick().unwrap().as_str() 
                    {
                        "socks5" => {
                            println!("socks5 selected!") },
                        "http" => {
                            println!("http selected!")
                },
                        _ => {}
                    };
                }
            })
        );
    }

    fn client_id(&self) -> i32 {
        let self_ = imp::ProxyHandleDialog::from_instance(self);
        self_.client_id.get()
    }

    fn add_save_proxy(&self) {
        let self_ = imp::ProxyHandleDialog::from_instance(self);
        let address = self_.proxy_address_entry.text().to_string();
        println!("{}",address);
        let port = self_.proxy_port_entry.text().to_string();
        println!("{}",port);

        let passwd = self_.proxy_auth_username_entry.text().to_string();
        /* do_async(
            
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::AddProxy::new().port(port).server(address).type
            }
        

        ) */
        



    }
}


