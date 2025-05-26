use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::timeout;

#[derive(Parser, Debug)]
#[command(author, version, about = "MCP Test Client", long_about = None)]
struct Args {
    /// Path to the MCP server binary
    #[arg(short, long)]
    server: String,

    /// Enable debug output
    #[arg(long)]
    debug: bool,

    /// Timeout in seconds for each request
    #[arg(short, long, default_value = "5")]
    timeout: u64,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Test the full initialization sequence
    Init,
    /// Test listing tools
    ListTools,
    /// Test calling a specific tool
    CallTool {
        #[arg(short, long)]
        tool: String,
        #[arg(short, long)]
        params: String,
    },
    /// Run a full lifecycle test
    Lifecycle,
    /// Send a custom request
    Custom {
        #[arg(short, long)]
        method: String,
        #[arg(short, long)]
        params: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct McpRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct McpResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
    id: Option<u64>,
}

struct McpTestClient {
    server_path: String,
    debug: bool,
    timeout_secs: u64,
}

impl McpTestClient {
    fn new(server_path: String, debug: bool, timeout_secs: u64) -> Self {
        Self {
            server_path,
            debug,
            timeout_secs,
        }
    }

    async fn run_test(&self, requests: Vec<McpRequest>) -> Result<Vec<McpResponse>> {
        let mut cmd = Command::new(&self.server_path)
            .args(if self.debug { vec!["--debug"] } else { vec![] })
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn MCP server")?;

        let mut stdin = cmd.stdin.take().context("Failed to get stdin")?;
        let stdout = cmd.stdout.take().context("Failed to get stdout")?;
        let stderr = cmd.stderr.take().context("Failed to get stderr")?;

        // Spawn a task to read stderr
        let debug = self.debug;
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if debug {
                        eprintln!("[SERVER STDERR] {}", line);
                    }
                }
            }
        });

        let mut responses = Vec::new();
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        for request in requests {
            if self.debug {
                eprintln!("[CLIENT] Sending: {}", serde_json::to_string(&request)?);
            }

            // Send request
            let request_str = serde_json::to_string(&request)?;
            writeln!(stdin, "{}", request_str)?;
            stdin.flush()?;

            // Read response with timeout
            let response_fut = async {
                loop {
                    if let Some(Ok(line)) = lines.next() {
                        if line.trim().is_empty() {
                            continue;
                        }
                        if self.debug {
                            eprintln!("[SERVER] Received: {}", line);
                        }
                        return serde_json::from_str::<McpResponse>(&line)
                            .context("Failed to parse response");
                    } else {
                        return Err(anyhow::anyhow!("Server closed connection"));
                    }
                }
            };

            let response = timeout(Duration::from_secs(self.timeout_secs), response_fut)
                .await
                .context("Timeout waiting for response")??;

            responses.push(response);
        }

        // Try to gracefully shutdown
        drop(stdin);

        // Wait a bit for the server to exit
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Check exit status
        match timeout(Duration::from_secs(2), async {
            cmd.wait().context("Failed to wait for server")
        })
        .await
        {
            Ok(Ok(status)) => {
                if !status.success() {
                    eprintln!("[WARNING] Server exited with status: {}", status);
                }
            }
            Ok(Err(e)) => eprintln!("[ERROR] Failed to get exit status: {}", e),
            Err(_) => {
                eprintln!("[WARNING] Server did not exit within timeout, killing...");
                let _ = cmd.kill();
            }
        }

        Ok(responses)
    }

    async fn test_init(&self) -> Result<()> {
        println!("🧪 Testing initialization sequence...");

        let requests = vec![
            McpRequest {
                jsonrpc: "2.0".to_string(),
                method: "initialize".to_string(),
                params: json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {}
                }),
                id: 1,
            },
            McpRequest {
                jsonrpc: "2.0".to_string(),
                method: "initialized".to_string(),
                params: json!({}),
                id: 2,
            },
        ];

        let responses = self.run_test(requests).await?;

        // Check responses
        for (i, response) in responses.iter().enumerate() {
            println!(
                "\n📥 Response {}: {}",
                i + 1,
                serde_json::to_string_pretty(&response)?
            );

            if let Some(error) = &response.error {
                return Err(anyhow::anyhow!("Error in response: {:?}", error));
            }
        }

        println!("\n✅ Initialization test passed!");
        Ok(())
    }

    async fn test_list_tools(&self) -> Result<()> {
        println!("🧪 Testing tools/list...");

        let requests = vec![
            McpRequest {
                jsonrpc: "2.0".to_string(),
                method: "initialize".to_string(),
                params: json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {}
                }),
                id: 1,
            },
            McpRequest {
                jsonrpc: "2.0".to_string(),
                method: "initialized".to_string(),
                params: json!({}),
                id: 2,
            },
            McpRequest {
                jsonrpc: "2.0".to_string(),
                method: "tools/list".to_string(),
                params: json!({}),
                id: 3,
            },
        ];

        let responses = self.run_test(requests).await?;

        if let Some(result) = &responses.last().unwrap().result {
            println!("\n📥 Available tools:");
            if let Some(tools) = result.get("tools").and_then(|t| t.as_array()) {
                for tool in tools {
                    if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                        println!("  - {}", name);
                    }
                }
            }
        }

        println!("\n✅ List tools test passed!");
        Ok(())
    }

    async fn test_lifecycle(&self) -> Result<()> {
        println!("🧪 Testing full lifecycle...");

        let requests = vec![
            McpRequest {
                jsonrpc: "2.0".to_string(),
                method: "initialize".to_string(),
                params: json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {}
                }),
                id: 1,
            },
            McpRequest {
                jsonrpc: "2.0".to_string(),
                method: "initialized".to_string(),
                params: json!({}),
                id: 2,
            },
            McpRequest {
                jsonrpc: "2.0".to_string(),
                method: "tools/list".to_string(),
                params: json!({}),
                id: 3,
            },
            McpRequest {
                jsonrpc: "2.0".to_string(),
                method: "tools/call".to_string(),
                params: json!({
                    "name": "list_docs",
                    "arguments": {}
                }),
                id: 4,
            },
        ];

        let responses = self.run_test(requests).await?;

        for (i, response) in responses.iter().enumerate() {
            println!(
                "\n📥 Step {}: {}",
                i + 1,
                if response.error.is_none() {
                    "✅ Success"
                } else {
                    "❌ Error"
                }
            );

            if self.debug {
                println!("{}", serde_json::to_string_pretty(&response)?);
            }
        }

        println!("\n✅ Lifecycle test completed!");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(if args.debug {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .init();

    let client = McpTestClient::new(args.server, args.debug, args.timeout);

    match args.command {
        Commands::Init => client.test_init().await?,
        Commands::ListTools => client.test_list_tools().await?,
        Commands::CallTool { tool, params } => {
            todo!("Implement tool call test");
        }
        Commands::Lifecycle => client.test_lifecycle().await?,
        Commands::Custom { method, params } => {
            todo!("Implement custom request");
        }
    }

    Ok(())
}
