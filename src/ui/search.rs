use futures::future::join_all;
use gio::ActionEntry;
use gtk4::{
    self,
    gdk::{self, Key, ModifierType},
    prelude::*,
    Builder, EventControllerKey, Image, Overlay, Spinner,
};
use gtk4::{glib, ApplicationWindow, Entry};
use gtk4::{Box as HVBox, Label, ListBox, ScrolledWindow};
use simd_json::prelude::ArrayTrait;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::tiles::util::{AsyncLauncherTile, SherlockSearch};
use super::util::*;
use crate::actions::execute_from_attrs;
use crate::g_subclasses::sherlock_row::SherlockRow;
use crate::launcher::{Launcher, ResultItem};
use crate::CONFIG;

#[allow(dead_code)]
struct SearchUI {
    result_viewport: ScrolledWindow,
    // will be later used for split view to display information about apps/commands
    preview_box: HVBox,
    search_bar: Entry,
    search_icon_holder: HVBox,
    mode_title: Label,
    spinner: Spinner,
}

pub fn search(
    launchers: &Vec<Launcher>,
    window: &ApplicationWindow,
    stack_page_ref: &Rc<RefCell<String>>,
) -> HVBox {
    // Initialize the view to show all apps
    let (mode, modes, stack_page, ui, results) = construct_window(&launchers);
    ui.result_viewport
        .set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);

    let initial_mode = mode.borrow().clone();
    let search_bar_clone = ui.search_bar.clone();
    let search_bar_clone2 = ui.search_bar.clone();
    let modes_clone = modes.clone();
    let mode_clone = Rc::clone(&mode);

    let search_bar = ui.search_bar.clone();
    stack_page.connect_realize(move |_| {
        search_bar.grab_focus();
    });

    let custom_binds = ConfKeys::new();



    // Initialize all tiles
    let (async_launchers, non_async_launchers): (Vec<Launcher>, Vec<Launcher>) = launchers
        .clone()
        .into_iter()
        .partition(|launcher| launcher.r#async);
    let mut patches: Vec<ResultItem> = non_async_launchers
        .into_iter()
        .map(|launcher| launcher.get_patch(""))
        .flatten()
        .collect();
    // Get asynchronous loader tiles
    let loaders: Vec<AsyncLauncherTile> = async_launchers
        .into_iter()
        .filter_map(|launcher| launcher.get_loader_widget(""))
        .map(|(launcher, row)| {
            patches.push(row);
            launcher
        })
        .collect();
    patches.sort_by(|a, b| a.priority.partial_cmp(&b.priority).unwrap());

    change_event(
        &ui.search_bar,
        modes,
        &mode,
        &results,
        &custom_binds,
        loaders,
        patches,
    );
    nav_event(
        &stack_page,
        results,
        ui.search_bar,
        ui.result_viewport,
        custom_binds,
        stack_page_ref,
    );

    // Improved mode selection
    let mode_action = ActionEntry::builder("switch-mode")
        .parameter_type(Some(&String::static_variant_type()))
        .state(initial_mode.to_variant())
        .activate(move |_, action, parameter| {
            let state = action.state().and_then(|s| s.get::<String>());
            let parameter = parameter.and_then(|p| p.get::<String>());

            if let (Some(mut state), Some(mut parameter)) = (state, parameter) {
                match parameter.as_str() {
                    "search" => {
                        ui.search_icon_holder.set_css_classes(&["back"]);
                        ui.mode_title.set_text("Search");
                    }
                    _ => {
                        parameter.push_str(" ");
                        let mode_name = modes_clone.get(&parameter);
                        match mode_name {
                            Some(name) => {
                                ui.search_icon_holder.set_css_classes(&["back"]);
                                *mode_clone.borrow_mut() = parameter.clone();
                                ui.mode_title.set_text(name.as_deref().unwrap_or_default());
                                state = parameter;
                            }
                            _ => {
                                ui.search_icon_holder.set_css_classes(&["search"]);
                                ui.mode_title.set_text("All");
                                parameter = String::from("all ");
                                *mode_clone.borrow_mut() = parameter.clone();
                                state = parameter;
                            }
                        }
                        let search_bar_clone = search_bar_clone.clone();
                        glib::idle_add_local(move || {
                            // to trigger homescreen rebuild
                            search_bar_clone.set_text("\n");
                            search_bar_clone.set_text("");
                            glib::ControlFlow::Break
                        });
                        action.set_state(&state.to_variant());
                    }
                }
            }
        })
        .build();

    // Spinner action
    let action_spinner = ActionEntry::builder("spinner-mode")
        .parameter_type(Some(&bool::static_variant_type()))
        .activate(move |_, _, parameter| {
            let parameter = parameter.and_then(|p| p.get::<bool>());
            parameter.map(|p| {
                if p {
                    ui.spinner.set_css_classes(&["spinner-appear"]);
                } else {
                    ui.spinner.set_css_classes(&["spinner-disappear"]);
                };
                ui.spinner.set_spinning(p);
            });
        })
        .build();

    let action_clear_win = ActionEntry::builder("clear-search")
        .activate(move |_: &ApplicationWindow, _, _| {
            let search_bar_clone = search_bar_clone2.clone();
            glib::idle_add_local(move || {
                search_bar_clone.set_text("");
                glib::ControlFlow::Break
            });
        })
        .build();
    window.add_action_entries([mode_action, action_clear_win, action_spinner]);

    return stack_page;
}

fn construct_window(
    launchers: &Vec<Launcher>,
) -> (
    Rc<RefCell<String>>,
    HashMap<String, Option<String>>,
    HVBox,
    SearchUI,
    Rc<ListBox>,
) {
    // Collect Modes
    let original_mode = CONFIG
        .get()
        .and_then(|c| c.behavior.sub_menu.as_deref())
        .unwrap_or("all");
    let mode = Rc::new(RefCell::new(original_mode.to_string()));
    let modes: HashMap<String, Option<String>> = launchers
        .iter()
        .filter_map(|item| item.alias.as_ref().map(|alias| (alias, &item.name)))
        .map(|(alias, name)| (format!("{} ", alias), name.clone()))
        .collect();

    // Initialize the builder with the correct path
    let builder = Builder::from_resource("/dev/skxxtz/sherlock/ui/search.ui");

    // Get the required object references
    let vbox: HVBox = builder.object("vbox").unwrap();
    let results: Rc<ListBox> = Rc::new(builder.object("result-frame").unwrap());

    let search_icon_holder: HVBox = builder.object("search-icon-holder").unwrap_or_default();
    search_icon_holder.add_css_class("search");
    // Create the search icon
    let search_icon = Image::new();
    search_icon.set_icon_name(Some("search"));
    search_icon.set_widget_name("search-icon");
    search_icon.set_halign(gtk4::Align::End);
    // Create the back arrow
    let search_icon_back = Image::new();
    search_icon_back.set_icon_name(Some("go-previous"));
    search_icon_back.set_widget_name("search-icon-back");
    search_icon_back.set_halign(gtk4::Align::End);

    let overlay = Overlay::new();
    overlay.set_child(Some(&search_icon));
    overlay.add_overlay(&search_icon_back);

    // Show notification-bar
    CONFIG.get().map(|c| {
        if !c.appearance.status_bar {
            let n: Option<HVBox> = builder.object("status-bar");
            n.map(|n| n.set_visible(false));
        }
    });

    search_icon_holder.append(&overlay);

    let ui = SearchUI {
        result_viewport: builder.object("scrolled-window").unwrap_or_default(),
        preview_box: builder.object("preview_box").unwrap_or_default(),
        search_bar: builder.object("search-bar").unwrap_or_default(),
        search_icon_holder,
        mode_title: builder.object("category-type-label").unwrap_or_default(),
        spinner: builder.object("status-bar-spinner").unwrap_or_default(),
    };
    CONFIG.get().map(|c| {
        ui.result_viewport
            .set_size_request((c.appearance.width as f32 * 0.4) as i32, 10);
        ui.search_icon_holder.set_visible(c.appearance.search_icon);
        search_icon.set_pixel_size(c.appearance.icon_size);
        search_icon_back.set_pixel_size(c.appearance.icon_size);
    });

    (mode, modes, vbox, ui, results)
}

fn nav_event(
    stack: &HVBox,
    results: Rc<ListBox>,
    search_bar: Entry,
    result_viewport: ScrolledWindow,
    custom_binds: ConfKeys,
    stack_page: &Rc<RefCell<String>>,
) {
    let stack_page = Rc::clone(stack_page);
    let event_controller = EventControllerKey::new();
    event_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    event_controller.connect_key_pressed(move |_, key, i, modifiers| {
        if stack_page.borrow().as_str() != "search-page" {
            return false.into();
        };
        match key {
            k if Some(k) == custom_binds.prev
                && custom_binds
                    .prev_mod
                    .map_or(true, |m| modifiers.contains(m)) =>
            {
                results.focus_prev(&result_viewport);
                return true.into();
            }
            k if Some(k) == custom_binds.next
                && custom_binds
                    .next_mod
                    .map_or(true, |m| modifiers.contains(m)) =>
            {
                results.focus_next(&result_viewport);
                return true.into();
            }
            gdk::Key::Up => {
                results.focus_prev(&result_viewport);
            }
            gdk::Key::Down => {
                results.focus_next(&result_viewport);
                return true.into();
            }
            gdk::Key::BackSpace => {
                let mut ctext = search_bar.text().to_string();
                if custom_binds
                    .shortcut_modifier
                    .map_or(false, |modifier| modifiers.contains(modifier))
                {
                    let _ = search_bar.set_text("");
                    ctext.clear();
                }
                if ctext.is_empty() {
                    let _ =
                        search_bar.activate_action("win.switch-mode", Some(&"all".to_variant()));
                }
                results.focus_first();
            }
            gdk::Key::Return => {
                if let Some(row) = results.selected_row().and_downcast_ref::<SherlockRow>() {
                    row.emit_by_name::<()>("row-should-activate", &[]);
                }
            }
            Key::_1 | Key::_2 | Key::_3 | Key::_4 | Key::_5 => {
                if custom_binds
                    .shortcut_modifier
                    .map_or(false, |modifier| modifiers.contains(modifier))
                {
                    let key_index = match key {
                        Key::_1 => 1,
                        Key::_2 => 2,
                        Key::_3 => 3,
                        Key::_4 => 4,
                        Key::_5 => 5,
                        _ => return false.into(),
                    };
                    execute_by_index(&*results, key_index);
                    return true.into();
                }
            }
            // Pain - solution for shift-tab since gtk handles it as an individual event
            _ if i == 23 && modifiers.contains(ModifierType::SHIFT_MASK) => {
                let shift = Some(ModifierType::SHIFT_MASK);
                let tab = Some(Key::Tab);
                if custom_binds.prev_mod == shift && custom_binds.prev == tab {
                    results.focus_prev(&result_viewport);
                    return true.into();
                } else if custom_binds.next_mod == shift && custom_binds.next == tab {
                    results.focus_next(&result_viewport);
                    return true.into();
                }
            }
            _ => (),
        }
        false.into()
    });

    stack.add_controller(event_controller);
}

fn change_event(
    search_bar: &Entry,
    modes: HashMap<String, Option<String>>,
    mode: &Rc<RefCell<String>>,
    results: &Rc<ListBox>,
    custom_binds: &ConfKeys,
    loaders: Vec<AsyncLauncherTile>,
    patches: Vec<ResultItem>,
) {
    // Setting up async capabilities
    let current_task: Rc<RefCell<Option<glib::JoinHandle<()>>>> = Rc::new(RefCell::new(None));

    // vars
    let mod_str = custom_binds.shortcut_modifier_str.clone();
    let async_launchers = Rc::new(loaders);

    if let Some(c) = CONFIG.get() {
        let mut shortcut_index = 1;
        for widget in patches.iter() {
            widget.row_item.set_visible(widget.home);
            if c.behavior.animate {
                widget.row_item.add_css_class("animate");
            }
            if let Some(shortcut_holder) = &widget.shortcut_holder {
                shortcut_index += shortcut_holder.apply_shortcut(shortcut_index, &mod_str);
            }
            results.append(&widget.row_item);
        }
    }
    async_calc(&async_launchers, &current_task, String::new());
    results.focus_first();

    let rc_patches: Rc<Vec<ResultItem>> = Rc::new(patches);

    search_bar.connect_changed({
        let patches = Rc::clone(&rc_patches);
        let results = Rc::clone(&results);
        let mode_clone = Rc::clone(mode);
        move |search_bar| {
            let mut current_text = search_bar.text().to_string();
            if current_text.len() == 1 && current_text != "\n" {
                let _ = search_bar.activate_action("win.switch-mode", Some(&"search".to_variant()));
            };
            if let Some(task) = current_task.borrow_mut().take() {
                task.abort();
            };
            let trimmed = current_text.trim();
            if !trimmed.is_empty() && modes.contains_key(&current_text) {
                // Logic to apply modes
                let _ = search_bar.activate_action("win.switch-mode", Some(&trimmed.to_variant()));
                current_text.clear();
            }
            // Update tile visibility
            let current_mode = mode_clone.borrow().clone();
            let current_mode = current_mode.trim();
            let all = current_mode == "all";
            if all {
                if current_text.trim().is_empty() {
                    patches.iter().for_each(|l| l.row_item.set_visible(l.home))
                } else {
                    patches
                        .iter()
                        .for_each(|r| {
                            let visible = match r {
                                r if r.only_home => false,
                                r if r.priority == 0.0 => r.alias.as_deref() == Some(current_mode),
                                r => r.row_item.get_search().fuzzy_match(&current_text)
                            };
                            r.row_item.set_visible(visible);
                        });
                }
            } else {
                patches
                    .iter()
                    .for_each(|r| r.row_item.set_visible(r.alias.as_deref() == Some(current_mode)));

            }
            results.focus_first();
        }
    });
}

/// Does the asynchronous retrieving and updating for async tiles. 
pub fn async_calc(
    launchers: &Rc<Vec<AsyncLauncherTile>>,
    current_task: &Rc<RefCell<Option<glib::JoinHandle<()>>>>,
    current_text: String,
) {
    // If task is currently running, abort it
    if let Some(t) = current_task.borrow_mut().take() {
        t.abort();
    };

    // Gather results for asynchronous widgets
    let task = glib::MainContext::default().spawn_local({
        let current_task_clone = Rc::clone(current_task);
        let launchers = Rc::clone(&launchers);
        async move {
            if let Some(row) = launchers.get(0) {
                let _ = row
                    .result_item
                    .row_item
                    .activate_action("win.spinner-mode", Some(&true.to_variant()));
            }
            // Make them update concurrently
            let futures: Vec<_> = launchers
                .iter()
                .map(|widget| {
                    let widget_clone = widget;
                    let current_text = current_text.clone();
                    async move {
                        let mut attrs = widget_clone.attrs.clone();

                        // Process text tile
                        if let Some(opts) = &widget_clone.text_tile {
                            if let Some((title, body, next_content)) =
                                widget_clone.launcher.get_result(&current_text).await
                            {
                                opts.title.set_text(&title);
                                opts.body.set_text(&body);
                                if let Some(next_content) = next_content {
                                    attrs.insert(
                                        String::from("next_content"),
                                        next_content.to_string(),
                                    );
                                }
                            }
                        }

                        // Process image replacement
                        if let Some(opts) = &widget_clone.image_replacement {
                            if let Some((image, was_cached)) =
                                widget_clone.launcher.get_image().await
                            {
                                if !was_cached {
                                    opts.icon_holder_overlay.add_css_class("image-replace-overlay");
                                }
                                let gtk_image = Image::from_pixbuf(Some(&image));
                                gtk_image.set_widget_name("album-cover");
                                gtk_image.set_pixel_size(50);
                                opts.icon_holder_overlay.add_overlay(&gtk_image);
                            }
                        }

                        // Process weather tile
                        if let Some(wtr) = &widget_clone.weather_tile {
                            if let Some((data, was_changed)) =
                                widget_clone.launcher.get_weather().await
                            {
                                let css_class = if was_changed {
                                    "weather-animate"
                                } else {
                                    "weather-no-animate"
                                };
                                widget_clone.result_item.row_item.add_css_class(css_class);
                                widget_clone.result_item.row_item.add_css_class(&data.icon);
                                wtr.temperature.set_text(&data.temperature);
                                wtr.spinner.set_spinning(false);
                                wtr.icon.set_icon_name(Some(&data.icon));
                                wtr.location.set_text(&data.format_str);
                            } else {
                                widget_clone.result_item.row_item.set_visible(false);
                            }
                        }

                        // Connect row-should-activate signal
                        widget_clone.result_item.row_item.connect(
                            "row-should-activate",
                            false,
                            move |row| {
                                let row = row.first().map(|f| f.get::<SherlockRow>().ok())??;
                                execute_from_attrs(&row, &attrs);
                                None
                            },
                        );
                    }
                })
                .collect();

            let _ = join_all(futures).await;
            // Set spinner inactive
            if let Some(row) = launchers.get(0) {
                let _ = row
                    .result_item
                    .row_item
                    .activate_action("win.spinner-mode", Some(&false.to_variant()));
            }
            *current_task_clone.borrow_mut() = None;
        }
    });
    *current_task.borrow_mut() = Some(task);
}

