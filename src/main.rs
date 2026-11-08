#![forbid(unsafe_code)]
mod app;
mod engine;
mod i18n;
mod shortcuts;
mod storage;
mod units;
mod view;

const APP_ID: &str = "org.enzimos.Calculator";

fn main() -> iced::Result {
    use enzui_widgets::app_shell::{DesktopApplicationIdentity, ShellLayout};
    use enzui_widgets::single_instance::{InstanceStart, SingleInstance};
    let instance = match SingleInstance::start(APP_ID) {
        InstanceStart::Forwarded => return Ok(()),
        InstanceStart::Owner(instance, _) => std::sync::Mutex::new(Some(instance)),
    };
    let identity = DesktopApplicationIdentity::new(APP_ID).expect("valid desktop identity");
    let window = identity.apply_to_window_settings(iced::window::Settings {
        size: iced::Size::new(420.0, 620.0),
        min_size: Some(iced::Size::new(360.0, 540.0)),
        exit_on_close_request: false,
        ..identity.window_settings(ShellLayout::TitlebarContent)
    });
    enzui_widgets::app_setup::run_with_services(
        iced::application(
            move || {
                let (mut app, task) = app::Calculator::new();
                app.desktop = instance
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take()
                    .unwrap_or_else(SingleInstance::standalone);
                (app, task)
            },
            app::Calculator::update,
            view::root,
        )
        .title(|_: &app::Calculator| i18n::tr("Calculator"))
        .theme(app::Calculator::theme)
        .subscription(app::Calculator::subscription)
        .window(window),
        enzui_widgets::app_setup::AppServices::default().localizer(i18n::from_environment()),
    )
}
