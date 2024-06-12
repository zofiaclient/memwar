use std::io;

use memwar::library::Library;

fn main() -> io::Result<()> {
    let library = Library::from(
        "Notepad",
        "Example library for Notepad",
        "Imajin Devon",
        "notepad.dll",
    );
    tora::write_to_file("notepad.mw", &library)
}
