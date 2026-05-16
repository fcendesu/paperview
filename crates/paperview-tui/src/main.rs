use paperview_core::Document;

fn main() {
    let document = Document::from_source("# PaperView");
    println!("PaperView TUI shell ready: {}", document.title());
}
