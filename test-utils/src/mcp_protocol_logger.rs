use anyhow::{Context, Result};
use clap::Parser;
use serde_json::Value;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tokio::process::Command as TokioCommand;

#[derive(Parser, Debug)]
#[command(author, version, about = "MCP Protocol Logger", long_about = None)]
struct Args {
    /// Path to the MCP server binary
    #[arg(short, long)]
    server: String,

    /// Log file path
    #[arg(short, long, default_value = "mcp-protocol.log")]
    log_file: String,

    /// Also print to stderr
    #[arg(short, long)]
    verbose: bool,

    /// Server arguments
    #[arg(trailing_var_arg = true)]
    server_args: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Open log file
    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&args.log_file)
        .context("Failed to open log file")?;

    writeln!(
        log_file,
        "\n=== MCP Protocol Log Started: {} ===",
        chrono::Local::now()
    )?;

    // Start the server
    let mut cmd = TokioCommand::new(&args.server)
        .args(&args.server_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn MCP server")?;

    let stdin_server = cmd.stdin.take().context("Failed to get server stdin")?;
    let stdout_server = cmd.stdout.take().context("Failed to get server stdout")?;
    let stderr_server = cmd.stderr.take().context("Failed to get server stderr")?;

    let request_counter = Arc::new(AtomicUsize::new(0));
    let response_counter = Arc::new(AtomicUsize::new(0));

    // Spawn stderr logger
    let log_file_clone = args.log_file.clone();
    let verbose = args.verbose;
    tokio::spawn(async move {
        let reader = AsyncBufReader::new(stderr_server);
        let mut lines = reader.lines();
        let mut log_file = OpenOptions::new()
            .append(true)
            .open(&log_file_clone)
            .expect("Failed to open log file");

        while let Ok(Some(line)) = lines.next_line().await {
            let _ = writeln!(log_file, "[STDERR] {}", line);
            if verbose {
                eprintln!("[STDERR] {}", line);
            }
        }
    });

    // Setup stdin relay
    let stdin = tokio::io::stdin();
    let mut stdin_reader = AsyncBufReader::new(stdin);
    let mut stdin_writer = stdin_server;

    let log_file_clone = args.log_file.clone();
    let verbose = args.verbose;
    let req_counter = request_counter.clone();
    let stdin_task = tokio::spawn(async move {
        let mut log_file = OpenOptions::new()
            .append(true)
            .open(&log_file_clone)
            .expect("Failed to open log file");

        let mut lines = stdin_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let req_num = req_counter.fetch_add(1, Ordering::SeqCst) + 1;

            // Log the request
            let _ = writeln!(
                log_file,
                "\n>>> REQUEST #{} @ {}",
                req_num,
                chrono::Local::now()
            );

            // Try to pretty-print JSON
            match serde_json::from_str::<Value>(&line) {
                Ok(json) => {
                    let pretty = serde_json::to_string_pretty(&json).unwrap();
                    let _ = writeln!(log_file, "{}", pretty);
                    if verbose {
                        eprintln!("\n>>> REQUEST #{}\n{}", req_num, pretty);
                    }
                }
                Err(_) => {
                    let _ = writeln!(log_file, "{}", line);
                    if verbose {
                        eprintln!("\n>>> REQUEST #{}\n{}", req_num, line);
                    }
                }
            }

            // Forward to server
            if let Err(e) = stdin_writer.write_all(line.as_bytes()).await {
                eprintln!("Failed to write to server: {}", e);
                break;
            }
            if let Err(e) = stdin_writer.write_all(b"\n").await {
                eprintln!("Failed to write newline to server: {}", e);
                break;
            }
            if let Err(e) = stdin_writer.flush().await {
                eprintln!("Failed to flush to server: {}", e);
                break;
            }
        }
    });

    // Setup stdout relay
    let stdout = tokio::io::stdout();
    let mut stdout_writer = stdout;
    let mut stdout_reader = AsyncBufReader::new(stdout_server);

    let log_file_clone = args.log_file.clone();
    let verbose = args.verbose;
    let resp_counter = response_counter.clone();
    let stdout_task = tokio::spawn(async move {
        let mut log_file = OpenOptions::new()
            .append(true)
            .open(&log_file_clone)
            .expect("Failed to open log file");

        let mut lines = stdout_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let resp_num = resp_counter.fetch_add(1, Ordering::SeqCst) + 1;

            // Log the response
            let _ = writeln!(
                log_file,
                "\n<<< RESPONSE #{} @ {}",
                resp_num,
                chrono::Local::now()
            );

            // Try to pretty-print JSON
            match serde_json::from_str::<Value>(&line) {
                Ok(json) => {
                    let pretty = serde_json::to_string_pretty(&json).unwrap();
                    let _ = writeln!(log_file, "{}", pretty);
                    if verbose {
                        eprintln!("\n<<< RESPONSE #{}\n{}", resp_num, pretty);
                    }
                }
                Err(_) => {
                    let _ = writeln!(log_file, "{}", line);
                    if verbose {
                        eprintln!("\n<<< RESPONSE #{}\n{}", resp_num, line);
                    }
                }
            }

            // Forward to stdout
            if let Err(e) = stdout_writer.write_all(line.as_bytes()).await {
                eprintln!("Failed to write to stdout: {}", e);
                break;
            }
            if let Err(e) = stdout_writer.write_all(b"\n").await {
                eprintln!("Failed to write newline to stdout: {}", e);
                break;
            }
            if let Err(e) = stdout_writer.flush().await {
                eprintln!("Failed to flush stdout: {}", e);
                break;
            }
        }
    });

    // Wait for tasks
    let _ = tokio::join!(stdin_task, stdout_task);

    // Wait for server to exit
    let status = cmd.wait().await?;

    let mut log_file = OpenOptions::new().append(true).open(&args.log_file)?;
    writeln!(log_file, "\n=== Server exited with status: {} ===", status)?;

    if verbose {
        eprintln!("\n=== Server exited with status: {} ===", status);
    }

    Ok(())
}
