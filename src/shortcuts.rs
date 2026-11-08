use crate::app::Message;
use crate::i18n::tr;
use enzui_core::action::{ShortcutBinding, dispatch_shortcut_command};
use enzui_core::shortcut::{Modifier, NamedKey, Shortcut};

pub(crate) fn bindings() -> Vec<ShortcutBinding<Message>> {
    let rows = [
        (
            "calculator.basic",
            "Basic",
            Message::Mode(0),
            Shortcut::new(Modifier::CTRL, '1'),
        ),
        (
            "calculator.scientific",
            "Scientific",
            Message::Mode(1),
            Shortcut::new(Modifier::CTRL, '2'),
        ),
        (
            "calculator.convert",
            "Convert",
            Message::Mode(2),
            Shortcut::new(Modifier::CTRL, '3'),
        ),
        (
            "calculator.copy",
            "Copy result",
            Message::Copy,
            Shortcut::new(Modifier::CTRL | Modifier::SHIFT, 'c'),
        ),
        (
            "calculator.paste",
            "Paste expression",
            Message::Paste,
            Shortcut::new(Modifier::CTRL, 'v'),
        ),
        (
            "calculator.evaluate",
            "Calculate · Enter",
            Message::Evaluate,
            Shortcut::new(Modifier::NONE, NamedKey::Enter),
        ),
        (
            "calculator.clear",
            "Clear",
            Message::Clear,
            Shortcut::new(Modifier::NONE, NamedKey::Escape),
        ),
        (
            "calculator.backspace",
            "Backspace",
            Message::Backspace,
            Shortcut::new(Modifier::NONE, NamedKey::Backspace),
        ),
        (
            "calculator.help",
            "Keyboard Shortcuts",
            Message::OpenShortcuts,
            Shortcut::new(Modifier::NONE, NamedKey::F1),
        ),
    ];
    rows.into_iter()
        .map(|(id, label, command, shortcut)| {
            let binding = ShortcutBinding::new(id, command, shortcut).with_description(tr(label));
            match id {
                "calculator.copy" => binding.with_alternate(Shortcut::new(Modifier::CTRL, 'c')),
                "calculator.evaluate" => binding.with_alternate(Shortcut::new(Modifier::NONE, '=')),
                _ => binding,
            }
        })
        .collect()
}

pub(crate) fn dispatch(event: &iced::keyboard::Event) -> Option<Message> {
    dispatch_shortcut_command(&bindings(), event, &())
}
