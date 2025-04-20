use gio::glib::object::ObjectExt;
use gtk4::{prelude::WidgetExt, Label};
use meval::eval_str;
use std::collections::HashSet;

use super::util::TileBuilder;
use super::Tile;
use crate::{
    actions::{execute_from_attrs, get_attrs_map},
    g_subclasses::sherlock_row::SherlockRow,
    launcher::{calc_launcher::{self, Calculator}, Launcher, ResultItem},
};

impl Tile {
    pub fn calc_tile(
        launcher: &Launcher,
        calc_launcher: &Calculator,
        keyword: &str,
    ) -> Vec<ResultItem> {
        let capabilities: HashSet<String> = match &calc_launcher.capabilities {
            Some(c) => c.clone(),
            _ => HashSet::from([String::from("calc.math"), String::from("calc.units")]),
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
        let builder = TileBuilder::new("/dev/skxxtz/sherlock/ui/calc_tile.ui");

        builder.object.add_css_class("calc-tile");
        builder.object.set_spawn_focus(launcher.spawn_focus);
        builder.object.set_shortcut(launcher.shortcut);
        builder.object.set_search(String::from("always-flag"));
        builder.object.set_visible(false);


        // Add action capabilities
        let attrs = get_attrs_map(vec![("method", &launcher.method), ("result", &r)]);
        builder
            .object
            .connect("row-should-activate", false, move |row| {
                let row = row.first().map(|f| f.get::<SherlockRow>().ok())??;
                execute_from_attrs(&row, &attrs);
                None
            });
        let key_string = keyword.to_string();
        builder
            .object
            .connect("row-update", false, move |params| {
                if params.len() >= 2 {
                    let row = params.first().map(|f| f.get::<SherlockRow>().ok())??;
                    let keyword = params[1].get::<String>().ok()?;
                    update_calc_tile(&row, &capabilities, &keyword, &builder.equation_holder, &builder.result_holder);

                }
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
    }
}

fn update_calc_tile(row: &SherlockRow, capabilities: &HashSet<String>, keyword: &str, equation_holder: &Label, result_holder: &Label){
        let mut result = None;
        if (capabilities.contains("calc.lengths") || capabilities.contains("calc.units"))
            && result.is_none()
        {
            result = Calculator::measurement(&keyword, "lengths")
        }

        if (capabilities.contains("calc.weights") || capabilities.contains("calc.units"))
            && result.is_none()
        {
            result = Calculator::measurement(&keyword, "weights")
        }

        if (capabilities.contains("calc.volumes") || capabilities.contains("calc.units"))
            && result.is_none()
        {
            result = Calculator::measurement(&keyword, "volumes")
        }

        if (capabilities.contains("calc.temperatures") || capabilities.contains("calc.units"))
            && result.is_none()
        {
            result = Calculator::temperature(&keyword)
        }
        row.set_visible(result.is_some());
        equation_holder.set_text(&keyword);
        result_holder.set_text(&result.map_or(String::new(), |s|s));
}
