use std::path::PathBuf;

use shallows_vm::{
    cursor::Cursor,
    lexer::{Lexer, TokenKind},
    line_map::Lines,
};
use tracing::info;

fn main() {
    tracing_subscriber::fmt::init();

    let path = PathBuf::from("showcase.ss");
    let file_len = std::fs::metadata(&path)
        .expect("Failed to get file metadata")
        .len();
    info!("Lexing {} of size {}", path.to_string_lossy(), file_len);
    let lines = Lines::from_path(path).expect("Failed to read lines from file");
    let cursor = Cursor::new(&lines);
    let mut lexer = Lexer::new(cursor);

    let now = std::time::Instant::now();
    loop {
        let token = lexer.next_token();
        // if let TokenKind::Error(msg) = &token.kind {
        //     error!("Lexer error: {} at {:?}", msg, token.span);
        // } else {
        //     info!("{:?}", token)
        // }
        if token.kind == TokenKind::Eof {
            break;
        }
    }
    let time = now.elapsed();
    info!("Lexing completed in {:?}", time);
    info!("Speed: {} bytes/sec", file_len as f64 / time.as_secs_f64());
}
