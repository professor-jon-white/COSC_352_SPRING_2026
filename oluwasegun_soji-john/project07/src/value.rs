use std::fmt;

use chrono::{DateTime, NaiveDate, NaiveDateTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrimitiveKind {
    Integer,
    Float,
    Boolean,
    Date,
    Text,
}

impl fmt::Display for PrimitiveKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            PrimitiveKind::Integer => "integer",
            PrimitiveKind::Float => "float",
            PrimitiveKind::Boolean => "boolean",
            PrimitiveKind::Date => "date",
            PrimitiveKind::Text => "text",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferredType {
    Integer,
    Float,
    Boolean,
    Date,
    Categorical,
    Text,
}

impl fmt::Display for InferredType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            InferredType::Integer => "integer",
            InferredType::Float => "float",
            InferredType::Boolean => "boolean",
            InferredType::Date => "date",
            InferredType::Categorical => "categorical",
            InferredType::Text => "text",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone)]
pub struct ValueInspection<'a> {
    raw: &'a str,
    numeric_value: Option<f64>,
    date_value: Option<NaiveDate>,
    primary_kind: PrimitiveKind,
}

impl<'a> ValueInspection<'a> {
    pub fn inspect(raw: &'a str) -> Self {
        let integer_value = raw.parse::<i64>().ok();
        let float_value = parse_float(raw);
        let boolean_value = parse_bool(raw);
        let date_value = parse_date(raw);
        let numeric_value = integer_value.map(|value| value as f64).or(float_value);

        let primary_kind = if integer_value.is_some() {
            PrimitiveKind::Integer
        } else if float_value.is_some() {
            PrimitiveKind::Float
        } else if boolean_value.is_some() {
            PrimitiveKind::Boolean
        } else if date_value.is_some() {
            PrimitiveKind::Date
        } else {
            PrimitiveKind::Text
        };

        Self {
            raw,
            numeric_value,
            date_value,
            primary_kind,
        }
    }

    pub fn raw(&self) -> &str {
        self.raw
    }

    pub fn numeric_value(&self) -> Option<f64> {
        self.numeric_value
    }

    pub fn date_value(&self) -> Option<NaiveDate> {
        self.date_value
    }

    pub fn primary_kind(&self) -> PrimitiveKind {
        self.primary_kind
    }
}

pub fn parse_bool(raw: &str) -> Option<bool> {
    match raw.to_ascii_lowercase().as_str() {
        "true" | "t" | "yes" | "y" => Some(true),
        "false" | "f" | "no" | "n" => Some(false),
        _ => None,
    }
}

fn parse_float(raw: &str) -> Option<f64> {
    let parsed = raw.parse::<f64>().ok()?;
    parsed.is_finite().then_some(parsed)
}

fn parse_date(raw: &str) -> Option<NaiveDate> {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(raw) {
        return Some(parsed.date_naive());
    }

    for format in DATE_TIME_FORMATS {
        if let Ok(parsed) = NaiveDateTime::parse_from_str(raw, format) {
            return Some(parsed.date());
        }
    }

    for format in DATE_FORMATS {
        if let Ok(parsed) = NaiveDate::parse_from_str(raw, format) {
            return Some(parsed);
        }
    }

    None
}

const DATE_FORMATS: &[&str] = &[
    "%Y-%m-%d", "%m/%d/%Y", "%m/%d/%y", "%Y/%m/%d", "%d/%m/%Y", "%d-%m-%Y", "%Y.%m.%d", "%b %d %Y",
    "%B %d %Y", "%d %b %Y", "%d %B %Y",
];

const DATE_TIME_FORMATS: &[&str] = &[
    "%Y-%m-%d %H:%M:%S",
    "%Y-%m-%d %H:%M",
    "%m/%d/%Y %H:%M:%S",
    "%m/%d/%Y %H:%M",
    "%m/%d/%y %H:%M:%S",
    "%m/%d/%y %H:%M",
];

#[cfg(test)]
mod tests {
    use super::{PrimitiveKind, ValueInspection};

    #[test]
    fn inspects_common_value_types() {
        let integer = ValueInspection::inspect("42");
        assert_eq!(integer.primary_kind(), PrimitiveKind::Integer);
        assert_eq!(integer.numeric_value(), Some(42.0));

        let float = ValueInspection::inspect("3.14");
        assert_eq!(float.primary_kind(), PrimitiveKind::Float);
        assert_eq!(float.numeric_value(), Some(3.14));

        let boolean = ValueInspection::inspect("Yes");
        assert_eq!(boolean.primary_kind(), PrimitiveKind::Boolean);

        let date = ValueInspection::inspect("2026-04-24");
        assert_eq!(date.primary_kind(), PrimitiveKind::Date);
        assert!(date.date_value().is_some());

        let text = ValueInspection::inspect("hello world");
        assert_eq!(text.primary_kind(), PrimitiveKind::Text);
    }
}
