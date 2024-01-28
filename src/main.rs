// #![allow(unused)]

use colored::{ColoredString, Colorize};
use std::collections::BTreeMap;
use zellij_tile::prelude::*;

#[derive(Default)]
struct State {
    tabs: Vec<TabInfo>,
    filter: String,
    selected: usize,
    ignore_case: bool,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        // ReadApplicationState permission to receive the ModeUpdate and TabUpdate events
        // ChangeApplicationState permission to Change Zellij state (Panes, Tabs and UI)
        request_permission(&[PermissionType::ReadApplicationState, PermissionType::ChangeApplicationState]);

        self.ignore_case = match configuration.get("ignore_case") {
            Some(value) => value.trim().parse().unwrap(),
            None => true,
        };

        subscribe(&[EventType::TabUpdate, EventType::Key]);
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        let selected = self.selected;

        let plugin_id = get_plugin_ids().plugin_id;
        rename_plugin_pane(plugin_id, "tab-finder");

        match event {
            Event::TabUpdate(tab_info) => {
                self.tabs = tab_info;
                should_render = true;
            }
            Event::Key(Key::Esc | Key::Ctrl('c')) => close_focus(),
            Event::Key(Key::Up) => {
                self.selected = if selected == 0 { self.tabs.len() - 1 } else { selected - 1 };
                should_render = true;
            }
            Event::Key(Key::Down) => {
                self.selected = if selected == self.tabs.len() - 1 { 0 } else { selected + 1 };
                should_render = true;
            }
            Event::Key(Key::Char('\n')) => {
                close_focus();
                switch_tab_to(self.selected as u32 + 1);
            }
            Event::Key(Key::Backspace) => {
                self.filter.pop();
                if self.filter.is_empty() {
                    self.selected = 0;
                }
                should_render = true;
            }
            Event::Key(Key::Char(c)) => {
                self.filter.push(c);
                should_render = true;
            }
            _ => (),
        };

        should_render
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        let filter = self.filter.as_str();

        let condition = |tab: &&TabInfo| {
            if self.ignore_case {
                tab.name.clone().to_lowercase().contains(&filter.to_lowercase())
            } else {
                tab.name.contains(filter)
            }
        };

        let prompt = if filter.is_empty() {
            String::new()
        } else {
            self.selected = self.tabs.iter().find(condition).map(|tab| tab.position).unwrap_or(0);
            format!("{} {}\n", ">".magenta(), filter.bright_green().italic())
        };

        let to_line = |tab: &TabInfo| {
            let line_text = format!(
                "{}, {} {}",
                tab.position + 1,
                &(tab.name),
                if tab.active { "(*)" } else { "" },
            )
            .cyan();

            if tab.position == self.selected {
                line_text.on_bright_magenta().black()
            } else {
                line_text
            }
        };

        let tabs = self
            .tabs
            .iter()
            .filter(condition)
            .map(to_line)
            .collect::<Vec<ColoredString>>();

        print!("{}", prompt);
        for tab in tabs {
            println!("{}", tab)
        }
    }
}
