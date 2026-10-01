//! Key specifications of the control protocol (`key ctrl+s`, `key code:Backquote`)
//! and the keyboard events a platform reports for them.

use iced::Event;
use iced::keyboard::key::{Code, Named, NativeCode, Physical};
use iced::keyboard::{self, Key, Modifiers};

/// A key press as a platform reports it: `key` without the modifiers,
/// `modified_key` with Shift applied, the physical key, and the text it
/// types, if any.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Keystroke {
    pub(crate) key: Key,
    pub(crate) modified_key: Key,
    pub(crate) physical: Physical,
    pub(crate) modifiers: Modifiers,
    pub(crate) text: Option<char>,
}

/// Named keys, canonical names first, then aliases.
const KEYS: &[(&str, Named)] = &[
    ("enter", Named::Enter),
    ("escape", Named::Escape),
    ("tab", Named::Tab),
    ("backspace", Named::Backspace),
    ("delete", Named::Delete),
    ("insert", Named::Insert),
    ("space", Named::Space),
    ("arrowup", Named::ArrowUp),
    ("arrowdown", Named::ArrowDown),
    ("arrowleft", Named::ArrowLeft),
    ("arrowright", Named::ArrowRight),
    ("home", Named::Home),
    ("end", Named::End),
    ("pageup", Named::PageUp),
    ("pagedown", Named::PageDown),
    ("f1", Named::F1),
    ("f2", Named::F2),
    ("f3", Named::F3),
    ("f4", Named::F4),
    ("f5", Named::F5),
    ("f6", Named::F6),
    ("f7", Named::F7),
    ("f8", Named::F8),
    ("f9", Named::F9),
    ("f10", Named::F10),
    ("f11", Named::F11),
    ("f12", Named::F12),
    ("up", Named::ArrowUp),
    ("down", Named::ArrowDown),
    ("left", Named::ArrowLeft),
    ("right", Named::ArrowRight),
    ("esc", Named::Escape),
    ("return", Named::Enter),
    ("del", Named::Delete),
    ("pgup", Named::PageUp),
    ("pgdown", Named::PageDown),
    ("pgdn", Named::PageDown),
];

/// Every key name `key` accepts, comma separated, for help and errors.
pub(crate) fn key_names() -> String {
    let mut names: Vec<&str> = KEYS.iter().map(|(name, _)| *name).collect();
    names.push("code:<Name>");
    names.join(", ")
}

impl Keystroke {
    /// `[ctrl+][shift+][alt+][super+]KEY`, modifiers in any order. `KEY` is
    /// one character, a key name, or `code:<Name>` for a physical key such as
    /// `code:Backquote`.
    pub(crate) fn parse(spec: &str) -> Result<Self, String> {
        let mut modifiers = Modifiers::empty();
        let mut rest = spec;
        'prefixes: loop {
            for (prefix, flag) in [
                ("ctrl+", Modifiers::CTRL),
                ("shift+", Modifiers::SHIFT),
                ("alt+", Modifiers::ALT),
                ("super+", Modifiers::LOGO),
            ] {
                if rest.len() > prefix.len()
                    && rest.is_char_boundary(prefix.len())
                    && rest[..prefix.len()].eq_ignore_ascii_case(prefix)
                {
                    modifiers |= flag;
                    rest = &rest[prefix.len()..];
                    continue 'prefixes;
                }
            }
            break;
        }
        if let Some(name) = rest.strip_prefix("code:") {
            let code = physical_code(name).ok_or_else(|| format!("unknown key code {name:?}"))?;
            return Ok(Self {
                key: Key::Unidentified,
                modified_key: Key::Unidentified,
                physical: Physical::Code(code),
                modifiers,
                text: None,
            });
        }
        let mut chars = rest.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => Ok(Self::character(c, modifiers)),
            _ => {
                let named = named_key(rest)
                    .ok_or_else(|| format!("unknown key {rest:?}; keys: {}", key_names()))?;
                Ok(Self::named(named, modifiers))
            }
        }
    }

    /// A character key. Shift is applied the way a US layout applies it to
    /// letters; other characters are taken as given.
    pub(crate) fn character(c: char, modifiers: Modifiers) -> Self {
        if c == ' ' {
            return Self::named(Named::Space, modifiers);
        }
        let (bare, shifted) = if modifiers.shift() {
            (c.to_ascii_lowercase(), c.to_ascii_uppercase())
        } else {
            (c, c)
        };
        Self {
            key: Key::Character(bare.to_string().into()),
            modified_key: Key::Character(shifted.to_string().into()),
            physical: Physical::Unidentified(NativeCode::Unidentified),
            modifiers,
            text: types_text(modifiers).then_some(shifted),
        }
    }

    fn named(named: Named, modifiers: Modifiers) -> Self {
        Self {
            key: Key::Named(named),
            modified_key: Key::Named(named),
            physical: Physical::Unidentified(NativeCode::Unidentified),
            modifiers,
            // Space is the one named key that types something.
            text: (named == Named::Space && types_text(modifiers)).then_some(' '),
        }
    }
}

/// Ctrl, Alt and Super turn a key into a shortcut rather than text.
fn types_text(modifiers: Modifiers) -> bool {
    !(modifiers.control() || modifiers.alt() || modifiers.logo())
}

fn named_key(name: &str) -> Option<Named> {
    KEYS.iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, key)| *key)
}

/// A physical key by its `Code` debug name.
fn physical_code(name: &str) -> Option<Code> {
    macro_rules! codes {
        ($($code:ident),* $(,)?) => {
            [$((stringify!($code), Code::$code)),*]
        };
    }
    let table = codes![
        Backquote,
        Backslash,
        BracketLeft,
        BracketRight,
        Comma,
        Equal,
        Minus,
        Period,
        Quote,
        Semicolon,
        Slash,
        CapsLock,
        Space,
        Tab,
        Enter,
        Escape,
        Backspace,
        Delete,
        ShiftLeft,
        ShiftRight,
        ControlLeft,
        ControlRight,
        AltLeft,
        AltRight,
        SuperLeft,
        SuperRight,
        Insert,
        Home,
        End,
        PageUp,
        PageDown,
        ArrowUp,
        ArrowDown,
        ArrowLeft,
        ArrowRight,
        F1,
        F2,
        F3,
        F4,
        F5,
        F6,
        F7,
        F8,
        F9,
        F10,
        F11,
        F12,
        KeyA,
        KeyB,
        KeyC,
        KeyD,
        KeyE,
        KeyF,
        KeyG,
        KeyH,
        KeyI,
        KeyJ,
        KeyK,
        KeyL,
        KeyM,
        KeyN,
        KeyO,
        KeyP,
        KeyQ,
        KeyR,
        KeyS,
        KeyT,
        KeyU,
        KeyV,
        KeyW,
        KeyX,
        KeyY,
        KeyZ,
        Digit0,
        Digit1,
        Digit2,
        Digit3,
        Digit4,
        Digit5,
        Digit6,
        Digit7,
        Digit8,
        Digit9,
    ];
    table.into_iter().find(|(n, _)| *n == name).map(|(_, c)| c)
}

/// The modifier change and press a keyboard reports when a key goes down.
pub(crate) fn key_down_events(keystroke: &Keystroke) -> [Event; 2] {
    [
        Event::Keyboard(keyboard::Event::ModifiersChanged(keystroke.modifiers)),
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keystroke.key.clone(),
            modified_key: keystroke.modified_key.clone(),
            physical_key: keystroke.physical,
            location: keyboard::Location::Standard,
            modifiers: keystroke.modifiers,
            text: keystroke.text.map(|c| c.to_string().into()),
            repeat: false,
        }),
    ]
}

/// The release and modifier change a keyboard reports when a key goes up.
pub(crate) fn key_up_events(keystroke: &Keystroke) -> [Event; 2] {
    [
        Event::Keyboard(keyboard::Event::KeyReleased {
            key: keystroke.key.clone(),
            modified_key: keystroke.modified_key.clone(),
            physical_key: keystroke.physical,
            location: keyboard::Location::Standard,
            modifiers: keystroke.modifiers,
        }),
        Event::Keyboard(keyboard::Event::ModifiersChanged(Modifiers::empty())),
    ]
}

/// A key press and release, bracketed by the modifier changes a keyboard
/// reports around them.
pub(crate) fn key_events(keystroke: &Keystroke) -> [Event; 4] {
    let [modifiers, press] = key_down_events(keystroke);
    let [release, cleared] = key_up_events(keystroke);
    [modifiers, press, release, cleared]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keystroke_modifiers_in_any_order() {
        let a = Keystroke::parse("ctrl+shift+k").unwrap();
        let b = Keystroke::parse("Shift+CTRL+k").unwrap();
        assert_eq!(a, b);
        assert_eq!(a.modifiers, Modifiers::CTRL | Modifiers::SHIFT);
        assert_eq!(a.key, Key::Character("k".into()));
        assert_eq!(a.modified_key, Key::Character("K".into()));
        assert_eq!(a.text, None, "a shortcut types nothing");
    }

    #[test]
    fn plain_character_types_text() {
        assert_eq!(Keystroke::parse("a").unwrap().text, Some('a'));
        assert_eq!(Keystroke::parse("shift+a").unwrap().text, Some('A'));
    }

    #[test]
    fn named_keys_and_space() {
        let k = Keystroke::parse("shift+enter").unwrap();
        assert_eq!(k.key, Key::Named(Named::Enter));
        assert_eq!(k.text, None);
        let k = Keystroke::parse("ctrl+space").unwrap();
        assert_eq!(k.key, Key::Named(Named::Space));
        assert_eq!(k.text, None);
        assert_eq!(Keystroke::parse("space").unwrap().text, Some(' '));
    }

    #[test]
    fn aliases_resolve_to_their_named_key() {
        for (alias, named) in [
            ("left", Named::ArrowLeft),
            ("right", Named::ArrowRight),
            ("up", Named::ArrowUp),
            ("down", Named::ArrowDown),
            ("esc", Named::Escape),
            ("return", Named::Enter),
            ("del", Named::Delete),
            ("pgup", Named::PageUp),
            ("pgdown", Named::PageDown),
            ("pgdn", Named::PageDown),
            ("insert", Named::Insert),
        ] {
            let k = Keystroke::parse(&format!("shift+{alias}")).unwrap();
            assert_eq!(k.key, Key::Named(named), "{alias}");
            assert_eq!(k.modifiers, Modifiers::SHIFT, "{alias}");
        }
    }

    #[test]
    fn unknown_key_lists_the_known_names() {
        let e = Keystroke::parse("nokey").unwrap_err();
        assert!(
            e.starts_with("unknown key \"nokey\"; keys: enter, escape"),
            "{e}"
        );
        assert!(e.contains("left") && e.contains("code:<Name>"), "{e}");
    }

    #[test]
    fn physical_code_keys() {
        let k = Keystroke::parse("code:Backquote").unwrap();
        assert_eq!(k.physical, Physical::Code(Code::Backquote));
        assert_eq!(k.text, None);
        assert!(Keystroke::parse("code:Nope").is_err());
    }
}
