use crate::engine::CalcError;

#[derive(Clone, Copy)]
pub(crate) struct Unit {
    pub name: &'static str,
    factor: f64,
    offset: f64,
}
const fn unit(name: &'static str, factor: f64) -> Unit {
    Unit {
        name,
        factor,
        offset: 0.0,
    }
}
pub(crate) const CATEGORIES: [&str; 5] = ["Length", "Mass", "Temperature", "Area", "Time"];
const MASS: &[Unit] = &[
    unit("Kilograms", 1.0),
    unit("Grams", 0.001),
    unit("Pounds", 0.45359237),
    unit("Ounces", 0.028349523125),
];
const TEMPERATURE: &[Unit] = &[
    Unit {
        name: "Celsius",
        factor: 1.0,
        offset: 273.15,
    },
    Unit {
        name: "Fahrenheit",
        factor: 5.0 / 9.0,
        offset: 255.3722222222222,
    },
    unit("Kelvin", 1.0),
];
const AREA: &[Unit] = &[
    unit("Square metres", 1.0),
    unit("Square kilometres", 1e6),
    unit("Hectares", 10000.0),
    unit("Acres", 4046.8564224),
    unit("Square feet", 0.09290304),
];
const TIME: &[Unit] = &[
    unit("Seconds", 1.0),
    unit("Minutes", 60.0),
    unit("Hours", 3600.0),
    unit("Days", 86400.0),
    unit("Weeks", 604800.0),
];
const LENGTH: &[Unit] = &[
    unit("Metres", 1.0),
    unit("Kilometres", 1000.0),
    unit("Centimetres", 0.01),
    unit("Millimetres", 0.001),
    unit("Miles", 1609.344),
    unit("Yards", 0.9144),
    unit("Feet", 0.3048),
    unit("Inches", 0.0254),
];
pub(crate) fn units(category: usize) -> &'static [Unit] {
    match category {
        1 => MASS,
        2 => TEMPERATURE,
        3 => AREA,
        4 => TIME,
        _ => LENGTH,
    }
}
pub(crate) fn convert(
    value: f64,
    category: usize,
    from: usize,
    to: usize,
) -> Result<f64, CalcError> {
    let list = units(category);
    let a = list.get(from).ok_or(CalcError::Syntax)?;
    let b = list.get(to).ok_or(CalcError::Syntax)?;
    let base = value * a.factor + a.offset;
    if category == 2 && base < -1e-10 {
        return Err(CalcError::Domain);
    }
    let result = (base - b.offset) / b.factor;
    if result.is_finite() {
        Ok(result)
    } else {
        Err(CalcError::Domain)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn calculator_conversion_contract() {
        assert!((convert(1.0, 0, 4, 0).expect("mile") - 1609.344).abs() < 1e-9);
        assert!((convert(32.0, 2, 1, 0).expect("freezing")).abs() < 1e-9);
        assert!((convert(100.0, 2, 0, 1).expect("boiling") - 212.0).abs() < 1e-9);
        assert_eq!(convert(-274.0, 2, 0, 2), Err(CalcError::Domain));
        for category in 0..5 {
            for index in 0..units(category).len() {
                let intermediate = convert(42.0, category, 0, index).expect("forward");
                assert!(
                    (convert(intermediate, category, index, 0).expect("back") - 42.0).abs() < 1e-8
                );
            }
        }
    }
}
