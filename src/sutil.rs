use std::{
    fs::{self},
    io,
    path::Path,
};

pub fn print_num_lines() {
    let mut root = match fs::read_dir("./") {
        Ok(r) => r,
        Err(e) => {
            println!("{e}");
            return;
        }
    };
    let src = match root.find(|e| e.as_ref().is_ok_and(|e| e.file_name() == "src")) {
        Some(Ok(s)) => s,
        _ => {
            println!("Unable to find directory 'src'");
            return;
        }
    };
    match get_num_lines(&src.path()) {
        Ok(sum) => {
            println!("Number of lines: {sum}");
        }
        Err(e) => {
            println!("{e}");
        }
    }
}

fn get_num_lines(path: &Path) -> Result<i32, io::Error> {
    if path.is_file() {
        if count_path(path) {
            Ok(fs::read_to_string(path)?.lines().count() as i32)
        } else {
            Ok(0)
        }
    } else if path.is_dir() {
        let mut sum: i32 = 0;
        for entry in fs::read_dir(path)? {
            sum += get_num_lines(&entry?.path())?;
        }
        Ok(sum)
    } else {
        Ok(0)
    }
}

fn count_path(path: &Path) -> bool {
    match path.file_name() {
        Some(n) => {
            if let Some(name) = n.to_str() {
                name.ends_with(".rs")
            } else {
                false
            }
        }
        _ => false,
    }
}

pub fn split_into_lines<'a>(text: &'a str, line_width: usize) -> Vec<&'a str> {
    struct StrPos {
        char: usize,
        byte: usize,
    }
    let mut lines: Vec<&'a str> = Vec::new();
    let mut last_space = StrPos { char: 0, byte: 0 };
    let mut line_start = StrPos { char: 0, byte: 0 };
    let mut char: usize = 0;
    for (byte, c) in text.char_indices() {
        if c == ' ' {
            last_space = StrPos { char, byte };
        }
        if char - line_start.char == line_width {
            if last_space.char > line_start.char {
                // gentle split at last space
                lines.push(&text[line_start.byte..last_space.byte]);
                line_start = StrPos {
                    char: last_space.char + 1,
                    byte: last_space.byte + 1,
                };
            } else {
                // hard split in the middle of the word
                lines.push(&text[line_start.byte..byte]);
                line_start = StrPos { char, byte };
            }
        }
        char += 1;
    }
    lines.push(&text[line_start.byte..]);
    lines
}
