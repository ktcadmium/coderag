use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(author, version, about = "MCP Debug Tool", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Test the MCP server binary
    TestServer {
        /// Path to the MCP server binary
        #[arg(short, long, default_value = "./target/debug/coderag-mcp")]
        server: String,

        /// Enable verbose debug output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Act as an MCP server for testing client behavior
    MockServer {
        /// Enable verbose debug output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Send raw requests to server
    Raw {
        /// Path to the MCP server binary
        #[arg(short, long, default_value = "./target/debug/coderag-mcp")]
        server: String,

        /// JSON-RPC method
        #[arg(short, long)]
        method: String,

        /// JSON params (as string)
        #[arg(short, long, default_value = "{}")]
        params: String,

        /// Request ID
        #[arg(short, long, default_value = "1")]
        id: u64,
    },

    /// Test MCP handshake protocol
    Handshake {
        /// Path to the MCP server binary
        #[arg(short, long, default_value = "./target/debug/coderag-mcp")]
        server: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct JsonRpcRequest {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

async fn test_server(server_path: &str, verbose: bool) -> Result<()> {
    info!("🧪 Testing MCP server: {}", server_path);

    // Test sequence
    let test_sequence = vec![
        (
            "Initialize",
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: Some(json!(1)),
                method: "initialize".to_string(),
                params: Some(json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "clientInfo": {
                        "name": "mcp-debug",
                        "version": "0.1.0"
                    }
                })),
            },
        ),
        (
            "Initialized notification",
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: None, // Notifications don't have IDs
                method: "initialized".to_string(),
                params: Some(json!({})),
            },
        ),
        (
            "List tools",
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: Some(json!(3)),
                method: "tools/list".to_string(),
                params: Some(json!({})),
            },
        ),
        (
            "Call list_docs",
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: Some(json!(4)),
                method: "tools/call".to_string(),
                params: Some(json!({
                    "name": "list_docs",
                    "arguments": {}
                })),
            },
        ),
    ];

    // Start the server
    let mut cmd = Command::new(server_path)
        .args(if verbose { vec!["--debug"] } else { vec![] })
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn MCP server")?;

    let mut stdin = cmd.stdin.take().context("Failed to get stdin")?;
    let stdout = cmd.stdout.take().context("Failed to get stdout")?;
    let stderr = cmd.stderr.take().context("Failed to get stderr")?;

    // Spawn stderr reader
    let stderr_handle = tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = Vec::new();
        for line in reader.lines().map_while(Result::ok) {
            if verbose {
                eprintln!("[STDERR] {}", line);
            }
            lines.push(line);
        }
        lines
    });

    // Create stdout reader
    let stdout_reader = Arc::new(Mutex::new(BufReader::new(stdout).lines()));

    // Run test sequence
    for (step_name, request) in test_sequence {
        info!("\n📤 Step: {}", step_name);

        let request_str = serde_json::to_string(&request)?;
        if verbose {
            info!("Request: {}", request_str);
        }

        // Send request
        writeln!(stdin, "{}", request_str)?;
        stdin.flush()?;

        // For notifications (no ID), we don't expect a response
        if request.id.is_none() {
            info!("✅ Notification sent (no response expected)");
            // Give it a moment to process
            tokio::time::sleep(Duration::from_millis(100)).await;
            continue;
        }

        // Read response
        let response_result = timeout(Duration::from_secs(5), async {
            let mut reader = stdout_reader.lock().await;
            loop {
                match reader.next() {
                    Some(Ok(line)) => {
                        if line.trim().is_empty() {
                            continue;
                        }
                        if verbose {
                            info!("Raw response: {}", line);
                        }
                        return serde_json::from_str::<JsonRpcResponse>(&line)
                            .context("Failed to parse response");
                    }
                    Some(Err(e)) => return Err(anyhow::anyhow!("Read error: {}", e)),
                    None => return Err(anyhow::anyhow!("Server closed connection")),
                }
            }
        })
        .await;

        match response_result {
            Ok(Ok(response)) => {
                if let Some(error) = response.error {
                    error!("❌ Error response: {:?}", error);
                } else if let Some(result) = response.result {
                    info!("✅ Success!");
                    if verbose {
                        info!("Result: {}", serde_json::to_string_pretty(&result)?);
                    }

                    // Special handling for tools/list to show available tools
                    if request.method == "tools/list" {
                        if let Some(tools) = result.get("tools").and_then(|t| t.as_array()) {
                            info!("Available tools:");
                            for tool in tools {
                                if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                                    info!("  - {}", name);
                                }
                            }
                        }
                    }
                } else {
                    info!("✅ Empty success response");
                }
            }
            Ok(Err(e)) => {
                error!("❌ Failed to parse response: {}", e);
            }
            Err(_) => {
                error!("❌ Timeout waiting for response");
            }
        }
    }

    // Cleanup
    drop(stdin);
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Check if process exited cleanly
    match timeout(Duration::from_secs(2), async { cmd.wait() }).await {
        Ok(Ok(status)) => {
            if !status.success() {
                error!("⚠️  Server exited with status: {}", status);
                let stderr_lines = stderr_handle.await?;
                if !stderr_lines.is_empty() {
                    error!("Server stderr output:");
                    for line in stderr_lines.iter().take(10) {
                        error!("  {}", line);
                    }
                }
            } else {
                info!("✅ Server exited cleanly");
            }
        }
        Ok(Err(e)) => error!("Failed to get exit status: {}", e),
        Err(_) => {
            info!("Server still running, killing...");
            let _ = cmd.kill();
        }
    }

    Ok(())
}

async fn mock_server(verbose: bool) -> Result<()> {
    info!("🎭 Starting mock MCP server on stdio");
    info!("This server responds correctly to help debug client issues");

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let reader = BufReader::new(stdin);

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        if verbose {
            eprintln!("[MOCK] Received: {}", line);
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                let response_str = serde_json::to_string(&error_response)?;
                writeln!(stdout, "{}", response_str)?;
                stdout.flush()?;
                continue;
            }
        };

        let response = match request.method.as_str() {
            "initialize" => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "mock-mcp-server",
                        "version": "0.1.0"
                    }
                })),
                error: None,
            },
            "initialized" => {
                // This is a notification, no response needed
                if verbose {
                    eprintln!("[MOCK] Received initialized notification");
                }
                continue;
            }
            "tools/list" => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(json!({
                    "tools": [
                        {
                            "name": "mock_tool",
                            "description": "A mock tool for testing",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "message": {
                                        "type": "string"
                                    }
                                }
                            }
                        }
                    ]
                })),
                error: None,
            },
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                }),
            },
        };

        let response_str = serde_json::to_string(&response)?;
        if verbose {
            eprintln!("[MOCK] Sending: {}", response_str);
        }
        writeln!(stdout, "{}", response_str)?;
        stdout.flush()?;
    }

    Ok(())
}

async fn test_handshake(server_path: &str) -> Result<()> {
    info!("🤝 Testing MCP handshake with {}", server_path);

    // Different initialization patterns to test
    let init_patterns = vec![
        (
            "Standard init",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }),
        ),
        (
            "Minimal init",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {}
            }),
        ),
        (
            "With tools capability",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                }
            }),
        ),
    ];

    for (pattern_name, params) in init_patterns {
        info!("\n🧪 Testing pattern: {}", pattern_name);

        let mut cmd = Command::new(server_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdin = cmd.stdin.take().unwrap();
        let stdout = BufReader::new(cmd.stdout.take().unwrap());

        // Send initialize
        let init_req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "initialize".to_string(),
            params: Some(params),
        };

        writeln!(stdin, "{}", serde_json::to_string(&init_req)?)?;
        stdin.flush()?;

        // Read response
        let mut found_response = false;
        for line in stdout.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<JsonRpcResponse>(&line) {
                Ok(response) => {
                    if response.error.is_some() {
                        error!("❌ Error: {:?}", response.error);
                    } else {
                        info!("✅ Success: {}", pattern_name);
                        if let Some(result) = response.result {
                            if let Some(protocol_version) = result.get("protocolVersion") {
                                info!("  Protocol version: {}", protocol_version);
                            }
                            if let Some(server_info) = result.get("serverInfo") {
                                info!("  Server: {}", serde_json::to_string(server_info)?);
                            }
                        }
                    }
                    found_response = true;
                    break;
                }
                Err(e) => {
                    error!("❌ Failed to parse response: {}", e);
                    error!("  Raw: {}", line);
                }
            }
        }

        if !found_response {
            error!("❌ No response received for: {}", pattern_name);
        }

        // Kill the process
        let _ = cmd.kill();
    }

    Ok(())
}

async fn send_raw_request(
    server_path: &str,
    method: &str,
    params_str: &str,
    id: u64,
) -> Result<()> {
    info!("📡 Sending raw request to {}", server_path);

    let params: Value =
        serde_json::from_str(params_str).context("Failed to parse params as JSON")?;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(id)),
        method: method.to_string(),
        params: Some(params),
    };

    info!("Request: {}", serde_json::to_string_pretty(&request)?);

    let mut cmd = Command::new(server_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stdin = cmd.stdin.take().unwrap();
    let stdout = BufReader::new(cmd.stdout.take().unwrap());
    let stderr = BufReader::new(cmd.stderr.take().unwrap());

    // Spawn stderr reader
    tokio::spawn(async move {
        for line in stderr.lines().map_while(Result::ok) {
            eprintln!("[STDERR] {}", line);
        }
    });

    // Send request
    writeln!(stdin, "{}", serde_json::to_string(&request)?)?;
    stdin.flush()?;

    // Read response
    let response_future = async {
        for line in stdout.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            info!("Raw response: {}", line);

            match serde_json::from_str::<JsonRpcResponse>(&line) {
                Ok(response) => {
                    info!("\nParsed response:");
                    info!("{}", serde_json::to_string_pretty(&response)?);
                    return Ok(());
                }
                Err(e) => {
                    error!("Failed to parse as JSON-RPC: {}", e);
                    return Err(e.into());
                }
            }
        }
        Err(anyhow::anyhow!("No response received"))
    };

    match timeout(Duration::from_secs(5), response_future).await {
        Ok(result) => result?,
        Err(_) => error!("Timeout waiting for response"),
    }

    let _ = cmd.kill();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = Args::parse();

    match args.command {
        Commands::TestServer { server, verbose } => {
            test_server(&server, verbose).await?;
        }
        Commands::MockServer { verbose } => {
            mock_server(verbose).await?;
        }
        Commands::Raw {
            server,
            method,
            params,
            id,
        } => {
            send_raw_request(&server, &method, &params, id).await?;
        }
        Commands::Handshake { server } => {
            test_handshake(&server).await?;
        }
    }

    Ok(())
}
