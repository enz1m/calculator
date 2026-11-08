use crate::engine::{evaluate, format_number};
use enzui_core::theme_file::integration::{ThemeEvent, ThemeIntegration};
use enzui_widgets::title_bar_gesture::WindowCommand;
use enzui_widgets::toast::ToastTone;
use iced::{Subscription, Task};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub(crate) enum Message {
    Theme(Box<ThemeEvent>),
    Launched,
    ToggleMenu,
    CloseMenu,
    OpenShortcuts,
    CloseShortcuts,
    Dialog(enzui_widgets::modal_host::DialogEvent<DialogKey>),
    Window(WindowCommand),
    Close,
    Mode(usize),
    Input(String),
    Insert(String),
    Evaluate,
    Clear,
    Backspace,
    Sign,
    Degrees(usize),
    Recall(usize),
    ClearHistory,
    Copy,
    CopyHistoryResult(String),
    Paste,
    Pasted(Option<String>),
    ClipboardStatus(Result<(), String>),
    Category(usize),
    From(usize),
    To(usize),
    Swap,
    MemoryStore,
    MemoryRecall,
    MemoryClear,
    Event(iced::Event),
    Keyboard(iced::keyboard::Event, bool),
    Toast(enzui_widgets::toast_host::ToastEvent),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Entry {
    pub expression: String,
    pub result: String,
    #[serde(default = "default_degrees")]
    pub(crate) degrees: bool,
    #[serde(default)]
    pub(crate) answer: f64,
}
#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Saved {
    pub(crate) history: Vec<Entry>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum DialogKey {
    Shortcuts,
}

pub(crate) struct Calculator {
    pub desktop: enzui_widgets::single_instance::SingleInstance,
    pub menu_open: bool,
    pub window_size: iced::Size,
    pub dialogs: enzui_widgets::modal_host::DialogStack<DialogKey>,
    pub themes: ThemeIntegration,
    pub mode: usize,
    pub expression: String,
    pub result: String,
    pub error: Option<String>,
    pub degrees: bool,
    pub answer: f64,
    pub memory: Option<f64>,
    pub history: Vec<Entry>,
    pub category: usize,
    pub from: usize,
    pub to: usize,
    pub completed: bool,
    pub toasts: enzui_widgets::toast_host::ToastHost,
    toast_expiry: enzui_widgets::toast_host::ToastExpirySchedule,
    storage: Option<std::path::PathBuf>,
    _storage_lock: Option<enzui_core::atomic_state::StateLock>,
}
impl Calculator {
    pub fn new() -> (Self, Task<Message>) {
        Self::with_storage(history_path())
    }
    fn with_storage(mut storage: Option<std::path::PathBuf>) -> (Self, Task<Message>) {
        let (mut themes, task) = ThemeIntegration::boot();
        if std::env::args().any(|a| a == "--dark") {
            themes.set_mode_override(Some(enzui_theme::EnzThemeMode::Dark));
        }
        if std::env::args().any(|a| a == "--light") {
            themes.set_mode_override(Some(enzui_theme::EnzThemeMode::Light));
        }
        let storage_lock = storage.as_deref().and_then(|path| {
            enzui_core::atomic_state::StateLock::try_acquire(path)
                .ok()
                .flatten()
        });
        let lock_failed = storage.is_some() && storage_lock.is_none();
        let (saved, notice) = match storage.as_deref().map(|path| {
            if lock_failed {
                Err(std::io::Error::other("history lock unavailable"))
            } else {
                crate::storage::load(path)
            }
        }) {
            Some(Ok(saved)) => (saved, None),
            Some(Err(_)) => {
                storage = None;
                (
                    Saved::default(),
                    Some(crate::i18n::tr(
                        "History could not be loaded; the existing file is preserved",
                    )),
                )
            }
            None => (Saved::default(), None),
        };
        let mut app = (
            Self {
                desktop: enzui_widgets::single_instance::SingleInstance::standalone(),
                menu_open: false,
                window_size: iced::Size::new(420.0, 620.0),
                dialogs: enzui_widgets::modal_host::DialogStack::new(),
                themes,
                mode: 0,
                expression: String::new(),
                result: "0".into(),
                error: None,
                degrees: true,
                answer: 0.0,
                memory: None,
                history: saved.history.into_iter().take(100).collect(),
                category: 0,
                from: 0,
                to: 1,
                completed: false,
                toasts: enzui_widgets::toast_host::ToastHost::new().id_prefix("calculator-toast"),
                toast_expiry: Default::default(),
                storage,
                _storage_lock: storage_lock,
            },
            task.map(|event| Message::Theme(Box::new(event))),
        );
        if let Some(notice) = notice {
            app.0.notify(notice, ToastTone::Error);
        }
        app
    }
    fn notify(&mut self, text: String, tone: ToastTone) {
        let now = std::time::Instant::now();
        self.toasts.push_at(
            enzui_widgets::toast_host::ToastSpec::new(text.clone())
                .tone(tone)
                .coalesce_key(text),
            now,
        );
        self.toasts.sync_expiry(&mut self.toast_expiry, now);
    }
    pub fn theme(&self) -> iced::Theme {
        self.themes.iced_theme()
    }
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            self.themes
                .subscription()
                .map(|event| Message::Theme(Box::new(event))),
            iced::event::listen_with(events),
            self.desktop.subscription(|_| Message::Launched),
            self.toast_expiry
                .armed()
                .map_or_else(Subscription::none, |(deadline, wait)| {
                    iced::time::every(wait).with(deadline).map(|(_, now)| {
                        Message::Toast(enzui_widgets::toast_host::ToastEvent::Tick(now))
                    })
                }),
        ])
    }
    pub fn preview(&self) -> Option<String> {
        if self.expression.is_empty() || self.completed {
            return None;
        }
        evaluate(&self.expression, self.degrees, self.answer)
            .ok()
            .map(format_number)
    }
    pub fn conversion(&self) -> Result<f64, crate::engine::CalcError> {
        let value = evaluate(
            if self.expression.is_empty() {
                "0"
            } else {
                &self.expression
            },
            self.degrees,
            self.answer,
        )?;
        crate::units::convert(value, self.category, self.from, self.to)
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        let refocus = matches!(
            &message,
            Message::Insert(_)
                | Message::Clear
                | Message::Backspace
                | Message::Sign
                | Message::Recall(_)
                | Message::MemoryRecall
                | Message::Pasted(Some(_))
        );
        match message {
            Message::Theme(event) => {
                return self
                    .themes
                    .update(*event)
                    .map(|event| Message::Theme(Box::new(event)));
            }
            Message::Launched => return self.desktop.drain(None).1,
            Message::ToggleMenu => self.menu_open = !self.menu_open,
            Message::CloseMenu => self.menu_open = false,
            Message::OpenShortcuts => {
                self.menu_open = false;
                return self.dialogs.open(
                    DialogKey::Shortcuts,
                    Some(iced::advanced::widget::Id::new("calculator-overflow")),
                );
            }
            Message::CloseShortcuts => {
                return self
                    .dialogs
                    .dismiss(
                        &DialogKey::Shortcuts,
                        enzui_core::modal::DismissCause::CloseButton,
                    )
                    .1;
            }
            Message::Dialog(event) => return self.dialogs.update(event).1,
            Message::Window(WindowCommand::Close) | Message::Close => {
                return iced::exit();
            }
            Message::Window(command) => {
                return iced::window::latest()
                    .then(move |id| id.map_or_else(Task::none, |id| command.into_task(id)));
            }
            Message::Mode(mode) => {
                if mode == 2 && self.mode != 2 && self.completed {
                    self.expression = self.answer.to_string();
                    self.completed = false;
                }
                self.mode = mode.min(2);
                self.error = None;
            }
            Message::Input(value) => {
                if value.len() <= 1024 {
                    if self.completed {
                        if let Some(suffix) = value.strip_prefix(&self.result) {
                            let suffix = suffix.to_owned();
                            self.insert(&suffix);
                        } else {
                            self.expression = value;
                        }
                    } else {
                        self.expression = value;
                    }
                    self.completed = false;
                    self.error = None;
                }
            }
            Message::Insert(value) => self.insert(&value),
            Message::Evaluate => self.calculate(),
            Message::Clear => self.clear(),
            Message::Backspace => {
                if self.completed {
                    self.expression.clone_from(&self.result);
                }
                self.expression.pop();
                self.completed = false;
                self.error = None;
            }
            Message::Sign => {
                if self.completed {
                    self.expression = self.answer.to_string();
                }
                if self.expression.is_empty() {
                    self.expression = "-".into();
                } else if self.expression.len() <= 1021 {
                    self.expression = format!("-({})", self.expression);
                }
                self.completed = false;
            }
            Message::Degrees(index) => self.degrees = index == 0,
            Message::Recall(index) => {
                if let Some(entry) = self.history.get(index) {
                    self.expression = entry.expression.clone();
                    self.degrees = entry.degrees;
                    self.answer = entry.answer;
                    self.completed = false;
                    self.error = None;
                }
            }
            Message::ClearHistory => {
                self.history.clear();
                self.save();
            }
            Message::Copy => {
                let value = if self.mode == 2 {
                    self.conversion().map(format_number).unwrap_or_default()
                } else {
                    self.preview().unwrap_or_else(|| self.result.clone())
                };
                return copy_result(value);
            }
            Message::CopyHistoryResult(value) => return copy_result(value),
            Message::Paste => {
                let too_long = crate::i18n::tr("Pasted expression is too long");
                let unreadable = crate::i18n::tr("Could not read text from the clipboard");
                return iced::clipboard::read_text().map(move |result| match result {
                    Ok(text) if text.len() <= 1024 => Message::Pasted(Some((*text).clone())),
                    Ok(_) => Message::ClipboardStatus(Err(too_long.clone())),
                    Err(_) => Message::ClipboardStatus(Err(unreadable.clone())),
                });
            }
            Message::Pasted(Some(value)) => {
                if value.len() <= 1024 {
                    self.expression = value.trim().into();
                    self.completed = false;
                    self.error = None;
                } else {
                    self.error = Some(crate::i18n::tr("Pasted expression is too long"));
                }
            }
            Message::Pasted(None) => self.notify(
                crate::i18n::tr("No text found on the clipboard"),
                ToastTone::Info,
            ),
            Message::ClipboardStatus(result) => match result {
                Ok(()) => self.notify(crate::i18n::tr("Result copied"), ToastTone::Success),
                Err(error) => self.notify(error, ToastTone::Error),
            },
            Message::Toast(event) => {
                let now = std::time::Instant::now();
                self.toasts.update(event);
                self.toasts.tick_at(now);
                self.toasts.sync_expiry(&mut self.toast_expiry, now);
            }
            Message::Category(category) => {
                self.category = category.min(crate::units::CATEGORIES.len() - 1);
                self.from = 0;
                self.to = 1;
            }
            Message::From(index) => {
                self.from = index.min(crate::units::units(self.category).len() - 1)
            }
            Message::To(index) => self.to = index.min(crate::units::units(self.category).len() - 1),
            Message::Swap => std::mem::swap(&mut self.from, &mut self.to),
            Message::MemoryStore if self.completed => self.memory = Some(self.answer),
            Message::MemoryStore => match evaluate(
                if self.expression.is_empty() {
                    &self.result
                } else {
                    &self.expression
                },
                self.degrees,
                self.answer,
            ) {
                Ok(value) => self.memory = Some(value),
                Err(error) => self.error = Some(error.to_string()),
            },
            Message::MemoryRecall => {
                if let Some(value) = self.memory {
                    self.insert(&value.to_string());
                }
            }
            Message::MemoryClear => self.memory = None,
            Message::Event(iced::Event::Window(iced::window::Event::Resized(size))) => {
                self.window_size = size
            }
            Message::Event(iced::Event::Window(iced::window::Event::CloseRequested)) => {
                return iced::exit();
            }
            Message::Event(iced::Event::Keyboard(_)) if !self.dialogs.is_empty() => {}
            // Read-only inputs may ignore keys that global calculator bindings mutate.
            Message::Event(iced::Event::Keyboard(event)) => {
                let ids = if self.mode == 2 {
                    vec![iced::advanced::widget::Id::new("conversion-result")]
                } else {
                    (0..self.history.len())
                        .map(|index| {
                            iced::advanced::widget::Id::from(format!("history-result-{index}"))
                        })
                        .collect()
                };
                return iced::advanced::widget::operate(ReadOnlyFocus {
                    ids,
                    focused: false,
                })
                .map(move |read_only| Message::Keyboard(event.clone(), read_only));
            }
            Message::Keyboard(_, _) if !self.dialogs.is_empty() => {}
            Message::Keyboard(event, read_only) => {
                if read_only {
                    if let Some(
                        command @ (Message::Copy | Message::Mode(_) | Message::OpenShortcuts),
                    ) = crate::shortcuts::dispatch(&event)
                    {
                        return self.update(command);
                    }
                    return Task::none();
                }
                if let Some(command) = crate::shortcuts::dispatch(&event) {
                    return self.update(command);
                }
                let iced::keyboard::Event::KeyPressed {
                    modified_key: key,
                    modifiers,
                    ..
                } = event
                else {
                    return Task::none();
                };
                use iced::keyboard::Key;
                if !modifiers.control() && !modifiers.alt() && !modifiers.logo() {
                    match key.as_ref() {
                        Key::Character(value)
                            if value.chars().all(|c| {
                                c.is_ascii_alphanumeric() || "+-*/().^%!π".contains(c)
                            }) =>
                        {
                            self.insert(value);
                            return focus_expression();
                        }
                        _ => {}
                    }
                }
            }
            Message::Event(_) => {}
        }
        if refocus {
            focus_expression()
        } else {
            Task::none()
        }
    }
    fn insert(&mut self, text: &str) {
        if self.completed {
            self.expression =
                if text.starts_with(['+', '-', '*', '/', '−', '×', '÷', '^', '%', '!']) {
                    self.answer.to_string()
                } else {
                    String::new()
                };
            self.completed = false;
        }
        if self.expression.len() + text.len() <= 1024 {
            self.expression.push_str(text);
            self.error = None;
        }
    }
    fn clear(&mut self) {
        self.expression.clear();
        self.result = "0".into();
        self.error = None;
        self.completed = false;
    }
    fn calculate(&mut self) {
        if self.expression.is_empty() {
            return;
        }
        match evaluate(&self.expression, self.degrees, self.answer) {
            Ok(value) => {
                let previous_answer = self.answer;
                self.answer = value;
                self.result = format_number(value);
                self.error = None;
                self.completed = true;
                self.history.insert(
                    0,
                    Entry {
                        expression: self.expression.clone(),
                        result: self.result.clone(),
                        degrees: self.degrees,
                        answer: previous_answer,
                    },
                );
                self.history.truncate(100);
                self.save();
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    fn save(&mut self) {
        let Some(path) = self.storage.clone() else {
            self.notify(
                crate::i18n::tr("History storage is unavailable"),
                ToastTone::Error,
            );
            return;
        };
        let saved = Saved {
            history: self.history.clone(),
        };
        let result = crate::storage::save(&path, &saved);
        if result.is_err() {
            self.notify(
                crate::i18n::tr("Could not save calculation history"),
                ToastTone::Error,
            );
        }
    }
}
fn copy_result(value: String) -> Task<Message> {
    let failure = crate::i18n::tr("Could not copy the result");
    iced::clipboard::write(value)
        .map(move |result| Message::ClipboardStatus(result.map_err(|_| failure.clone())))
}

struct ReadOnlyFocus {
    ids: Vec<iced::advanced::widget::Id>,
    focused: bool,
}
impl iced::advanced::widget::Operation<bool> for ReadOnlyFocus {
    fn traverse(
        &mut self,
        operate: &mut dyn FnMut(&mut dyn iced::advanced::widget::Operation<bool>),
    ) {
        operate(self);
    }
    fn focusable(
        &mut self,
        id: Option<&iced::advanced::widget::Id>,
        _: iced::Rectangle,
        state: &mut dyn iced::advanced::widget::operation::Focusable,
    ) {
        self.focused |= state.is_focused() && id.is_some_and(|id| self.ids.contains(id));
    }
    fn finish(&self) -> iced::advanced::widget::operation::Outcome<bool> {
        iced::advanced::widget::operation::Outcome::Some(self.focused)
    }
}

fn focus_expression() -> Task<Message> {
    iced::advanced::widget::operate(enzui_core::focus::focus_and_reveal_id(
        iced::advanced::widget::Id::new("calculator-expression"),
    ))
}

fn default_degrees() -> bool {
    true
}

fn history_path() -> Option<std::path::PathBuf> {
    dirs::data_local_dir().map(|p| p.join("enz-calculator/history.json"))
}

fn events(event: iced::Event, status: iced::event::Status, _: iced::window::Id) -> Option<Message> {
    let shortcut = matches!(&event, iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {modifiers,..}) if modifiers.control());
    let copy_captured = matches!(&event, iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { modified_key: iced::keyboard::Key::Character(key), modifiers, .. }) if modifiers.control() && !modifiers.shift() && key.as_str() == "c")
        && status == iced::event::Status::Captured;
    if (shortcut && !copy_captured) || status == iced::event::Status::Ignored {
        Some(Message::Event(event))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Calculator {
        Calculator::with_storage(None).0
    }
    fn send(app: &mut Calculator, message: Message) {
        let _ = app.update(message);
    }
    #[test]
    fn calculator_keypad_chains_and_recovers_from_errors() {
        let mut app = fixture();
        for key in ["2", "+", "3", "×", "4"] {
            send(&mut app, Message::Insert(key.into()));
        }
        assert_eq!(app.preview().as_deref(), Some("14"));
        send(&mut app, Message::Evaluate);
        assert_eq!(app.result, "14");
        send(&mut app, Message::Insert("+".into()));
        send(&mut app, Message::Insert("6".into()));
        send(&mut app, Message::Evaluate);
        assert_eq!(app.result, "20");
        assert_eq!(app.history.len(), 2);
        send(&mut app, Message::Input("20+22".into()));
        send(&mut app, Message::Evaluate);
        assert_eq!(app.result, "42");
        send(&mut app, Message::Backspace);
        assert_eq!(app.expression, "4");
        send(&mut app, Message::Input(String::new()));
        assert!(app.expression.is_empty());
        assert!(!app.completed);
        send(&mut app, Message::Recall(2));
        assert_eq!(app.expression, "2+3×4");
        send(&mut app, Message::Input("1/0".into()));
        send(&mut app, Message::Evaluate);
        assert!(app.error.is_some());
        send(&mut app, Message::Clear);
        send(&mut app, Message::Insert("7".into()));
        send(&mut app, Message::Evaluate);
        assert_eq!(app.result, "7");
        assert!(app.error.is_none());
        send(&mut app, Message::Insert("8".into()));
        assert_eq!(app.expression, "8");
    }
    #[test]
    fn calculator_memory_modes_and_units_use_app_routing() {
        let mut app = fixture();
        send(&mut app, Message::Input("6*7".into()));
        send(&mut app, Message::MemoryStore);
        send(&mut app, Message::Mode(1));
        assert_eq!(app.expression, "6*7");
        send(&mut app, Message::Clear);
        send(&mut app, Message::MemoryRecall);
        assert_eq!(app.expression, "42");
        send(&mut app, Message::MemoryClear);
        assert_eq!(app.memory, None);
        send(&mut app, Message::Mode(2));
        send(&mut app, Message::Category(2));
        send(&mut app, Message::Input("100".into()));
        assert!((app.conversion().expect("boiling") - 212.0).abs() < 1e-9);
        send(&mut app, Message::Swap);
        assert_eq!((app.from, app.to), (1, 0));
        send(&mut app, Message::Pasted(Some("sqrt(81)".into())));
        send(&mut app, Message::Mode(1));
        send(&mut app, Message::Evaluate);
        assert_eq!(app.result, "9");
        send(&mut app, Message::Input("ans+2".into()));
        send(&mut app, Message::Evaluate);
        send(&mut app, Message::Mode(2));
        send(&mut app, Message::Category(0));
        assert_eq!(app.expression, "11");
        assert!((app.conversion().expect("completed answer") - 0.011).abs() < 1e-12);
    }
    #[test]
    fn calculator_history_is_persisted_bounded_and_clearable() {
        let folder = tempfile::tempdir().expect("test storage");
        let mut app = fixture();
        let path = folder.path().join("history.json");
        app.storage = Some(path.clone());
        for number in 0..105 {
            send(&mut app, Message::Input(number.to_string()));
            send(&mut app, Message::Evaluate);
        }
        let saved: Saved = serde_json::from_slice(&std::fs::read(&path).expect("history file"))
            .expect("valid history");
        assert_eq!(saved.history.len(), 100);
        assert_eq!(saved.history[0].result, "104");
        assert_eq!(saved.history[99].result, "5");
        send(&mut app, Message::ClearHistory);
        let saved: Saved = serde_json::from_slice(&std::fs::read(path).expect("cleared file"))
            .expect("valid history");
        assert!(saved.history.is_empty());
        assert!(app.history.is_empty());
    }
    #[test]
    fn calculator_history_restores_angle_and_previous_answer_context() {
        let mut app = fixture();
        send(&mut app, Message::Input("sin(30)".into()));
        send(&mut app, Message::Evaluate);
        send(&mut app, Message::Input("ans+2".into()));
        send(&mut app, Message::Evaluate);
        assert_eq!(app.result, "2.5");
        send(&mut app, Message::MemoryStore);
        assert_eq!(app.memory, Some(2.5));
        send(&mut app, Message::Degrees(1));
        send(&mut app, Message::Input("99".into()));
        send(&mut app, Message::Evaluate);
        send(&mut app, Message::Recall(1));
        assert!(app.degrees);
        send(&mut app, Message::Evaluate);
        assert_eq!(app.result, "2.5");
    }
    fn key_press(
        key: iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> iced::keyboard::Event {
        iced::keyboard::Event::KeyPressed {
            modified_key: key.clone(),
            key,
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: iced::keyboard::Location::Standard,
            modifiers,
            text: None,
            repeat: false,
        }
    }

    #[test]
    fn calculator_keyboard_preserves_readonly_and_modal_input() {
        let mut app = fixture();
        send(&mut app, Message::Input("2+3*4".into()));
        for (key, modifiers) in [
            (
                iced::keyboard::Key::Character("9".into()),
                iced::keyboard::Modifiers::empty(),
            ),
            (
                iced::keyboard::Key::Named(iced::keyboard::key::Named::Backspace),
                iced::keyboard::Modifiers::empty(),
            ),
            (
                iced::keyboard::Key::Character("v".into()),
                iced::keyboard::Modifiers::CTRL,
            ),
        ] {
            assert_eq!(
                app.update(Message::Keyboard(key_press(key, modifiers), true))
                    .units(),
                0
            );
            assert_eq!(app.expression, "2+3*4");
        }
        send(&mut app, Message::OpenShortcuts);
        send(
            &mut app,
            Message::Event(iced::Event::Keyboard(key_press(
                iced::keyboard::Key::Character("9".into()),
                iced::keyboard::Modifiers::empty(),
            ))),
        );
        assert_eq!(app.expression, "2+3*4");
        send(&mut app, Message::CloseShortcuts);
        assert!(app.dialogs.is_empty());
    }
    #[test]
    fn calculator_history_preserves_refused_files_and_excludes_other_writers() {
        let folder = tempfile::tempdir().expect("test storage");
        let path = folder.path().join("history.json");
        let mut oversized = br#"{"history":[]}"#.to_vec();
        oversized.resize(262_145, b' ');
        for original in [b"{broken".to_vec(), oversized] {
            std::fs::write(&path, &original).expect("invalid history fixture");
            let (mut app, _) = Calculator::with_storage(Some(path.clone()));
            assert!(!app.toasts.is_empty());
            send(&mut app, Message::Input("6*7".into()));
            send(&mut app, Message::Evaluate);
            send(&mut app, Message::ClearHistory);
            assert_eq!(std::fs::read(&path).expect("preserved history"), original);
        }
        std::fs::remove_file(&path).expect("remove invalid fixture");
        let (mut owner, _) = Calculator::with_storage(Some(path.clone()));
        send(&mut owner, Message::Input("7*8".into()));
        send(&mut owner, Message::Evaluate);
        let (mut second, _) = Calculator::with_storage(Some(path.clone()));
        assert!(!second.toasts.is_empty());
        send(&mut second, Message::ClearHistory);
        assert_eq!(
            crate::storage::load(&path).expect("owner history").history[0].result,
            "56"
        );
        drop(second);
        drop(owner);
        let (restored, _) = Calculator::with_storage(Some(path));
        assert_eq!(restored.history[0].expression, "7*8");
    }

    #[cfg(unix)]
    #[test]
    fn calculator_history_does_not_follow_symlinks() {
        let folder = tempfile::tempdir().expect("test storage");
        let original = folder.path().join("original.json");
        let path = folder.path().join("history.json");
        let bytes = br#"{"history":[]}"#;
        std::fs::write(&original, bytes).expect("original history");
        std::os::unix::fs::symlink(&original, &path).expect("history symlink");
        let (mut app, _) = Calculator::with_storage(Some(path.clone()));
        assert!(!app.toasts.is_empty());
        send(&mut app, Message::Input("99".into()));
        send(&mut app, Message::Evaluate);
        assert_eq!(std::fs::read(original).expect("preserved target"), bytes);
        assert!(
            std::fs::symlink_metadata(path)
                .expect("preserved link")
                .is_symlink()
        );
    }
}
