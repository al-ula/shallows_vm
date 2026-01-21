use std::path::PathBuf;

use shallows_vm::line_map::Lines;

fn main() {
    let path = PathBuf::from("showcase.ss");
    let lines = Lines::from_path(path).expect("Failed to read lines");
    for line in lines {
        println!("Line {}: {}", line.idx, line.content);
    }
}
