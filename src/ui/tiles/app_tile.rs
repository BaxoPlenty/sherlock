use gtk4::prelude::*;
use std::collections::HashMap;

use crate::actions::{execute_from_attrs, get_attrs_map};
use crate::g_subclasses::sherlock_row::SherlockRow;
use crate::launcher::{Launcher, ResultItem};
use crate::loader::util::AppData;

use super::util::TileBuilder;
use super::Tile;

impl Tile {
    pub fn app_tile(
        launcher: &Launcher,
        keyword: &str,
        commands: &HashMap<String, AppData>,
    ) -> Vec<ResultItem> {
        let mut results: Vec<ResultItem> = Default::default();

        for (key, value) in commands.into_iter() {
            let builder = TileBuilder::new("/dev/skxxtz/sherlock/ui/tile.ui");
            builder.object.set_spawn_focus(launcher.spawn_focus);
            builder.object.set_shortcut(launcher.shortcut);
            builder.object.set_search(value.search_string.to_lowercase());

            let tile_name = key.replace("{keyword}", keyword);
            builder.display_tag_start(&value.tag_start, keyword);
            builder.display_tag_end(&value.tag_end, keyword);

            if let Some(name) = &launcher.name {
                builder.category.set_text(name);
            } else {
                builder.category.set_visible(false);
            }

            // Icon stuff
            builder.icon.set_icon_name(Some(&value.icon));
            value
                .icon_class
                .as_ref()
                .map(|c| builder.icon.add_css_class(c));
            builder.title.set_markup(&tile_name);

            let attrs =
                get_attrs_map(vec![("method", &launcher.method), ("exec", &value.exec)]);

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
            results.push(ResultItem {
                priority: value.priority,
                row_item: builder.object,
                shortcut_holder,
                alias: launcher.alias.clone(),
                home: launcher.home,
                only_home: launcher.only_home,
            });
        }
        results
    }
}
