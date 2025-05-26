use std::io::{self, Write};

fn main() {
    // Test different line ending approaches
    let test_message = r#"{"jsonrpc":"2.0","result":{"protocolVersion":"2024-11-05","serverInfo":{"name":"mcp-server-fetch","version":"0.1.0"},"capabilities":{}},"id":"1"}"#;

    println!("Testing line endings in Rust stdio");
    println!("==================================");

    // Test 1: Using writeln! (platform-specific)
    print!("Test 1 - writeln!: ");
    io::stdout().flush().unwrap();
    writeln!(&mut io::stdout(), "{}", test_message).unwrap();

    // Test 2: Using write! with \n (always Unix-style)
    print!("Test 2 - write! with \\n: ");
    io::stdout().flush().unwrap();
    write!(&mut io::stdout(), "{}\n", test_message).unwrap();
    io::stdout().flush().unwrap();

    // Test 3: Using write! with \r\n (always Windows-style)
    print!("Test 3 - write! with \\r\\n: ");
    io::stdout().flush().unwrap();
    write!(&mut io::stdout(), "{}\r\n", test_message).unwrap();
    io::stdout().flush().unwrap();

    // Show raw bytes
    println!("\nRaw bytes comparison:");
    println!("=====================");

    let mut buffer = Vec::new();

    // writeln! bytes
    writeln!(&mut buffer, "{}", test_message).unwrap();
    println!("writeln! produces: {:?}", String::from_utf8_lossy(&buffer));
    println!("Last bytes: {:?}", &buffer[buffer.len() - 2..]);

    // write! with \n bytes
    buffer.clear();
    write!(&mut buffer, "{}\n", test_message).unwrap();
    println!(
        "\nwrite! with \\n produces: {:?}",
        &buffer[buffer.len() - 2..]
    );

    // write! with \r\n bytes
    buffer.clear();
    write!(&mut buffer, "{}\r\n", test_message).unwrap();
    println!(
        "write! with \\r\\n produces: {:?}",
        &buffer[buffer.len() - 3..]
    );
}
