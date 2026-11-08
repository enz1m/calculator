use crate::app::{Calculator, Message};
use crate::i18n::tr;
use enzui_widgets::{
    app_shell::{ShellLayout, ShellTokens, enz_app_shell},
    button::{ButtonKind, ButtonSize, enz_button},
    combo_box::enz_combo_box,
    list_row_surface::{ListRowSurfaceTokens, TrailingVisibility, enz_list_row_surface_lazy},
    scroll_view::enz_scroll_view,
    surface::{SurfaceTone, enz_surface},
    text::{TextTone, enz_text},
    text_field::{EnzTextField, TextFieldSize, enz_text_field_owned},
    title_bar::enz_title_bar,
    tokens::FieldVariant,
    toolbar_icon::enz_toolbar_icon,
    view_switcher::{SwitcherPage, enz_view_switcher},
};
use iced::widget::{Space, column, container, row};
use iced::{Element, Length};

pub(crate) fn root(app: &Calculator) -> Element<'_, Message> {
    let shell = enz_app_shell(move || body(app))
        .layout(ShellLayout::TitlebarContent)
        .tokens(
            ShellTokens::default()
                .gap(0.0)
                .subtitlebar_height(enzui_theme::text_scale::ui_px(40.0)),
        )
        .titlebar(move || {
            use enzui_widgets::action_bar::ActionItem;
            enz_title_bar()
                .title(tr("Calculator"))
                .actions(
                    vec![
                        ActionItem::icon(
                            "calculator.copy",
                            "copy",
                            tr("Copy result"),
                            Message::Copy,
                        ),
                        ActionItem::icon(
                            "calculator.paste",
                            "clipboard-paste",
                            tr("Paste expression"),
                            Message::Paste,
                        ),
                        ActionItem::icon(
                            "calculator.clear-history",
                            "trash-2",
                            tr("Clear history"),
                            Message::ClearHistory,
                        )
                        .destructive(true),
                        ActionItem::icon(
                            "calculator.shortcuts",
                            "keyboard",
                            tr("Keyboard Shortcuts"),
                            Message::OpenShortcuts,
                        ),
                    ],
                    Message::ToggleMenu,
                )
                .action_inline_limit(1)
                .overflow_open(app.menu_open)
                .action_dismiss(Message::CloseMenu)
                .action_menu_id_prefix("calculator-actions")
                .action_overflow_trigger_id(iced::advanced::widget::Id::new("calculator-overflow"))
                .id("calculator-titlebar")
                .into_element()
        })
        .subtitlebar(move || {
            let switcher = enz_view_switcher(
                vec![
                    SwitcherPage::new("basic", tr("Basic")).with_icon("calculator"),
                    SwitcherPage::new("scientific", tr("Scientific")).with_icon("flask-conical"),
                    SwitcherPage::new("convert", tr("Convert")).with_icon("arrow-right-left"),
                ],
                app.mode,
                Message::Mode,
            )
            .tab_id_prefix("calculator-modes")
            .size(TextFieldSize::Sm)
            .stretch(true)
            .aria_label(tr("Calculator mode"));
            container(
                container(switcher).width(Length::Fill.max(enzui_theme::text_scale::ui_px(560.0))),
            )
            .padding([0.0, app.themes.theme().spacing.sm()])
            .center_x(Length::Fill)
            .into()
        })
        .on_close(Message::Close)
        .on_window_gesture(Message::Window)
        .into_element();
    let presentation = app.dialogs.view(
        shell,
        |_| {
            enzui_widgets::shortcuts_dialog::enz_shortcuts_dialog::<Message>(
                true,
                enzui_widgets::shortcut_list::shortcut_catalog_entries(
                    &crate::shortcuts::bindings(),
                ),
            )
            .title(tr("Keyboard Shortcuts"))
            .surface_radius(enzui_theme::text_defaults::shape_radius(|shape| {
                shape.dialog
            }))
            .viewport_height(app.window_size.height)
            .into_surface(Some(Box::new(|| Message::CloseShortcuts)))
        },
        Message::Dialog,
    );
    let toasts = (!app.toasts.is_empty()).then(|| app.toasts.view().map(Message::Toast));
    enzui_widgets::toast_host::enz_toast_layer(
        presentation,
        toasts,
        enzui_widgets::toast_host::ToastOverlayPlacement::default(),
    )
}

fn body(app: &Calculator) -> Element<'_, Message> {
    let spacing = app.themes.theme().spacing;
    let column_cap = enzui_theme::text_scale::ui_px(560.0);
    if app.mode == 2 {
        return container(
            enz_scroll_view(
                container(container(converter(app)).width(Length::Fill.max(column_cap)))
                    .center_x(Length::Fill),
            )
            .height(Length::Fill)
            .into_element(),
        )
        .padding(spacing.space5)
        .height(Length::Fill)
        .into();
    }
    let keypad = container(
        column![controls(app), keypad(app)]
            .spacing(spacing.space3)
            .width(Length::Fill),
    )
    .width(Length::Fill.max(enzui_theme::text_scale::ui_px(404.0)));
    let keypad = container(keypad).center_x(Length::Fill);
    container(
        container(column![sheet(app), keypad].spacing(spacing.space3))
            .width(Length::Fill.max(column_cap))
            .height(Length::Fill),
    )
    .center_x(Length::Fill)
    .padding(spacing.sm())
    .height(Length::Fill)
    .into()
}

fn sheet(app: &Calculator) -> Element<'_, Message> {
    let spacing = app.themes.theme().spacing;
    let mut entries = column![].spacing(spacing.xs()).width(Length::Fill);
    for (index, entry) in app.history.iter().enumerate().rev() {
        entries = entries.push(
            enz_list_row_surface_lazy(
                column![
                    enz_text(&entry.expression)
                        .size_px(12.0)
                        .tone(TextTone::Muted)
                        .single_line_elided(),
                    enz_text_field_owned("", "", entry.result.clone(), Message::Input)
                        .id(iced::advanced::widget::Id::from(format!(
                            "history-result-{index}"
                        )))
                        .aria_label(format!("{} {}", tr("Result"), entry.result))
                        .size(TextFieldSize::Sm)
                        .variant(FieldVariant::Outlined)
                        .border_variant(FieldVariant::Inline)
                        .read_only(true)
                        .quiet(true),
                ]
                .spacing(0)
                .width(Length::Fill),
            )
            .id(iced::advanced::widget::Id::from(format!(
                "calculator-history-{index}"
            )))
            .content_height(true)
            .content_padding(0.0)
            .outer_gutter(0.0)
            .tokens_resolved(|theme| {
                let mut tokens = ListRowSurfaceTokens::from_palette(
                    &enzui_widgets::theme_lookup::palette(theme),
                );
                tokens.hover_overlay = iced::Color::TRANSPARENT;
                tokens.pressed_overlay = iced::Color::TRANSPARENT;
                tokens
            })
            .trailing(
                row![
                    enz_toolbar_icon("copy", Message::CopyHistoryResult(entry.result.clone()))
                        .id(iced::advanced::widget::Id::from(format!(
                            "calculator-history-copy-{index}"
                        )))
                        .size(26.0, 14.0)
                        .aria_label(format!("{} {}", tr("Copy result"), entry.result))
                        .tooltip(tr("Copy result")),
                    enz_toolbar_icon("corner-up-left", Message::Recall(index))
                        .id(iced::advanced::widget::Id::from(format!(
                            "calculator-recall-{index}"
                        )))
                        .size(26.0, 14.0)
                        .aria_label(format!("{} {}", tr("Use"), entry.expression))
                        .tooltip(tr("Use")),
                ]
                .align_y(iced::Alignment::Center),
            )
            .trailing_visibility(TrailingVisibility::OnHoverOrFocus),
        );
    }
    let value = if app.completed {
        &app.result
    } else {
        &app.expression
    };
    let input = calculation_input(
        value,
        "0",
        24.0,
        "calculator-expression".into(),
        "Calculation and result",
        false,
    )
    .variant(FieldVariant::Outlined)
    .on_submit_with(|| Message::Evaluate);
    let input = enz_surface(input)
        .tone(SurfaceTone::Elevated)
        .radius(enzui_theme::text_defaults::shape_radius(|shape| {
            shape.field
        }))
        .width(Length::Fill);
    let status: Element<'_, Message> = if let Some(error) = app.error.as_deref() {
        enz_text(error)
            .size_px(12.0)
            .single_line_elided()
            .tone(TextTone::Danger)
            .into()
    } else if let Some(preview) = app.preview() {
        enz_text(format!("= {preview}"))
            .size_px(12.0)
            .single_line_elided()
            .tone(TextTone::Muted)
            .into()
    } else {
        Space::new().into()
    };
    let sheet = column![
        enz_scroll_view(entries)
            .id(iced::widget::Id::new("calculation-history"))
            .initial_offset(iced::widget::scrollable::AbsoluteOffset {
                x: 0.0,
                y: f32::MAX
            })
            .height(Length::Fill)
            .into_element(),
        input,
        container(status)
            .width(Length::Fill)
            .height(enzui_theme::text_scale::ui_px(20.0)),
    ]
    .spacing(spacing.xs())
    .height(Length::Fill);
    enz_surface(sheet)
        .tone(SurfaceTone::SurfaceAlt)
        .padding(spacing.sm())
        .height(Length::Fill)
        .width(Length::Fill)
        .into_element()
}

fn controls(app: &Calculator) -> Element<'_, Message> {
    let spacing = app.themes.theme().spacing;
    let mut controls = row![
        enz_button("MS", Message::MemoryStore)
            .kind(ButtonKind::Quiet)
            .size(ButtonSize::Sm)
            .aria_label(tr("Store result in memory"))
            .id(iced::advanced::widget::Id::new("calculator-memory-store")),
        enz_button("MR", Message::MemoryRecall)
            .kind(ButtonKind::Quiet)
            .size(ButtonSize::Sm)
            .enabled(app.memory.is_some())
            .aria_label(tr("Recall memory"))
            .id(iced::advanced::widget::Id::new("calculator-memory-recall")),
        enz_button("MC", Message::MemoryClear)
            .kind(ButtonKind::Quiet)
            .size(ButtonSize::Sm)
            .enabled(app.memory.is_some())
            .aria_label(tr("Clear memory"))
            .id(iced::advanced::widget::Id::new("calculator-memory-clear")),
    ]
    .spacing(spacing.space1)
    .align_y(iced::Alignment::Center);
    if app.mode == 1 {
        controls = controls.push(
            enz_button(
                if app.degrees { "DEG" } else { "RAD" },
                Message::Degrees(usize::from(app.degrees)),
            )
            .kind(ButtonKind::Quiet)
            .size(ButtonSize::Sm)
            .aria_label(tr("Toggle degrees and radians"))
            .id(iced::advanced::widget::Id::new("calculator-angle")),
        );
    }
    controls.into()
}

fn keypad(app: &Calculator) -> Element<'_, Message> {
    let spacing = app.themes.theme().spacing;
    let scientific = app.mode == 1;
    // Short windows reserve enough height for the display above the keypad.
    let height = app.window_size.height;
    let key_size = if height > 0.0 && height < enzui_theme::text_scale::ui_px(480.0) {
        ButtonSize::Sm
    } else if scientific || (height > 0.0 && height < enzui_theme::text_scale::ui_px(600.0)) {
        ButtonSize::Md
    } else {
        ButtonSize::Lg
    };
    let lines: Vec<Vec<(&str, &str)>> = if scientific {
        vec![
            vec![
                ("sin", "sin("),
                ("cos", "cos("),
                ("C", ""),
                ("(", "("),
                (")", ")"),
                ("⌫", ""),
            ],
            vec![
                ("tan", "tan("),
                ("π", "pi"),
                ("7", "7"),
                ("8", "8"),
                ("9", "9"),
                ("÷", "÷"),
            ],
            vec![
                ("ln", "ln("),
                ("log", "log("),
                ("4", "4"),
                ("5", "5"),
                ("6", "6"),
                ("×", "×"),
            ],
            vec![
                ("√", "sqrt("),
                ("x²", "^2"),
                ("1", "1"),
                ("2", "2"),
                ("3", "3"),
                ("−", "−"),
            ],
            vec![
                ("xʸ", "^"),
                ("x!", "!"),
                ("±", ""),
                ("0", "0"),
                (".", "."),
                ("+", "+"),
            ],
            vec![("e", "e"), ("ans", "ans"), ("%", "%"), ("=", "")],
        ]
    } else {
        vec![
            vec![("C", ""), ("(", "("), (")", ")"), ("⌫", "")],
            vec![("7", "7"), ("8", "8"), ("9", "9"), ("÷", "÷")],
            vec![("4", "4"), ("5", "5"), ("6", "6"), ("×", "×")],
            vec![("1", "1"), ("2", "2"), ("3", "3"), ("−", "−")],
            vec![("±", ""), ("0", "0"), (".", "."), ("+", "+")],
            vec![("%", "%"), ("=", "")],
        ]
    };
    let mut keys = column![].spacing(spacing.xs());
    for line in lines {
        let mut buttons = iced::widget::Row::new().spacing(spacing.xs());
        for (index, (label, value)) in line.into_iter().enumerate() {
            let message = match label {
                "C" => Message::Clear,
                "⌫" => Message::Backspace,
                "±" => Message::Sign,
                "=" => Message::Evaluate,
                _ => Message::Insert(value.into()),
            };
            let kind = if label == "=" {
                ButtonKind::Accent
            } else if "+−×÷C⌫".contains(label) || (scientific && index < 2) {
                ButtonKind::Secondary
            } else {
                ButtonKind::Primary
            };
            let name = match label {
                "⌫" => "Backspace",
                "±" => "Change sign",
                "×" => "Multiply",
                "÷" => "Divide",
                "−" => "Subtract",
                "C" => "Clear",
                "√" => "Square root",
                "=" => "Calculate · Enter",
                other => other,
            };
            buttons = buttons.push(
                enz_button(label, message)
                    .kind(kind)
                    .size(key_size)
                    .id(iced::advanced::widget::Id::from(format!(
                        "calculator-key-{label}"
                    )))
                    .aria_label(tr(name))
                    .width(if label == "=" {
                        Length::FillPortion(3)
                    } else {
                        Length::Fill
                    })
                    .into_element(),
            );
        }
        keys = keys.push(buttons);
    }
    keys.into()
}

fn converter(app: &Calculator) -> Element<'_, Message> {
    let spacing = app.themes.theme().spacing;
    let units = crate::units::units(app.category);
    let options: Vec<String> = units.iter().map(|unit| tr(unit.name)).collect();
    column![
        enz_view_switcher(
            vec![
                SwitcherPage::new("length", tr(crate::units::CATEGORIES[0])).with_icon("ruler"),
                SwitcherPage::new("mass", tr(crate::units::CATEGORIES[1])).with_icon("weight"),
                SwitcherPage::new("temperature", tr("Temp.")).with_icon("thermometer"),
                SwitcherPage::new("area", tr(crate::units::CATEGORIES[3]))
                    .with_icon("square-dashed"),
                SwitcherPage::new("time", tr(crate::units::CATEGORIES[4])).with_icon("clock"),
            ],
            app.category,
            Message::Category,
        )
        .size(TextFieldSize::Sm)
        .label_size(11.0)
        .stretch(true)
        .aria_label(tr("Conversion quantity")),
        enz_text(tr("Value")).size_px(12.0).tone(TextTone::Muted),
        calculation_input(
            &app.expression,
            "Value or expression",
            if app.expression.is_empty() {
                13.0
            } else {
                18.0
            },
            "calculator-expression".into(),
            "Value to convert",
            false,
        ),
        enz_combo_box(
            tr("From"),
            options.clone(),
            Some(tr(units[app.from].name)),
            |_| Message::From(0)
        )
        .id(iced::advanced::widget::Id::new("calculator-from"))
        .on_select_index(Message::From)
        .width(Length::Fill),
        row![
            Space::new().width(Length::Fill),
            enz_toolbar_icon("arrow-up-down", Message::Swap)
                .id(iced::advanced::widget::Id::new("calculator-swap"))
                .aria_label(tr("Swap units"))
                .tooltip(tr("Swap units")),
            Space::new().width(Length::Fill),
        ]
        .align_y(iced::Alignment::Center),
        enz_combo_box(tr("To"), options, Some(tr(units[app.to].name)), |_| {
            Message::To(0)
        })
        .id(iced::advanced::widget::Id::new("calculator-to"))
        .on_select_index(Message::To)
        .width(Length::Fill),
        enz_text(tr("Result")).size_px(12.0).tone(TextTone::Muted),
        enz_surface(
            row![
                calculation_input(
                    &app.conversion()
                        .map(crate::engine::format_number)
                        .unwrap_or_else(|e| e.to_string()),
                    "",
                    18.0,
                    "conversion-result".into(),
                    "Conversion result",
                    true,
                ),
                enz_toolbar_icon("copy", Message::Copy)
                    .id(iced::advanced::widget::Id::new(
                        "calculator-conversion-copy"
                    ))
                    .aria_label(tr("Copy conversion result"))
                    .tooltip(tr("Copy result")),
            ]
            .align_y(iced::Alignment::Center)
        )
        .tone(SurfaceTone::SurfaceAlt)
        .padding(spacing.space1)
        .width(Length::Fill),
    ]
    .spacing(spacing.space5)
    .into()
}

fn calculation_input<'a>(
    value: &str,
    placeholder: &str,
    font_size: f32,
    identity: String,
    label: &str,
    read_only: bool,
) -> EnzTextField<'a, Message> {
    enz_text_field_owned("", tr(placeholder), value.to_owned(), Message::Input)
        .id(iced::advanced::widget::Id::from(identity))
        .aria_label(tr(label))
        .size(if font_size >= 20.0 {
            TextFieldSize::Lg
        } else {
            TextFieldSize::Sm
        })
        .text_size(font_size)
        .border_variant(FieldVariant::Inline)
        .quiet(true)
        .subtle_focus(!read_only)
        .read_only(read_only)
}
