use gtk4::prelude::*;

use crate::actions::{execute_from_attrs, get_attrs_map};
use crate::g_subclasses::sherlock_row::SherlockRow;
use crate::launcher::process_launcher::ProcessLauncher;
use crate::launcher::{Launcher, ResultItem};

use super::util::{SherlockSearch, TileBuilder};
use super::Tile;

impl Tile {
    pub fn process_tile(
        launcher: &Launcher,
        keyword: &str,
        proc: &ProcessLauncher,
    ) -> Vec<ResultItem> {
        let mut results: Vec<ResultItem> = Default::default();

        for (key, value) in proc.processes.iter() {
            if value.to_lowercase().fuzzy_match(&keyword.to_lowercase()) {
                let builder = TileBuilder::new("/dev/skxxtz/sherlock/ui/tile.ui");
                builder.object.set_spawn_focus(launcher.spawn_focus);
                builder.object.set_shortcut(launcher.shortcut);

                if let Some(name) = &launcher.name {
                    builder.category.set_text(name);
                } else {
                    builder.category.set_visible(false);
                }
                builder.title.set_markup(&value);
                builder.icon.set_icon_name(Some(&proc.icon));
                let ppid = key.0;
                let cpid = key.1;
                let parent = ppid.to_string();
                let child = cpid.to_string();

                // Construct attrs and enable action capabilities
                let attrs = get_attrs_map(vec![
                    ("method", "kill-process"),
                    ("result", value),
                    ("keyword", keyword),
                    ("parent-pid", &parent),
                    ("child-pid", &child),
                ]);

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
                    priority: launcher.priority as f32,
                    row_item: builder.object,
                    shortcut_holder,
                    alias: launcher.alias.clone(),
                    home: launcher.home,
                    only_home: launcher.only_home,
                });
            }
        }
        return results;
    }
}
