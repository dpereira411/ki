use crate::cmd::schematic::erc::Severity;

pub(crate) struct ErcAssertion {
    pub(crate) severity: Severity,
    pub(crate) message: String,
    pub(crate) violation_type: &'static str,
}

pub(crate) fn property_contains_unresolved_variable(value: &str) -> bool {
    value.contains("${")
        && parse_erc_assertion(value).is_none()
        && !contains_supported_text_variable(value)
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

fn contains_supported_text_variable(value: &str) -> bool {
    const KNOWN: &[&str] = &["${INTERSHEET_REFS}", "${SHEETNAME}", "${SHEETPATH}", "${#}"];

    KNOWN.iter().any(|known| value.contains(known))
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
