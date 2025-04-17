use gdk_pixbuf::subclass::prelude::ObjectSubclassIsExt;
use gtk4::gdk::{Key, ModifierType, Rectangle};
use gtk4::{prelude::*, Box as HVBox, Label, ListBox, ListBoxRow, ScrolledWindow};

use crate::g_subclasses::sherlock_row::SherlockRow;
use crate::CONFIG;

pub fn execute_by_index(results: &ListBox, index: i32) {
    let mut child_counter = 1;
    for child in &results.observe_children() {
        if let Some(child) = child.ok() {
            if let Some(row) = child.downcast_ref::<SherlockRow>() {
                if row.imp().shortcut.get() {
                    if child_counter == index {
                        row.emit_by_name::<()>("row-should-activate", &[]);
                        return;
                    } else {
                        child_counter += 1
                    }
                }
            }
        }
    }
}

pub trait RowOperations {
    fn focus_next(&self, result_viewport: &ScrolledWindow);
    fn focus_prev(&self, result_viewport: &ScrolledWindow);
    fn focus_first(&self);
    fn select_offset_row(&self, offset: i32) -> ListBoxRow;
}

impl RowOperations for ListBox {
    fn focus_next(&self, result_viewport: &ScrolledWindow) {
        let new_row = self.select_offset_row(1);
        let allocation = result_viewport.allocation();
        let list_box_rect = Rectangle::from(allocation);

        let row_allocation = new_row.allocation();
        let row_rect = Rectangle::from(row_allocation);

        let list_height = list_box_rect.height() as f64;
        let row_end = (row_rect.y() + row_rect.height() + 14) as f64;
        let vadjustment = result_viewport.vadjustment();

        let current_value = vadjustment.value();
        let list_end = list_height + current_value;
        if row_end > list_end {
            let delta = row_end - list_end;
            let new_value = current_value + delta;
            vadjustment.set_value(new_value);
        }
    }
    fn focus_prev(&self, result_viewport: &ScrolledWindow) {
        let new_row = self.select_offset_row(-1);

        let row_allocation = new_row.allocation();
        let row_rect = Rectangle::from(row_allocation);

        let row_start = (row_rect.y()) as f64;
        let vadjustment = result_viewport.vadjustment();

        let current_value = vadjustment.value();
        if current_value > row_start {
            vadjustment.set_value(row_start);
        }
    }
    fn focus_first(&self) {
        for child in &self.observe_children() {
            if let Some(child) = child.ok() {
                if let Some(row) = child.downcast_ref::<SherlockRow>() {
                    if row.is_visible() && row.imp().spawn_focus.get() {
                        self.select_row(Some(row));
                        return;
                    }
                }
            }
        }
    }
    fn select_offset_row(&self, offset: i32) -> ListBoxRow {
        if let Some(row) = self.selected_row() {
            let mut new_index = row.index() + offset;
            while new_index >= 0 {
                if let Some(new_row) = self.row_at_index(new_index) {
                    if new_row.is_visible() {
                        self.select_row(Some(&new_row));
                        return new_row;
                    } else {
                        new_index += offset;
                    }
                } else {
                    return row;
                };
            }
        };
        return ListBoxRow::new();
    }
}

pub trait ShortCut {
    fn apply_shortcut(&self, index: i32, mod_str: &str) -> i32;
}
impl ShortCut for HVBox {
    fn apply_shortcut(&self, index: i32, mod_str: &str) -> i32 {
        if index < 6 {
            if let Some(child) = self.first_child() {
                if let Some(label) = child.downcast_ref::<Label>() {
                    self.set_visible(true);
                    label.set_text(&format!("{}", mod_str));
                }
            }
            if let Some(child) = self.last_child() {
                if let Some(label) = child.downcast_ref::<Label>() {
                    self.set_visible(true);
                    label.set_text(&format!("{}", index));
                    return 1;
                }
            }
        }
        return 0;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfKeys {
    pub next: Option<Key>,
    pub next_mod: Option<ModifierType>,
    pub prev: Option<Key>,
    pub prev_mod: Option<ModifierType>,
    pub shortcut_modifier: Option<ModifierType>,
    pub shortcut_modifier_str: String,
}
impl ConfKeys {
    pub fn new() -> Self {
        if let Some(c) = CONFIG.get() {
            let (prev_mod, prev) = match &c.binds.prev {
                Some(prev) => ConfKeys::eval_bind_combination(prev),
                _ => (None, None),
            };
            let (next_mod, next) = match &c.binds.next {
                Some(next) => ConfKeys::eval_bind_combination(next),
                _ => (None, None),
            };
            let shortcut_modifier = match &c.binds.modifier {
                Some(shortcut) => ConfKeys::eval_mod(shortcut),
                _ => Some(ModifierType::CONTROL_MASK),
            };
            let shortcut_modifier_str = ConfKeys::get_mod_str(&shortcut_modifier);
            return ConfKeys {
                next,
                next_mod,
                prev,
                prev_mod,
                shortcut_modifier,
                shortcut_modifier_str,
            };
        }
        ConfKeys::empty()
    }
    pub fn empty() -> Self {
        ConfKeys {
            next: None,
            next_mod: None,
            prev: None,
            prev_mod: None,
            shortcut_modifier: None,
            shortcut_modifier_str: String::new(),
        }
    }
    fn eval_bind_combination<T: AsRef<str>>(key: T) -> (Option<ModifierType>, Option<Key>) {
        let key_str = key.as_ref();
        match key_str.split("-").collect::<Vec<&str>>().as_slice() {
            [modifier, key, ..] => (ConfKeys::eval_mod(modifier), ConfKeys::eval_key(key)),
            [key, ..] => (None, ConfKeys::eval_key(key)),
            _ => (None, None),
        }
    }
    fn eval_key<T: AsRef<str>>(key: T) -> Option<Key> {
        match key.as_ref() {
            "tab" => Some(Key::Tab),
            "up" => Some(Key::Up),
            "down" => Some(Key::Down),
            "left" => Some(Key::Left),
            "right" => Some(Key::Right),
            "pgup" => Some(Key::Page_Up),
            "pgdown" => Some(Key::Page_Down),
            "end" => Some(Key::End),
            "home" => Some(Key::Home),
            _ => None,
        }
    }
    fn eval_mod(key: &str) -> Option<ModifierType> {
        match key {
            "shift" => Some(ModifierType::SHIFT_MASK),
            "control" => Some(ModifierType::CONTROL_MASK),
            "alt" => Some(ModifierType::ALT_MASK),
            "super" => Some(ModifierType::SUPER_MASK),
            "lock" => Some(ModifierType::LOCK_MASK),
            "hypr" => Some(ModifierType::HYPER_MASK),
            "meta" => Some(ModifierType::META_MASK),
            _ => None,
        }
    }
    fn get_mod_str(mod_key: &Option<ModifierType>) -> String {
        let k = match mod_key {
            Some(ModifierType::SHIFT_MASK) | Some(ModifierType::LOCK_MASK) => "⇧",
            Some(ModifierType::CONTROL_MASK) | Some(ModifierType::META_MASK) => "⌘",
            Some(ModifierType::ALT_MASK) => "⎇",
            Some(ModifierType::SUPER_MASK) | Some(ModifierType::HYPER_MASK) => "✦",
            _ => "⌘",
        };
        k.to_string()
    }
}
