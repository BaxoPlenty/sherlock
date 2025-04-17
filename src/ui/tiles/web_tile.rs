use gio::glib::object::ObjectExt;
use gtk4::prelude::WidgetExt;

use super::util::TileBuilder;
use super::Tile;
use crate::actions::{execute_from_attrs, get_attrs_map};
use crate::g_subclasses::sherlock_row::SherlockRow;
use crate::launcher::web_launcher::Web;
use crate::launcher::{Launcher, ResultItem};

impl Tile {
    pub fn web_tile(launcher: &Launcher, keyword: &str, web: &Web) -> Vec<ResultItem> {
        let builder = TileBuilder::new("/dev/skxxtz/sherlock/ui/tile.ui");
        builder.object.set_spawn_focus(launcher.spawn_focus);
        builder.object.set_shortcut(launcher.shortcut);
        builder.object.set_search(String::from("always-flag"));

        if let Some(name) = &launcher.name {
            builder.category.set_text(name);
        } else {
            builder.category.set_visible(false);
        }

        builder.icon.set_icon_name(Some(&web.icon));

        let tile_name = if web.display_name.contains("{keyword}") {
            web.display_name.replace("{keyword}", keyword)
        } else {
            web.display_name.clone()
        };

        builder.title.set_text(&tile_name);
        builder.display_tag_start(&launcher.tag_start, keyword);
        builder.display_tag_end(&launcher.tag_end, keyword);

        // Construct attrs and enable action capabilities
        let mut attrs = get_attrs_map(vec![
            ("method", &launcher.method),
            ("result", keyword),
            ("keyword", keyword),
            ("engine", &web.engine),
        ]);
        if let Some(next) = launcher.next_content.as_deref() {
            attrs.insert(String::from("next_content"), next.to_string());
        }
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
        println!("{:?}", res);

        return vec![res];
    }
}
