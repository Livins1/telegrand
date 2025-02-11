mod avatar;
mod row;

use self::row::Row;

use glib::clone;
use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use tdgrand::{enums, functions, types};

use crate::session::{Chat, User};
use crate::utils::do_async;
use crate::Session;

pub use self::avatar::Avatar;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/sidebar.ui")]
    pub struct Sidebar {
        pub compact: Cell<bool>,
        pub selected_chat: RefCell<Option<Chat>>,
        pub session: RefCell<Option<Session>>,
        pub filter: RefCell<Option<gtk::CustomFilter>>,
        pub selection: RefCell<Option<gtk::SingleSelection>>,
        pub searched_chats: RefCell<Vec<i64>>,
        pub searched_users: RefCell<Vec<i64>>,
        pub already_searched_users: RefCell<Vec<i64>>,
        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub search_bar: TemplateChild<gtk::SearchBar>,
        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub list_view: TemplateChild<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Sidebar {
        const NAME: &'static str = "Sidebar";
        type Type = super::Sidebar;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Row::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Sidebar {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_boolean(
                        "compact",
                        "Compact",
                        "Wheter a compact view is used or not",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_object(
                        "selected-chat",
                        "Selected Chat",
                        "The selected chat in this sidebar",
                        Chat::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_object(
                        "session",
                        "Session",
                        "The session",
                        Session::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "compact" => {
                    let compact = value.get().unwrap();
                    self.compact.set(compact);
                }
                "selected-chat" => {
                    let selected_chat = value.get().unwrap();
                    obj.set_selected_chat(selected_chat);
                }
                "session" => {
                    let session = value.get().unwrap();
                    obj.set_session(session);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "compact" => self.compact.get().to_value(),
                "selected-chat" => obj.selected_chat().to_value(),
                "session" => obj.session().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.search_entry
                .connect_search_changed(clone!(@weak obj => move |entry| {
                    let query = entry.text().to_string();
                    obj.search(query);
                }));
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.header_bar.unparent();
            self.search_bar.unparent();
            self.scrolled_window.unparent();
        }
    }

    impl WidgetImpl for Sidebar {}
}

glib::wrapper! {
    pub struct Sidebar(ObjectSubclass<imp::Sidebar>)
        @extends gtk::Widget;
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Sidebar {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Sidebar")
    }

    pub fn begin_chats_search(&self) {
        let self_ = imp::Sidebar::from_instance(self);
        self_.search_bar.set_search_mode(true);
        self_.search_entry.grab_focus();
    }

    fn search(&self, query: String) {
        let self_ = imp::Sidebar::from_instance(self);
        self_.searched_chats.borrow_mut().clear();
        self_.searched_users.borrow_mut().clear();
        self_.already_searched_users.borrow_mut().clear();

        if query.is_empty() {
            if let Some(filter) = self_.filter.borrow().as_ref() {
                filter.changed(gtk::FilterChange::Different);
            }
        } else {
            let client_id = self
                .session()
                .expect("The session needs to be set to be able to search")
                .client_id();

            // Search chats
            let query_clone = query.clone();
            do_async(
                glib::PRIORITY_DEFAULT_IDLE,
                async move {
                    functions::SearchChats::new()
                        .query(query_clone)
                        .limit(100)
                        .send(client_id)
                        .await
                },
                clone!(@weak self as obj => move |result| async move {
                    if let Ok(enums::Chats::Chats(chats)) = result {
                        let self_ = imp::Sidebar::from_instance(&obj);

                        if let Some(filter) = self_.filter.borrow().as_ref() {
                            let session = obj
                                .session()
                                .expect("The session needs to be set to be able to search");
                            let chat_list = session.chat_list();

                            self_.already_searched_users.borrow_mut().extend(chats.chat_ids.iter()
                                .filter_map(|id| chat_list.get_chat(*id))
                                .filter_map(|chat| match chat.type_() {
                                    enums::ChatType::Private(types::ChatTypePrivate { user_id }) => Some(*user_id),
                                    _ => None
                                }
                            ));

                            self_.searched_chats.borrow_mut().extend(chats.chat_ids);
                            filter.changed(gtk::FilterChange::Different);
                        }
                    }
                }),
            );

            // Search contacts
            do_async(
                glib::PRIORITY_DEFAULT_IDLE,
                async move {
                    functions::SearchContacts::new()
                        .query(query)
                        .limit(100)
                        .send(client_id)
                        .await
                },
                clone!(@weak self as obj => move |result| async move {
                    if let Ok(enums::Users::Users(users)) = result {
                        let self_ = imp::Sidebar::from_instance(&obj);

                        if let Some(filter) = self_.filter.borrow().as_ref() {
                            self_.searched_users.borrow_mut().extend(users.user_ids);
                            filter.changed(gtk::FilterChange::Different);
                        }
                    }
                }),
            );
        }
    }

    fn selected_chat(&self) -> Option<Chat> {
        let self_ = imp::Sidebar::from_instance(self);
        self_.selected_chat.borrow().clone()
    }

    fn set_selected_chat(&self, selected_chat: Option<Chat>) {
        if self.selected_chat() == selected_chat {
            return;
        }

        // TODO: change the selection in the sidebar if it's
        // different from the current selection

        let self_ = imp::Sidebar::from_instance(self);
        if selected_chat.is_none() {
            self_
                .selection
                .borrow()
                .as_ref()
                .unwrap()
                .set_selected(gtk::INVALID_LIST_POSITION);
        }

        self_.selected_chat.replace(selected_chat);
        self.notify("selected-chat");
    }

    pub fn set_session(&self, session: Option<Session>) {
        if self.session() == session {
            return;
        }

        let self_ = imp::Sidebar::from_instance(self);

        if let Some(ref session) = session {
            // Merge ChatList and UserList into a single list model
            let list = gio::ListStore::new(gio::ListModel::static_type());
            list.append(session.chat_list());
            list.append(session.user_list());
            let model = gtk::FlattenListModel::new(Some(&list));

            let filter = gtk::CustomFilter::new(
                clone!(@weak self as obj => @default-return false, move |item| {
                    let self_ = imp::Sidebar::from_instance(&obj);
                    let is_searching = !self_.search_entry.text().is_empty();

                    if is_searching {
                        if let Some(chat) = item.downcast_ref::<Chat>() {
                            self_.searched_chats.borrow().contains(&chat.id())
                        } else if let Some(user) = item.downcast_ref::<User>() {
                            // Show searched users, but only the ones that haven't
                            // already been searched by the chats search
                            !self_.already_searched_users.borrow().contains(&user.id())
                                && self_.searched_users.borrow().contains(&user.id())
                        } else {
                            false
                        }
                    } else if let Some(chat) = item.downcast_ref::<Chat>() {
                        chat.order() > 0
                    } else {
                        false
                    }
                }),
            );
            let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
                let chat1 = obj1.downcast_ref::<Chat>();
                let chat2 = obj2.downcast_ref::<Chat>();

                // Always show chats first and then users
                if let Some(chat1) = chat1 {
                    if let Some(chat2) = chat2 {
                        chat2.order().cmp(&chat1.order()).into()
                    } else {
                        gtk::Ordering::Smaller
                    }
                } else if chat2.is_some() {
                    gtk::Ordering::Larger
                } else {
                    gtk::Ordering::Equal
                }
            });

            session.chat_list().connect_positions_changed(
                clone!(@weak filter, @weak sorter => move |_| {
                    filter.changed(gtk::FilterChange::Different);
                    sorter.changed(gtk::SorterChange::Different);
                }),
            );

            let filter_model = gtk::FilterListModel::new(Some(&model), Some(&filter));
            let sort_model = gtk::SortListModel::new(Some(&filter_model), Some(&sorter));
            let selection = gtk::SingleSelection::new(Some(&sort_model));
            selection.set_autoselect(false);

            selection.connect_selected_item_notify(
                clone!(@weak self as obj, @weak session => move |selection| {
                    if let Some(item) = selection.selected_item() {
                        if let Some(chat) = item.downcast_ref::<Chat>() {
                            obj.set_selected_chat(Some(chat.to_owned()));
                        } else if let Some(user) = item.downcast_ref::<User>() {
                            // Create a chat with the user and then select the created chat
                            let user_id = user.id();
                            let client_id = session.client_id();
                            do_async(
                                glib::PRIORITY_DEFAULT_IDLE,
                                async move {
                                    functions::CreatePrivateChat::new()
                                        .user_id(user_id)
                                        .send(client_id)
                                        .await
                                },
                                clone!(@weak obj, @weak session => move |result| async move {
                                    if let Ok(enums::Chat::Chat(chat)) = result {
                                        let chat = session.chat_list().get_chat(chat.id);
                                        obj.set_selected_chat(chat);
                                    }
                                }),
                            );
                        } else {
                            unreachable!("Unexpected item type: {:?}", item);
                        }
                    } else {
                        obj.set_selected_chat(None);
                    }
                }),
            );

            self_.list_view.set_model(Some(&selection));
            self_.filter.replace(Some(filter));
            self_.selection.replace(Some(selection));
        }

        self_.session.replace(session);
        self.notify("session");
    }

    pub fn session(&self) -> Option<Session> {
        let self_ = imp::Sidebar::from_instance(self);
        self_.session.borrow().to_owned()
    }
}
