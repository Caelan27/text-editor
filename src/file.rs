use std::fs;
use std::io;

pub fn load_file(path: &str) -> io::Result<String> {
    let contents = fs::read_to_string(path)?;
    Ok(contents)
}

pub fn save_file(path: &str, content: String) -> io::Result<()> {
    fs::write(path, content)?;
    Ok(())
}
