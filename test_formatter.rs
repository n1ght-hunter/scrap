use std::fs;

fn main() {
    let source = fs::read_to_string("tests/basic.sc").expect("Failed to read file");

    let config = scrap_formatter::FormatterConfig::default();
    let formatted = scrap_formatter::format_file(&source, &config);

    println!("{}", formatted);
}
