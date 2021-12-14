use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use crate::proxy::proxy_object::ProxyObject;
use glib::Binding;
use glib::BindingFlags;
mod imp {
    use super::*;
    use adw::subclass::prelude::ActionRowImpl;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/proxy-row.ui")]
    pub struct ProxyRow {
        pub bindings: RefCell<Vec<Binding>>,
        #[template_child]
        pub check_button: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub type_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub server_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProxyRow {
        const NAME: &'static str = "ProxyRow";
        type Type = super::ProxyRow;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProxyRow {}
    impl BoxImpl for ProxyRow {}
    impl ListBoxRowImpl for ProxyRow {}
    impl WidgetImpl for ProxyRow {}
}

glib::wrapper! {
    pub struct ProxyRow(ObjectSubclass<imp::ProxyRow>)
    @extends gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl ProxyRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Fail to create ProxyRow")
    }

    pub fn check_button_set_group(&self,button: Option<&gtk::CheckButton>) {
        let self_ = imp::ProxyRow::from_instance(self);
        self_.check_button.set_group(button);
    }

    pub fn check_button(&self) -> gtk::CheckButton {
        let self_ = imp::ProxyRow::from_instance(self);
        self_.check_button.get()
    }


    pub fn bind(&self, proxy_object: &ProxyObject) {
        let self_ = imp::ProxyRow::from_instance(self);
        let checkt_button = self_.check_button.get();
        let server_label = self_.server_label.get();
        let type_label = self_.type_label.get();
        let mut bindings = self_.bindings.borrow_mut();

        let checkt_button_binding = proxy_object
            .bind_property("enabled", &checkt_button, "active")
            .flags(BindingFlags::SYNC_CREATE | BindingFlags::BIDIRECTIONAL)
            .build()
            .expect("Could not bind properties");

        bindings.push(checkt_button_binding);

        let type_label_binding = proxy_object
            .bind_property("type", &type_label, "label")
            .flags(BindingFlags::SYNC_CREATE)
            .build()
            .expect("Could not bind properties");
        // Save binding
        bindings.push(type_label_binding);

        let content_label_binding = proxy_object
            .bind_property("server", &server_label, "label")
            .flags(BindingFlags::SYNC_CREATE)
            .build()
            .expect("Could not bind properties");
        // Save binding
        bindings.push(content_label_binding);
    }

    pub fn unbind(&self) {
        let self_ = imp::ProxyRow::from_instance(self);
        for binding in self_.bindings.borrow_mut().drain(..) {
            binding.unbind();
        }
    }
}
