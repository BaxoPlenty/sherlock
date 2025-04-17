use std::vec;

use gio::glib::object::ObjectExt;
use gtk4::prelude::WidgetExt;

use super::util::EventTileBuilder;
use super::Tile;
use crate::actions::{execute_from_attrs, get_attrs_map};
use crate::g_subclasses::sherlock_row::SherlockRow;
use crate::launcher::event_launcher::EventLauncher;
use crate::launcher::{Launcher, ResultItem};

impl Tile {
    pub fn event_tile(
        launcher: &Launcher,
        keyword: &str,
        event_launcher: &EventLauncher,
    ) -> Vec<ResultItem> {
        let event = match &event_launcher.event {
            Some(event) => event,
            None => return vec![],
        };

        //Handle searching
        if !event.title.contains(keyword) {
            return vec![];
        }

        let builder = EventTileBuilder::new("/dev/skxxtz/sherlock/ui/event_tile.ui");
        builder.object.add_css_class("event-tile");
        builder.object.set_spawn_focus(launcher.spawn_focus);
        builder.object.set_shortcut(launcher.shortcut);

        builder.title.set_text(&event.title);
        builder
            .icon
            .set_icon_name(Some(event_launcher.icon.as_ref()));
        builder.start_time.set_text(&event.start_time);
        builder
            .end_time
            .set_text(format!(".. {}", event.end_time).as_str());

        let mut constructor: Vec<(&str, &str)> = vec![
            ("method", &launcher.method),
            ("meeting_url", &event.meeting_url),
        ];
        if let Some(next) = launcher.next_content.as_deref() {
            constructor.push(("next_content", next));
        }
        let attrs = get_attrs_map(constructor);

        builder
            .object
            .connect("row-should-activate", false, move |row| {
                let row = row.first().map(|f| f.get::<SherlockRow>().ok())??;
                execute_from_attrs(&row, &attrs);
                None
            });

        let shortcut_holder = match launcher.shortcut {
            true => builder.shortcut_holder,
            _ => None,
        };
        let res = ResultItem {
            priority: launcher.priority as f32,
            row_item: builder.object,
            shortcut_holder,
            alias: launcher.alias.clone(),
            home: launcher.home,
            only_home: launcher.only_home,
        };
        return vec![res];
    }
}
