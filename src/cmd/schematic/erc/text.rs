use crate::cmd::schematic::erc::Severity;
use crate::extract::model::Property;

pub(crate) struct ErcAssertion {
    pub(crate) severity: Severity,
    pub(crate) message: String,
    pub(crate) violation_type: &'static str,
}

pub(crate) fn property_contains_unresolved_variable(
    value: &str,
    symbol_properties: &[Property],
) -> bool {
    value.contains("${")
        && parse_erc_assertion(value).is_none()
        && unresolved_variable_names(value).into_iter().any(|name| {
            !contains_supported_text_variable_name(name)
                && !symbol_properties.iter().any(|property| property.name == name)
        })
}

pub(crate) fn parse_erc_assertion(value: &str) -> Option<ErcAssertion> {
    if let Some(message) = value
        .strip_prefix("${ERC_WARNING")
        .and_then(|rest| rest.strip_suffix('}'))
    {
        return Some(ErcAssertion {
            severity: Severity::Warning,
            message: message.trim().to_string(),
            violation_type: "generic-warning",
        });
    }

    if let Some(message) = value
        .strip_prefix("${ERC_ERROR")
        .and_then(|rest| rest.strip_suffix('}'))
    {
        return Some(ErcAssertion {
            severity: Severity::Error,
            message: message.trim().to_string(),
            violation_type: "generic-error",
        });
    }

    None
}

pub(crate) fn resembles_invalid_stacked_pin(pin: &str) -> bool {
    let has_open = pin.contains('[');
    let has_close = pin.contains(']');

    if !(has_open || has_close) {
        return false;
    }

    if !pin.starts_with('[') || !pin.ends_with(']') {
        return true;
    }

    let inner = &pin[1..pin.len() - 1];
    let mut saw_any = false;

    for raw_part in inner.split(',') {
        let part = raw_part.trim();
        if part.is_empty() {
            continue;
        }
        saw_any = true;

        if let Some((start, end)) = part.split_once('-') {
            let start = start.trim();
            let end = end.trim();
            let (start_prefix, start_num) = parse_alphanumeric_pin(start);
            let (end_prefix, end_num) = parse_alphanumeric_pin(end);

            if start_prefix != end_prefix
                || start_num.is_none()
                || end_num.is_none()
                || start_num > end_num
            {
                return true;
            }
        }
    }

    !saw_any
}

fn unresolved_variable_names(value: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut rest = value;

    while let Some(start) = rest.find("${") {
        let after_start = &rest[start + 2..];
        let Some(end) = after_start.find('}') else {
            break;
        };
        let name = after_start[..end].trim();
        if !name.is_empty() {
            names.push(name);
        }
        rest = &after_start[end + 1..];
    }

    names
}

fn contains_supported_text_variable_name(name: &str) -> bool {
    matches!(name, "INTERSHEET_REFS" | "SHEETNAME" | "SHEETPATH" | "#")
}

fn parse_alphanumeric_pin(text: &str) -> (String, Option<i64>) {
    let split_at = text
        .char_indices()
        .find(|(_, ch)| ch.is_ascii_digit())
        .map(|(idx, _)| idx)
        .unwrap_or(text.len());
    let (prefix, number) = text.split_at(split_at);

    if number.is_empty() {
        return (prefix.to_string(), None);
    }

    (prefix.to_string(), number.parse::<i64>().ok())
}

#[cfg(test)]
mod tests {
    use super::property_contains_unresolved_variable;
    use crate::extract::model::Property;

    fn property(name: &str, value: &str) -> Property {
        Property {
            name: name.to_string(),
            value: value.to_string(),
            x: None,
            y: None,
        }
    }

    #[test]
    fn symbol_property_references_are_not_unresolved_variables() {
        let properties = vec![
            property("Value", "${Sim.Device} ${Sim.Type}"),
            property("Sim.Device", "V"),
            property("Sim.Type", "SIN"),
        ];

        assert!(!property_contains_unresolved_variable(
            "${Sim.Device} ${Sim.Type}",
            &properties,
        ));
    }

    #[test]
    fn missing_symbol_property_references_still_report_unresolved_variables() {
        let properties = vec![property("Value", "${Sim.Device} ${Sim.Kind}")];

        assert!(property_contains_unresolved_variable(
            "${Sim.Device} ${Sim.Kind}",
            &properties,
        ));
    }
}
