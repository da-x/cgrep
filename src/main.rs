#![warn(unused_extern_crates)]

use std::io::BufRead;
use isatty::stdout_isatty;
use structopt::StructOpt;

mod error;
use error::Error;

#[derive(StructOpt, Debug)]
struct Opt {
    regex: String,

    path: Option<String>,
}

struct Main {}

impl Main {
    fn new(opt: Opt) -> Result<(), Error> {
        let regex = regex::bytes::Regex::new(&opt.regex)?;
        if stdout_isatty() {
            grep_on_writer(&opt, &mut std::io::stdout(), regex)?;
        } else {
            grep_on_writer(&opt, std::io::BufWriter::with_capacity(0x400000, std::io::stdout()), regex)?;
        }

        Ok(())
    }
}

fn grep_on_writer<S>(params: &Opt, mut stdout: S, regex: regex::bytes::Regex) -> Result<(), Error>
    where S: std::io::Write
{
    Ok(if let Some(path) = &params.path {
        let file = std::io::BufReader::with_capacity(0x400000, std::fs::OpenOptions::new().read(true).open(&path)?);
        grep_on_reader(file, &mut stdout, regex)?;
    } else {
        let stdin = std::io::BufReader::with_capacity(0x400000, std::io::stdin());
        grep_on_reader(stdin, &mut stdout, regex)?;
    })
}

pub fn strip_ansi_escapes(in_buffer: &[u8], out_buffer: &mut Vec<u8>) {
    out_buffer.clear();

    let mut is_escape_sequence = false;
    let len = in_buffer.len();
    let mut i = 0;
    while i < len {
        if is_escape_sequence {
            if in_buffer[i] == b'm' {
                is_escape_sequence = false;
            }
        } else if in_buffer[i] == 0x1b && i + 1 < len && in_buffer[i + 1] == b'[' {
            is_escape_sequence = true;
            i += 1;  // Skip the '['
        } else {
            out_buffer.push(in_buffer[i]);
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main() {
        let mut v = Vec::new();
        strip_ansi_escapes("no ansi codes".as_bytes(), &mut v);
        assert!(String::from_utf8_lossy(&v) == "no ansi codes");

        strip_ansi_escapes("with \x1b[38;2;102;102;102mansi\x1b[0m codes".as_bytes(), &mut v);
        println!("{:?}", String::from_utf8_lossy(&v));
        assert!(String::from_utf8_lossy(&v) == "with ansi codes");
    }
}

fn grep_on_reader<T, S>(mut stdin: std::io::BufReader<T>, write: &mut S, regex: regex::bytes::Regex) -> Result<(), Error>
    where T: std::io::Read,
          S: std::io::Write
{
    let mut line_raw = String::new();
    let mut line_cooked = Vec::new();

    while let Ok(count) = stdin.read_line(&mut line_raw) {
        if count == 0 {
            break;
        }

        strip_ansi_escapes(&line_raw.as_bytes(), &mut line_cooked);

        if regex.is_match(&line_cooked) {
            write!(write, "{}", line_raw)?;
        }

        line_raw.clear();
    }

    Ok(())
}

fn main_wrap() -> Result<(), Error> {
    Main::new(Opt::from_args())?;

    Ok(())
}

fn main() {
    match main_wrap() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(-1);
        }
    }
}
