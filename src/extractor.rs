use anyhow::{anyhow, Result};

use crate::types::{FloatFormat, NumberFormat};

pub(crate) fn extract_number_format(from: &str) -> Result<NumberFormat> {
    if from.len() == 0 {
        return Ok(NumberFormat::default());
    }

    let mut result = NumberFormat::default();

    result.digits = from.parse::<u16>().ok();
    result.fill_zeros = from.chars().next().expect("Unexpected end of input") == '0';

    Ok(result)
}

pub(crate) fn extract_float_format(from: &str) -> Result<FloatFormat> {
    if !from.contains('.') {
        return Ok(FloatFormat {
            base: extract_number_format(from)?,
            ..Default::default()
        });
    }

    let mut splitted = from.split('.');

    let base = extract_number_format(
        splitted
            .next()
            .ok_or_else(|| anyhow!("Unexpected end of input"))?,
    )?;

    let exponent = if let Some(s) = splitted.next() {
        extract_number_format(s)?
    } else {
        NumberFormat::default()
    };

    Ok(FloatFormat {
        base,
        fraction: exponent,
    })
}
