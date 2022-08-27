use std::collections::HashMap;

use metrics_evaluation::{MapResolver, Value};
use sprintf::{parser::parse_format_string, printer::sprintf};

fn main() {
    let s = r#""Hello, %s - this is test number %d in %.06fs having 0x%02X%% matches and %06d zeroes", user.name, user.tries, test.seconds, test.percent, test.zeroes"#;

    let mut values = HashMap::new();
    values.insert("user.name", Value::String("FooUser".into()));
    values.insert("user.tries", Value::Numeric(42.into()));
    values.insert("test.seconds", Value::Numeric(1.4711));
    values.insert("test.percent", Value::Numeric(8.into()));
    values.insert("test.zeroes", Value::Numeric(6.into()));
    let values: MapResolver = values.into();

    // FTR: parsed can be re-used.
    let parsed = parse_format_string(s).expect("Bamm");
    println!("Parsed: {:?}", parsed);

    let s = sprintf(&parsed, &values);
    println!("Printed: {:?}", s);
}
