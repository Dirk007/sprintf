use std::fmt::{Display, LowerHex, UpperHex};

use anyhow::{anyhow, bail, Result};
use metrics_evaluation::{Resolver, Value};

use crate::{
    parser::{Entry, ParsedFormatString, Placeholder},
    types::{HexFormat, NumberFormat},
};

const DEFAULT_FRACT_DIGITS: u16 = 2;

fn get_string(value: &Value) -> Result<&String> {
    match value {
        Value::String(s) => Ok(s),
        _ => bail!("Not a string value"),
    }
}

fn get_number(value: &Value) -> Result<&f64> {
    match value {
        Value::Numeric(n) => Ok(n),
        _ => bail!("Not a numeric value"),
    }
}

enum FillStyle {
    Prepend,
    Append,
}

fn round(number: f64, decimals: u16) -> f64 {
    let y = 10i32.pow(decimals as u32) as f64;
    (number * y).round() / y
}

fn print_number(format: &NumberFormat, value: impl Display, fill_style: FillStyle) -> String {
    // Prepend
    // 08d for 123 = 00000123
    // 02d for 123 = 123
    // 2d for 123 = 123

    let mut repr = format!("{}", value);
    if let (Some(digits), true) = (format.digits, format.fill_zeros) {
        match fill_style {
            FillStyle::Prepend => (repr.len() as u16..digits).for_each(|_| repr.insert(0, '0')),
            FillStyle::Append => (repr.len() as u16..digits).for_each(|_| repr.push('0')),
        }
    }
    repr
}

fn print_hex(format: &HexFormat, value: impl UpperHex + LowerHex) -> String {
    let mut repr = if format.uppercase {
        format!("{:X}", value)
    } else {
        format!("{:x}", value)
    };
    if let (Some(digits), true) = (format.nf.digits, format.nf.fill_zeros) {
        (repr.len() as u16..digits).for_each(|_| repr.insert(0, '0'));
    }
    repr
}

/// ```
/// use metrics_evaluation::Value;
/// use sprintf::{
///     parser::Placeholder,
///     printer::print_value,
///     types::{FloatFormat, NumberFormat},
/// };
///
/// let s = print_value(
///     &Placeholder::Number(NumberFormat {
///         digits: Some(3),
///         fill_zeros: false,
///     }),
///     &123.into(),
/// )
/// .unwrap();
/// assert_eq!(s, "123".to_string());
///
/// let s = print_value(
///     &Placeholder::Number(NumberFormat {
///         digits: Some(3),
///         fill_zeros: false,
///     }),
///     &Value::Numeric(-123f64),
/// )
/// .unwrap();
/// assert_eq!(s, "-123".to_string());
///
/// let s = print_value(
///     &Placeholder::Number(NumberFormat {
///         digits: Some(3),
///         fill_zeros: true,
///     }),
///     &123.into(),
/// )
/// .unwrap();
/// assert_eq!(s, "123".to_string());
///
/// let s = print_value(
///     &Placeholder::Number(NumberFormat {
///         digits: Some(5),
///         fill_zeros: true,
///     }),
///     &123.into(),
/// )
/// .unwrap();
/// assert_eq!(s, "00123".to_string());
///
/// let s = print_value(&Placeholder::Float(FloatFormat::default()), &42.123.into()).unwrap();
/// assert_eq!(s, "42.12".to_string());
///
/// let s = print_value(&Placeholder::Float(FloatFormat::default()), &42.125.into()).unwrap();
/// assert_eq!(s, "42.13".to_string());
///
/// let s = print_value(
///     &Placeholder::Float(FloatFormat {
///         fraction: NumberFormat {
///             digits: Some(1),
///             fill_zeros: false,
///         },
///         ..Default::default()
///     }),
///     &42.123.into(),
/// )
/// .unwrap();
/// assert_eq!(s, "42.1".to_string());
///
/// let s = print_value(
///     &Placeholder::Float(FloatFormat {
///         fraction: NumberFormat {
///             digits: Some(4),
///             fill_zeros: true,
///         },
///         ..Default::default()
///     }),
///     &42.12.into(),
/// )
/// .unwrap();
/// assert_eq!(s, "42.1200".to_string());
///
/// let s = print_value(
///     &Placeholder::Float(FloatFormat {
///         base: NumberFormat::default(),
///         fraction: NumberFormat {
///             digits: Some(5),
///             fill_zeros: true,
///         },
///     }),
///     &42.1.into(),
/// )
/// .unwrap();
/// assert_eq!(s, "42.10000".to_string());
/// ```
pub fn print_value(format: &Placeholder, value: &Value) -> Result<String> {
    let result = match format {
        Placeholder::Display => format!("{}", value),
        Placeholder::String => format!("{}", get_string(value)?),
        Placeholder::Number(nf) => print_number(nf, get_number(value)?.trunc() as i128, FillStyle::Prepend),
        Placeholder::Hex(hf) => print_hex(hf, get_number(value)?.trunc() as i128),
        Placeholder::Float(ff) => {
            let base = print_number(&ff.base, get_number(value)?.trunc() as i128, FillStyle::Prepend);
            let digits: u16 = ff.fraction.digits.unwrap_or_else(|| DEFAULT_FRACT_DIGITS);
            let fract = get_number(value)?.fract();
            // let value = (fract * 10f64.powf(digits as f64)).trunc() as i128;
            let value = (round(fract, digits) * 10f64.powf(digits as f64)).trunc() as i128;
            let exponent = print_number(&ff.fraction, value, FillStyle::Append);
            format!("{}.{}", base, exponent)
        }
    };

    Ok(result)
}

pub fn sprintf(parsed: &ParsedFormatString, resolver: &impl Resolver) -> Result<String> {
    let mut result: String = String::new();

    let mut vars = parsed.variables.iter();
    for entry in &parsed.entries {
        match entry {
            Entry::Text(text) => {
                result.push_str(text);
            }
            Entry::Placeholder(format) => {
                let variable_name = vars
                    .next()
                    .ok_or_else(|| anyhow!("No variable for placeholder {:?}", format))?;
                let value = resolver
                    .resolve(variable_name)
                    .ok_or_else(|| anyhow!("Unable to resolve variable {:?}", variable_name))?;
                result.push_str(print_value(format, value)?.as_str());
            }
        }
    }

    Ok(result)
}
