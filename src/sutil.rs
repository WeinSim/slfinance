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
