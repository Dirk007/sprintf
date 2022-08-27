use anyhow::{anyhow, bail, Result};

use crate::{
    extractor::{extract_float_format, extract_number_format},
    types::{FloatFormat, HexFormat, NumberFormat},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Placeholder {
    Display,              //< %v
    String,               //< %s
    Float(FloatFormat),   //< %f
    Number(NumberFormat), //< %d
    Hex(HexFormat),       //< %x
}

/// ```
/// use printer_lib::{extractor::*, parser::*, types::*};
///
/// let res: Placeholder = "%2d".to_string().try_into().unwrap();
/// assert_eq!(
///     res,
///     Placeholder::Number(NumberFormat {
///         digits: Some(2),
///         fill_zeros: false
///     })
/// );
///
/// let res: Placeholder = "%04d".to_string().try_into().unwrap();
/// assert_eq!(
///     res,
///     Placeholder::Number(NumberFormat {
///         digits: Some(4),
///         fill_zeros: true
///     })
/// );
///
/// let res: Placeholder = "%08x".to_string().try_into().unwrap();
/// assert_eq!(
///     res,
///     Placeholder::Hex(HexFormat {
///         uppercase: false,
///         nf: NumberFormat {
///             digits: Some(8),
///             fill_zeros: true,
///         },
///     })
/// );
///
/// let res: Placeholder = "%2X".to_string().try_into().unwrap();
/// assert_eq!(
///     res,
///     Placeholder::Hex(HexFormat {
///         uppercase: true,
///         nf: NumberFormat {
///             digits: Some(2),
///             fill_zeros: false,
///         },
///     })
/// );
///
/// let res: Placeholder = "%v".to_string().try_into().unwrap();
/// assert_eq!(res, Placeholder::Display);
///
/// let res: Placeholder = "%s".to_string().try_into().unwrap();
/// assert_eq!(res, Placeholder::String);
///
/// let res: Placeholder = "%.02f".to_string().try_into().unwrap();
/// assert_eq!(
///     res,
///     Placeholder::Float(FloatFormat {
///         fraction: NumberFormat {
///             digits: Some(2),
///             fill_zeros: true
///         },
///         ..Default::default()
///     })
/// );
/// ```
impl TryFrom<String> for Placeholder {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        if s.len() < 2 {
            panic!("Placeholder must be at least 2 characters long");
        }
        if s.chars().next().ok_or_else(|| anyhow!("Unexpected end of input"))? != '%' {
            bail!("Placeholder have to begin with '%'")
        }

        let what = s
            .chars()
            .rev()
            .next()
            .ok_or_else(|| anyhow!("Unexpected end of input"))?;
        let cutted_s = &s[1..s.len() - 1];
        match what {
            'v' => Ok(Self::Display),
            's' => Ok(Self::String),
            'd' => Ok(Self::Number(extract_number_format(cutted_s)?)),
            'x' | 'X' => Ok(Self::Hex(HexFormat {
                nf: extract_number_format(cutted_s)?,
                uppercase: what == 'X',
            })),
            'f' => Ok(Self::Float(extract_float_format(cutted_s)?)),
            _ => bail!("Placeholder '{}' unknown", what),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    Text(String),
    Placeholder(Placeholder),
}

impl TryFrom<String> for Entry {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        if s.starts_with("%%") {
            return Ok(Self::Text(s[1..].into()));
        }
        match s.chars().next().ok_or_else(|| anyhow!("Empty input string"))? {
            '%' => Ok(Self::Placeholder(s.try_into()?)),
            _ => Ok(Self::Text(s)),
        }
    }
}

#[derive(Debug)]
pub struct ParsedFormatString {
    pub entries: Vec<Entry>,
    pub variables: Vec<String>,
}

impl ParsedFormatString {
    pub fn new(entries: Vec<Entry>, variables: Vec<String>) -> Self {
        Self { entries, variables }
    }
}

/// Explodes `input` into Entry::Placeholder and Entry::Text parts.
/// ```
/// use printer_lib::{extractor::*, parser::*, types::*};
///
/// let x = explode("%x this is a test %d hello %.2fh with 42%% foo").unwrap();
/// println!("{:?}", x);
/// assert_eq!(x.len(), 7);
/// assert!(matches!(x[0], Entry::Placeholder(_)));
/// assert_eq!(x[1], Entry::Text(" this is a test ".to_string()));
/// assert!(matches!(x[2], Entry::Placeholder(_)));
/// assert_eq!(x[3], Entry::Text(" hello ".to_string()));
/// assert!(matches!(x[4], Entry::Placeholder(_)));
/// assert_eq!(x[5], Entry::Text("h with 42".to_string()));
/// assert_eq!(x[6], Entry::Text("% foo".to_string()));
/// ```
pub fn explode(input: &str) -> Result<Vec<Entry>> {
    let mut result: Vec<Entry> = Vec::new();
    let mut in_placeholder = false;
    let mut buffer = String::new();
    for c in input.chars() {
        match c {
            '%' if in_placeholder => {
                // %%
                in_placeholder = false;
                buffer.push(c);
            }
            '%' => {
                if buffer.len() > 0 {
                    result.push(buffer.try_into()?);
                }
                buffer = String::new();
                buffer.push(c);
                in_placeholder = true
            }
            'a'..='z' | 'A'..='Z' => {
                buffer.push(c);
                if in_placeholder {
                    in_placeholder = false;
                    result.push(buffer.try_into()?);
                    buffer = String::new();
                }
            }
            _ => buffer.push(c),
        }
    }
    if buffer.len() > 0 {
        result.push(buffer.try_into()?);
    }

    Ok(result)
}

pub fn parse_format_string(input: &str) -> Result<ParsedFormatString> {
    let mut variables = Vec::<String>::new();

    let text_start = input.find('\"').unwrap_or(0) + 1;
    let text_end = input[text_start + 1..]
        .find('\"')
        .ok_or(anyhow!("No terminating quote found"))?
        + 1;

    let placeholder = explode(&input[text_start..text_start + text_end])?;

    let vars = input[2 + text_end..].split(",");
    for var in vars {
        // var could also be the maybe existing comma behind the text
        if var.len() > 0 {
            variables.push(var.trim().into());
        }
    }

    let placeholder_count = placeholder
        .iter()
        .filter(|item| matches!(item, Entry::Placeholder(_)))
        .count();

    if variables.len() != placeholder_count {
        bail!(
            "Unmatched variables({}) and placeholders({})",
            variables.len(),
            placeholder_count
        );
    }

    Ok(ParsedFormatString::new(placeholder, variables))
}
