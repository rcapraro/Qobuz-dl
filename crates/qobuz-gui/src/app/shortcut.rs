//! Keyboard shortcuts, mapped from raw key presses. Pure, so the mapping is
//! tested without a window.
//!
//! No shortcut is a command+character combination: on macOS such a press still
//! carries its character, which a focused text field types in.

use super::Screen;
use iced::event::Status;
use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Shortcut {
    FocusSearch,
    NextTab,
    PreviousTab,
    Escape,
    AddSelected,
}

/// The shortcut a key press triggers, if any. `status` is whether a widget
/// already handled the press: a focused text field captures nearly every key,
/// so shortcuts that never type anything fire either way, while `/` and
/// command+Enter fire only when no field took them.
pub(super) fn shortcut(key: &Key, modifiers: Modifiers, status: Status) -> Option<Shortcut> {
    let ignored = status == Status::Ignored;
    match key.as_ref() {
        Key::Named(Named::Escape) => Some(Shortcut::Escape),
        Key::Named(Named::Tab) if modifiers.control() => Some(if modifiers.shift() {
            Shortcut::PreviousTab
        } else {
            Shortcut::NextTab
        }),
        Key::Named(Named::Enter) if modifiers.command() && ignored => Some(Shortcut::AddSelected),
        // Shift is allowed (AZERTY needs it for `/`), and so is Ctrl+Alt, which
        // is how Windows reports AltGr; a plain ⌘ or Ctrl chord is not.
        Key::Character("/") if ignored && !chorded(modifiers) => Some(Shortcut::FocusSearch),
        _ => None,
    }
}

fn chorded(modifiers: Modifiers) -> bool {
    (modifiers.command() || modifiers.control()) && !modifiers.alt()
}

/// The tab after `screen` in tab-bar order, wrapping around.
pub(super) fn next_tab(screen: Screen) -> Screen {
    match screen {
        Screen::Search => Screen::Queue,
        Screen::Queue => Screen::Settings,
        Screen::Settings => Screen::Search,
    }
}

/// The tab before `screen` in tab-bar order, wrapping around.
pub(super) fn previous_tab(screen: Screen) -> Screen {
    match screen {
        Screen::Search => Screen::Settings,
        Screen::Queue => Screen::Search,
        Screen::Settings => Screen::Queue,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command() -> Modifiers {
        if cfg!(target_os = "macos") {
            Modifiers::LOGO
        } else {
            Modifiers::CTRL
        }
    }

    fn slash() -> Key {
        Key::Character("/".into())
    }

    #[test]
    fn control_tab_cycles_even_when_captured() {
        let tab = Key::Named(Named::Tab);
        for status in [Status::Ignored, Status::Captured] {
            assert_eq!(
                shortcut(&tab, Modifiers::CTRL, status),
                Some(Shortcut::NextTab)
            );
            assert_eq!(
                shortcut(&tab, Modifiers::CTRL | Modifiers::SHIFT, status),
                Some(Shortcut::PreviousTab)
            );
        }
        assert_eq!(shortcut(&tab, Modifiers::empty(), Status::Ignored), None);
    }

    #[test]
    fn tabs_wrap_in_tab_bar_order() {
        let mut screen = Screen::Search;
        for expected in [Screen::Queue, Screen::Settings, Screen::Search] {
            screen = next_tab(screen);
            assert_eq!(screen, expected);
        }
        assert_eq!(previous_tab(Screen::Search), Screen::Settings);
    }

    #[test]
    fn slash_focuses_search_only_when_not_typing() {
        assert_eq!(
            shortcut(&slash(), Modifiers::empty(), Status::Ignored),
            Some(Shortcut::FocusSearch)
        );
        assert_eq!(
            shortcut(&slash(), Modifiers::SHIFT, Status::Ignored),
            Some(Shortcut::FocusSearch)
        );
        assert_eq!(
            shortcut(&slash(), Modifiers::empty(), Status::Captured),
            None
        );
        assert_eq!(shortcut(&slash(), command(), Status::Ignored), None);
        assert_eq!(shortcut(&slash(), Modifiers::CTRL, Status::Ignored), None);
    }

    #[test]
    fn slash_typed_with_altgr_focuses_search() {
        let altgr = Modifiers::CTRL | Modifiers::ALT;
        assert_eq!(
            shortcut(&slash(), altgr, Status::Ignored),
            Some(Shortcut::FocusSearch)
        );
    }

    #[test]
    fn command_characters_are_not_shortcuts() {
        for c in ["f", "1", "2", "3"] {
            assert_eq!(
                shortcut(&Key::Character(c.into()), command(), Status::Ignored),
                None
            );
        }
    }

    #[test]
    fn escape_maps_whatever_the_status() {
        let esc = Key::Named(Named::Escape);
        assert_eq!(
            shortcut(&esc, Modifiers::empty(), Status::Captured),
            Some(Shortcut::Escape)
        );
    }

    #[test]
    fn command_enter_only_when_no_field_took_it() {
        let enter = Key::Named(Named::Enter);
        assert_eq!(
            shortcut(&enter, command(), Status::Ignored),
            Some(Shortcut::AddSelected)
        );
        assert_eq!(shortcut(&enter, command(), Status::Captured), None);
        assert_eq!(shortcut(&enter, Modifiers::empty(), Status::Ignored), None);
    }
}
