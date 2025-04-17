use gio::glib::object::ObjectExt;
use gtk4::prelude::WidgetExt;
use meval::eval_str;
use std::collections::HashSet;

use super::util::TileBuilder;
use super::Tile;
use crate::{
    actions::{execute_from_attrs, get_attrs_map},
    g_subclasses::sherlock_row::SherlockRow,
    launcher::{calc_launcher::Calculator, Launcher, ResultItem},
};

impl Tile {
    pub fn calc_tile(
        launcher: &Launcher,
        calc_launcher: &Calculator,
        keyword: &str,
    ) -> Vec<ResultItem> {
        let capabilities: HashSet<&str> = match &calc_launcher.capabilities {
            Some(c) => c.iter().map(|s| s.as_str()).collect(),
            _ => HashSet::from(["calc.math", "calc.units"]),
        };
        let mut result: Option<String> = None;

        if capabilities.contains("calc.math") {
            let trimmed_keyword = keyword.trim();
            if let Ok(r) = eval_str(trimmed_keyword) {
                let r = r.to_string();
                if &r != trimmed_keyword {
                    result = Some(format!("= {}", r));
                }
            }
        }

        if (capabilities.contains("calc.lengths") || capabilities.contains("calc.units"))
            && result.is_none()
        {
            result = calc_launcher.measurement(&keyword, "lengths")
        }

        if (capabilities.contains("calc.weights") || capabilities.contains("calc.units"))
            && result.is_none()
        {
            result = calc_launcher.measurement(&keyword, "weights")
        }

        if (capabilities.contains("calc.volumes") || capabilities.contains("calc.units"))
            && result.is_none()
        {
            result = calc_launcher.measurement(&keyword, "volumes")
        }

        if (capabilities.contains("calc.temperatures") || capabilities.contains("calc.units"))
            && result.is_none()
        {
            result = calc_launcher.temperature(&keyword)
        }

        if let Some(r) = result {
            let builder = TileBuilder::new("/dev/skxxtz/sherlock/ui/calc_tile.ui");
            builder.object.add_css_class("calc-tile");
            builder.object.set_spawn_focus(launcher.spawn_focus);
            builder.object.set_shortcut(launcher.shortcut);
            builder.object.set_search(String::from("always-flag"));

            builder.equation_holder.set_text(&keyword);
            builder.result_holder.set_text(&r);

            // Add action capabilities
            let attrs = get_attrs_map(vec![("method", &launcher.method), ("result", &r)]);
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

            vec![res]
        } else {
            return vec![];
        }
    }
}
