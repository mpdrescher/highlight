extern crate term;
extern crate regex;

use term::{
    color,
    Terminal,
    Attr
};

use regex::Regex;

use std::env;
use std::io::{
    self,
    Read,
    Result as IOResult,
    Stdout
};
use std::marker::Send;
use std::fs::File;
use std::boxed::Box;

fn main() {
    let mut args = env::args().skip(1).rev().collect::<Vec<String>>();
    if args.len() < 1 {
        print_help();
        return;
    }
    let regex_str = args.pop().unwrap();
    let regex = match Regex::new(&regex_str){
        Ok(v) => v,
        Err(e) => {
            println!("Regex error: {}.", e);
            return;
        }
    };
    if args.len() == 0 {
        match print_from_stdin(regex) {
            Ok(_) => {},
            Err(e) => {
                println!("Error while printing from stdin: {}.", e);
            }
        }
        return;
    }
    match print_from_file(regex, args.into_iter().rev().collect::<Vec<String>>()) {
        Ok(_) => {},
        Err(e) => {
            println!("Error while printing: {}", e);
        }
    };
}

fn print_from_file(regex: Regex, files: Vec<String>) -> IOResult<()> {
    let mut term = term::stdout().expect("failed to unwrap stdout");
    println!();
    for file_path in files {
        print_filepath(&mut term, file_path.clone())?;
        let file = match load_file(file_path.clone()) {
            Ok(v) => v,
            Err(e) => {
                writeln!(term, "File error: {}.\n", e)?;
                continue;
            }
        };
        for line in file.split('\n') {
            print_line(&mut term, &regex, line.to_owned())?;
        }
        println!();
    }
    Ok(())
}

fn print_from_stdin(regex: Regex) -> IOResult<()> {
    let mut term = term::stdout().expect("failed to unwrap stdout");
    let stdin = io::stdin();
    let mut buffer = String::new();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut buffer)?;
    for line in buffer.split('\n') {
        print_line(&mut term, &regex, line.to_owned())?;
    }
    Ok(())
}

type StdoutTerminal = Box<Terminal<Output=Stdout> + Send>;

fn print_filepath(term: &mut StdoutTerminal, filename: String) -> IOResult<()> {
    write!(term, "  [")?;
    term.fg(color::CYAN)?;
    write!(term, "{}", filename)?;
    term.reset()?;
    writeln!(term, "]\n")?;
    Ok(())
}

fn print_line(term: &mut StdoutTerminal, regex: &Regex, line: String) -> IOResult<()> {
    let mut highlights = Vec::new();
    for capture_group in regex.captures_iter(&line) {
        let capture = capture_group.get(0).unwrap(); //should always be present
        highlights.push(Region::new(capture.start(), capture.end()));
    }
    for i_ch in line.chars().enumerate() {
        match i_ch {
            (index, ch) => {
                if highlights.iter().any(|x| x.contains(index)) {
                    term.attr(Attr::Reverse)?;
                    term.fg(color::GREEN)?;
                }
                else {
                    term.reset()?;
                }
                write!(term, "{}", ch)?;
            }
        };
    }
    writeln!(term, "")?;
    term.reset()?;
    Ok(())
}

fn load_file(path: String) -> IOResult<String> {
    let mut buffer = String::new();
    let mut file = File::open(path)?;
    let _ = file.read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn print_help() {
    println!();
    println!("USAGE: highlight <regex> [files]...");
    println!("If no files are provided, the input will be read from stdin.");
    println!();
}

struct Region {
    start: usize,
    end: usize
}

impl Region {
    pub fn new(start: usize, end: usize) -> Region {
        Region {
            start: start,
            end: end
        }
    }

    pub fn contains(&self, index: usize) -> bool {
        self.start <= index && index < self.end
    }
}
