use std::collections::HashMap;

use gio::glib::Bytes;
use gtk4::prelude::{BoxExt, WidgetExt};
use gtk4::{gdk, Image, Overlay};

use super::util::{AsyncLauncherTile, ImageReplacementElements, TileBuilder};
use super::Tile;
use crate::launcher::audio_launcher::MusicPlayerLauncher;
use crate::launcher::{Launcher, ResultItem};

impl Tile {
    pub fn mpris_tile(
        launcher: Launcher,
        mpris: &MusicPlayerLauncher,
    ) -> Option<(AsyncLauncherTile, ResultItem)> {
        let builder = TileBuilder::new("/dev/skxxtz/sherlock/ui/mpris_tile.ui");
        builder.object.add_css_class("mpris-tile");
        builder.object.set_spawn_focus(launcher.spawn_focus);
        builder.object.set_shortcut(launcher.shortcut);

        builder
            .category
            .set_text(&mpris.mpris.metadata.artists.join(", "));
        builder.title.set_text(&mpris.mpris.metadata.title);
        builder.object.set_overflow(gtk4::Overflow::Hidden);

        let overlay = Overlay::new();

        builder.icon.set_visible(false);

        let pix_buf = vec![0, 0, 0];
        let image_buf = gdk::gdk_pixbuf::Pixbuf::from_bytes(
            &Bytes::from_owned(pix_buf),
            gdk::gdk_pixbuf::Colorspace::Rgb,
            false,
            8,
            1,
            1,
            3,
        );
        if let Some(image_buf) =
            image_buf.scale_simple(30, 30, gdk::gdk_pixbuf::InterpType::Nearest)
        {
            let image = Image::from_pixbuf(Some(&image_buf));
            overlay.set_child(Some(&image));
            image.set_widget_name("placeholder-icon");
            image.set_pixel_size(50);
        };
        builder.icon_holder.append(&overlay);

        builder.icon_holder.set_overflow(gtk4::Overflow::Hidden);
        builder.icon_holder.set_widget_name("mpris-icon-holder");
        builder.icon_holder.set_margin_top(10);
        builder.icon_holder.set_margin_bottom(10);

        // Add attrs and implement double click capabilities
        let attrs: HashMap<String, String> =
            vec![("method", &launcher.method), ("player", &mpris.player)]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

        // Make shortcut holder
        let shortcut_holder = match launcher.shortcut {
            true => builder.shortcut_holder,
            _ => None,
        };
        let result_item = ResultItem {
            priority: launcher.priority as f32,
            row_item: builder.object,
            shortcut_holder,
            alias: launcher.alias.clone(),
            home: launcher.home,
            only_home: launcher.only_home,
        };

        let options = ImageReplacementElements { icon_holder_overlay: overlay };

        return Some((AsyncLauncherTile {
            launcher,
            result_item: result_item.clone(),
            text_tile: None,
            image_replacement: Some(options),
            weather_tile: None,
            attrs,
        }, result_item));
    }
}
