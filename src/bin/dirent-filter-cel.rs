use std::io;
use std::io::{BufRead, Write};
use std::path::PathBuf;

use cel::{Context, Program, Value};
use clap::Parser;

use rs_dirent_filter_cel::{DirentInfo, parser2ctx}; // Import DirentInfo from lib.rs

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The CEL expression to filter directory entries
    #[arg(long)]
    expr: String,

    /// The name of the variable for the directory entry in the CEL expression
    #[arg(long, default_value = "item")]
    name: String,
}

fn main() {
    let cli = Cli::parse();
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    if let Err(e) = run(cli, &mut stdin_lock, &mut stdout_lock) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run<R: BufRead, W: Write>(cli: Cli, reader: &mut R, writer: &mut W) -> Result<(), io::Error> {
    let program = compile_cel_program(&cli.expr)?;

    let mut ctx = Context::default();
    parser2ctx(&mut ctx, "parseSize");

    let mut buffer = String::new();

    while reader.read_line(&mut buffer)? > 0 {
        let entry_name = buffer.trim_end();
        if entry_name.is_empty() {
            buffer.clear();
            continue;
        }

        let path = PathBuf::from(entry_name);
        let dirent_info = DirentInfo::from(path.as_path());
        let dirent_cel_value: cel::Value = dirent_info.into();

        ctx.add_variable(&cli.name, dirent_cel_value).map_err(|e| {
            io::Error::other(format!("Error adding variable to CEL context: {}", e))
        })?;

        // Execute the program
        let result_value = program
            .execute(&ctx)
            .map_err(|e| io::Error::other(format!("CEL execution error: {}", e)))?;

        // Check the boolean result
        if let Value::Bool(b) = result_value {
            if b {
                // Manually add the newline because trim_end removed it
                writeln!(writer, "{}", entry_name)?;
            }
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "CEL expression did not return a boolean result.",
            ));
        }
        buffer.clear();
    }

    Ok(())
}

fn compile_cel_program(expr: &str) -> Result<Program, io::Error> {
    Program::compile(expr).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("CEL compilation error: {}", e),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::BufReader;
    use tempfile::tempdir;

    #[test]
    fn test_run_filter_by_size_with_parse_size() {
        let dir = tempdir().unwrap();
        let file_path_small = dir.path().join("small.txt");
        fs::File::create(&file_path_small)
            .unwrap()
            .write_all(&[0; 10 * 1024]) // 10 KiB
            .unwrap();
        let file_path_large = dir.path().join("large.txt");
        fs::File::create(&file_path_large)
            .unwrap()
            .write_all(&[0; 10 * 1024 * 1024]) // 10 MiB
            .unwrap();

        let cli = Cli {
            expr: "item.len > parseSize('5MiB')".to_string(),
            name: "item".to_string(),
        };

        let input_str = format!(
            "{}
{}
",
            file_path_small.to_str().unwrap(),
            file_path_large.to_str().unwrap(),
        );
        let mut reader = BufReader::new(input_str.as_bytes());
        let mut output = Vec::new();

        let result = run(cli, &mut reader, &mut output);
        assert!(result.is_ok());

        let expected_output = format!("{}\n", file_path_large.to_str().unwrap());
        assert_eq!(String::from_utf8(output).unwrap(), expected_output);
    }
}
