use std::io::{self, IsTerminal};

/// Prints key-value pairs to stdout, handling binary data gracefully.
///
/// When stdout is a terminal and `show_binary` is false, binary values
/// are replaced with "(omitted binary data)". When piped, all data is
/// passed through raw.
pub fn print_kv(key: &[u8], value: &[u8], delimiter: &str, show_binary: bool) {
    if show_binary || !io::stdout().is_terminal() {
        println!(
            "{}{delimiter}{}",
            String::from_utf8_lossy(key),
            String::from_utf8_lossy(value)
        );
    } else {
        let key_str = safe_string(key);
        let val_str = safe_string(value);
        println!("{}{delimiter}{}", key_str, val_str);
    }
}

/// Prints a single value to stdout.
pub fn print_value(data: &[u8]) {
    let stdout = io::stdout();
    if !stdout.is_terminal() {
        // When piped, output raw bytes
        use std::io::Write;
        let mut handle = stdout.lock();
        let _ = handle.write_all(data);
    } else {
        println!("{}", safe_string(data));
    }
}

/// Prints a key only.
pub fn print_key(key: &[u8]) {
    println!("{}", safe_string(key));
}

/// Returns a display-safe string: if valid UTF-8, return as-is;
/// otherwise return "(omitted binary data)".
fn safe_string(data: &[u8]) -> String {
    match std::str::from_utf8(data) {
        Ok(s) => s.to_string(),
        Err(_) => "(omitted binary data)".to_string(),
    }
}
