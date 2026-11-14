use arity_core::Decimal;
use arity_units::Unit;
use serde::Serialize;

/// A reported value in the display unit of its property, for tables and
/// figures. arity calculates it from the reported value and unit.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DisplayValue {
    /// The UCUM code of the display unit.
    pub unit: String,
    pub value: Option<f64>,
    pub low: Option<f64>,
    pub high: Option<f64>,
    /// The lower end of the uncertainty interval, in the display unit.
    pub interval_low: Option<f64>,
    /// The upper end of the uncertainty interval, in the display unit.
    pub interval_high: Option<f64>,
}

fn decimal(value: &serde_json::Value, field: &str) -> Option<Decimal> {
    value.get(field)?.as_str()?.parse().ok()
}

/// Converts the value of a report body into the display unit, when the two
/// units measure the same quantity.
pub fn display_value(value: &serde_json::Value, display_unit: &str) -> Option<DisplayValue> {
    let from = Unit::parse(value.get("unit")?.as_str()?).ok()?;
    let to = Unit::parse(display_unit).ok()?;
    if !from.is_commensurable_with(&to) {
        return None;
    }
    let absolute =
        |number: Option<Decimal>| number.and_then(|number| from.convert(number.to_f64(), &to).ok());
    let ratio = from.scale() / to.scale();
    let number = decimal(value, "number");
    let converted = absolute(number);
    let uncertainty = value.get("uncertainty");
    let (interval_low, interval_high) = match uncertainty {
        Some(uncertainty) => match decimal(uncertainty, "number") {
            Some(half_width) => {
                let width = half_width.to_f64() * ratio;
                (converted.map(|v| v - width), converted.map(|v| v + width))
            }
            None => (
                absolute(decimal(uncertainty, "low")),
                absolute(decimal(uncertainty, "high")),
            ),
        },
        None => (None, None),
    };
    Some(DisplayValue {
        unit: display_unit.to_owned(),
        value: converted,
        low: absolute(decimal(value, "low")),
        high: absolute(decimal(value, "high")),
        interval_low,
        interval_high,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn close(actual: Option<f64>, expected: f64) -> bool {
        actual.is_some_and(|actual| (actual - expected).abs() <= 1e-9 * expected.abs().max(1.0))
    }

    #[test]
    fn converts_a_value_and_its_uncertainty() {
        let value = json!({
            "number": "240", "unit": "nmol/L",
            "uncertainty": { "kind": "sd", "number": "20" }
        });
        let display = display_value(&value, "umol/L").unwrap();
        assert!(close(display.value, 0.24));
        assert!(close(display.interval_low, 0.22));
        assert!(close(display.interval_high, 0.26));
    }

    #[test]
    fn converts_ranges_and_intervals_as_absolute_values() {
        let value = json!({
            "low": "20", "high": "25", "unit": "Cel",
            "uncertainty": { "kind": "ci", "low": "19", "high": "26" }
        });
        let display = display_value(&value, "K").unwrap();
        assert!(close(display.low, 293.15));
        assert!(close(display.high, 298.15));
        assert!(close(display.interval_low, 292.15));
        assert_eq!(display.value, None);
    }

    #[test]
    fn a_temperature_difference_does_not_take_the_offset() {
        let value = json!({
            "number": "37", "unit": "Cel",
            "uncertainty": { "kind": "sd", "number": "1" }
        });
        let display = display_value(&value, "K").unwrap();
        assert!(close(display.value, 310.15));
        assert!(close(display.interval_high, 311.15));
    }

    #[test]
    fn gives_nothing_for_units_of_different_quantities() {
        let value = json!({ "number": "1", "unit": "mg" });
        assert_eq!(display_value(&value, "umol/L"), None);
        assert_eq!(display_value(&json!({ "number": "1" }), "umol/L"), None);
    }
}
