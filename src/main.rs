use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

fn read_input(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();

    loop {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            use crossterm::event::KeyModifiers;

            if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
                return Err(io::Error::new(io::ErrorKind::Interrupted, "User cancelled"));
            }

            match code {
                KeyCode::Enter => break,
                KeyCode::Char(c) => {
                    input.push(c);
                    print!("{}", c);
                    io::stdout().flush()?;
                }
                KeyCode::Backspace if !input.is_empty() => {
                    input.pop();
                    print!("\x08 \x08");
                    io::stdout().flush()?;
                }
                _ => {}
            }
        }
    }
    println!();
    Ok(input)
}

fn copy_github_folder(wrapper_path: &PathBuf) -> io::Result<()> {
    let current_dir = env::current_dir()?;
    let src_github = current_dir.join(".github");
    let dst_github = wrapper_path.join(".github");

    if !src_github.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            ".github folder not found",
        ));
    }

    copy_dir(&src_github, &dst_github)
}

fn copy_dir(src: &PathBuf, dst: &PathBuf) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn wrap_existing() -> io::Result<()> {
    let source = read_input("Source project path: ")?;
    let source_path = PathBuf::from(source.trim());

    if !source_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Source path not found",
        ));
    }

    let project_name = source_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");
    let wrapper_path = PathBuf::from(format!("wrapped-{}", project_name));

    if wrapper_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Wrapper already exists",
        ));
    }

    copy_dir(&source_path, &wrapper_path)?;
    copy_github_folder(&wrapper_path)?;

    println!("✅ Wrapped project at: {}", wrapper_path.display());
    Ok(())
}

fn create_empty() -> io::Result<()> {
    let name = read_input("Project name: ")?;
    let wrapper_path = PathBuf::from(format!("wrapped-{}", name.trim()));

    if wrapper_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Wrapper already exists",
        ));
    }

    fs::create_dir_all(&wrapper_path)?;
    copy_github_folder(&wrapper_path)?;

    println!("✅ Created wrapper at: {}", wrapper_path.display());
    Ok(())
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), Clear(ClearType::All))?;

    println!("🦖 WRAPTOR\r");
    println!("\r");
    println!("1. Wrap existing project\r");
    println!("2. Create empty wrapper\r");
    println!("\r");

    let choice = read_input("Select (1/2): ")?;

    let result = match choice.trim() {
        "1" => wrap_existing(),
        "2" => create_empty(),
        _ => {
            disable_raw_mode()?;
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid choice",
            ));
        }
    };

    disable_raw_mode()?;
    result
}
