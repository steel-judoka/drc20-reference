//! Small helper tooling for DRC20.
//!
//! Currently implemented commands:
//! - `init-args`: encode the constructor `Init` JSON into rkyv bytes and print hex.

use std::env;
use std::fs;
use std::io::{self, Read};

use drc20_types::Init;
use dusk_data_driver::json_to_rkyv;

fn main() {
    let mut args = env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| {
        eprintln!(
            "{}",
            r#"Usage:
  drc20-tools init-args [--file <path>]

Examples:
  echo '{"initial_balances":[{"account":{"External":"A7gM..."},"amount":"1000"}]}' | drc20-tools init-args
  drc20-tools init-args --file init.json"#
        );
        std::process::exit(2);
    });

    match cmd.as_str() {
        "init-args" => {
            let json = match args.next().as_deref() {
                Some("--file") => {
                    let path = args.next().unwrap_or_else(|| {
                        eprintln!("Missing value for --file");
                        std::process::exit(2);
                    });
                    fs::read_to_string(path).unwrap_or_else(|e| {
                        eprintln!("Failed to read file: {e}");
                        std::process::exit(1);
                    })
                }
                Some(other) => {
                    // Treat remaining args as a single JSON payload.
                    let mut rest = vec![other.to_string()];
                    rest.extend(args.map(|s| s));
                    rest.join(" ")
                }
                None => {
                    // Read JSON from stdin.
                    let mut buf = String::new();
                    io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
                        eprintln!("Failed to read stdin: {e}");
                        std::process::exit(1);
                    });
                    buf
                }
            };

            let json = json.trim();
            if json.is_empty() {
                eprintln!("Empty JSON input");
                std::process::exit(2);
            }

            let bytes = json_to_rkyv::<Init>(json).unwrap_or_else(|e| {
                eprintln!("Failed to encode Init JSON to rkyv: {e}");
                std::process::exit(1);
            });

            print!("{}", to_hex(&bytes));
        }
        _ => {
            eprintln!("Unknown command: {cmd}");
            std::process::exit(2);
        }
    }
}

fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}
