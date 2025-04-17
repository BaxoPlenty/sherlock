use gtk4::prelude::*;
use std::collections::HashMap;

use crate::launcher::{Launcher, ResultItem};

use super::util::{AsyncLauncherTile, WeatherTileBuilder, WeatherTileElements};
use super::Tile;

impl Tile {
    pub fn weather_tile_loader(launcher: Launcher) -> Option<(AsyncLauncherTile, ResultItem)> {
        let builder = WeatherTileBuilder::new("/dev/skxxtz/sherlock/ui/weather_tile.ui");
        builder.object.add_css_class("weather-tile");
        builder.object.set_spawn_focus(launcher.spawn_focus);
        builder.object.set_shortcut(launcher.shortcut);

        // Add attrs and implement double click capabilities
        let attrs: HashMap<String, String> = vec![("method", &launcher.method)]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        builder.icon.set_pixel_size(40);

        let result_item = ResultItem {
            priority: launcher.priority as f32,
            row_item: builder.object,
            shortcut_holder: None,
            alias: launcher.alias.clone(),
            home: launcher.home,
            only_home: launcher.only_home,
        };
        let weather_tile = Some(WeatherTileElements {
            icon: builder.icon,
            temperature: builder.temperature,
            location: builder.location,
            spinner: builder.spinner,
        });
        return Some((AsyncLauncherTile {
            launcher,
            result_item: result_item.clone(),
            text_tile: None,
            image_replacement: None,
            weather_tile,
            attrs,
        }, result_item));
    }
}
