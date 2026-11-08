use enzui_core::localization::{self, CatalogBuilder, Localizer};

pub(crate) fn from_environment() -> Localizer {
    let mut catalog = CatalogBuilder::new();
    for (locale, source) in [
        ("en-US", include_str!("../assets/locales/en-US.ftl")),
        ("de", include_str!("../assets/locales/de.ftl")),
        ("ar", include_str!("../assets/locales/ar.ftl")),
    ] {
        catalog
            .add_resource(locale, source)
            .expect("valid embedded calculator catalog");
    }
    Localizer::new(
        catalog.build(),
        localization::preferred_locales(),
        localization::Locale::default(),
    )
}

pub(crate) fn tr(source: &str) -> String {
    let key = match source {
        "Calculator" => "calc-calculator",
        "Basic" => "calc-basic",
        "Scientific" => "calc-scientific",
        "Convert" => "calc-convert",
        "Calculator mode" => "calc-calculator-mode",
        "Copy result" => "calc-copy-result",
        "Paste expression" => "calc-paste-expression",
        "Clear history" => "calc-clear-history",
        "Keyboard Shortcuts" => "calc-keyboard-shortcuts",
        "Use" => "calc-use",
        "Result" => "calc-result",
        "Store result in memory" => "calc-store-result-in-memory",
        "Recall memory" => "calc-recall-memory",
        "Clear memory" => "calc-clear-memory",
        "Toggle degrees and radians" => "calc-toggle-degrees-and-radians",
        "Backspace" => "calc-backspace",
        "Change sign" => "calc-change-sign",
        "Multiply" => "calc-multiply",
        "Divide" => "calc-divide",
        "Subtract" => "calc-subtract",
        "Clear" => "calc-clear",
        "Square root" => "calc-square-root",
        "Calculate · Enter" => "calc-calculate-enter",
        "Conversion quantity" => "calc-conversion-quantity",
        "Value" => "calc-value",
        "Value or expression" => "calc-value-or-expression",
        "Value to convert" => "calc-value-to-convert",
        "From" => "calc-from",
        "To" => "calc-to",
        "Swap units" => "calc-swap-units",
        "Conversion result" => "calc-conversion-result",
        "Copy conversion result" => "calc-copy-conversion-result",
        "Calculation and result" => "calc-calculation-and-result",
        "Length" => "calc-length",
        "Mass" => "calc-mass",
        "Temperature" => "calc-temperature",
        "Temp." => "calc-temp",
        "Area" => "calc-area",
        "Time" => "calc-time",
        "Kilograms" => "calc-kilograms",
        "Grams" => "calc-grams",
        "Pounds" => "calc-pounds",
        "Ounces" => "calc-ounces",
        "Celsius" => "calc-celsius",
        "Fahrenheit" => "calc-fahrenheit",
        "Kelvin" => "calc-kelvin",
        "Square metres" => "calc-square-metres",
        "Square kilometres" => "calc-square-kilometres",
        "Hectares" => "calc-hectares",
        "Acres" => "calc-acres",
        "Square feet" => "calc-square-feet",
        "Seconds" => "calc-seconds",
        "Minutes" => "calc-minutes",
        "Hours" => "calc-hours",
        "Days" => "calc-days",
        "Weeks" => "calc-weeks",
        "Metres" => "calc-metres",
        "Kilometres" => "calc-kilometres",
        "Centimetres" => "calc-centimetres",
        "Millimetres" => "calc-millimetres",
        "Miles" => "calc-miles",
        "Yards" => "calc-yards",
        "Feet" => "calc-feet",
        "Inches" => "calc-inches",
        "Check the expression and parentheses" => "calc-check-the-expression-and-parentheses",
        "Cannot divide by zero" => "calc-cannot-divide-by-zero",
        "Result is outside the real number range" => "calc-result-is-outside-the-real-number-range",
        "Expression is too long or deeply nested" => "calc-expression-is-too-long-or-deeply-nested",
        "Could not copy the result" => "calc-could-not-copy-the-result",
        "Pasted expression is too long" => "calc-pasted-expression-is-too-long",
        "Could not read text from the clipboard" => "calc-could-not-read-text-from-the-clipboard",
        "No text found on the clipboard" => "calc-no-text-found-on-the-clipboard",
        "Result copied" => "calc-result-copied",
        "History storage is unavailable" => "calc-history-storage-is-unavailable",
        "Could not save calculation history" => "calc-could-not-save-calculation-history",
        "History could not be loaded; the existing file is preserved" => {
            "calc-history-could-not-be-loaded-the-existing-file-is-preserved"
        }
        other => return other.to_owned(),
    };
    let translated = localization::text(key);
    if translated == key {
        source.to_owned()
    } else {
        translated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculator_catalog_scopes_localize_app_errors_and_direction() {
        let localizer = from_environment();
        for (language, title, error, direction) in [
            (
                "de",
                "Rechner",
                "Division durch null ist nicht möglich",
                enzui_core::placement::Direction::LeftToRight,
            ),
            (
                "ar",
                "الحاسبة",
                "لا يمكن القسمة على صفر",
                enzui_core::placement::Direction::RightToLeft,
            ),
            (
                "en-US",
                "Calculator",
                "Cannot divide by zero",
                enzui_core::placement::Direction::LeftToRight,
            ),
        ] {
            localizer.select(vec![language.parse().expect("locale")]);
            localizer.scope(|| {
                assert_eq!(tr("Calculator"), title);
                assert_eq!(crate::engine::CalcError::DivisionByZero.to_string(), error);
                assert_eq!(localization::current().locale().direction(), direction);
                assert_eq!(
                    crate::engine::format_number(
                        crate::engine::evaluate("2+3*4", true, 0.0).expect("expression")
                    ),
                    "14"
                );
            });
        }
    }
}
