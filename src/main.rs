use include_dir::{Dir, include_dir};
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

static GITHUB_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/.github");

fn copy_github_from_embedded(wrapper_path: &PathBuf) -> io::Result<()> {
    let dst_github = wrapper_path.join(".github");
    GITHUB_DIR.extract(&dst_github)?;
    Ok(())
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

fn wrap_existing(source: &str) -> io::Result<()> {
    let source_path = PathBuf::from(source);

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
    copy_github_from_embedded(&wrapper_path)?;

    println!("✅ Wrapped project at: {}", wrapper_path.display());
    Ok(())
}

fn create_empty(name: &str) -> io::Result<()> {
    let wrapper_path = PathBuf::from(format!("wrapped-{}", name));

    if wrapper_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Wrapper already exists",
        ));
    }

    fs::create_dir_all(&wrapper_path)?;
    copy_github_from_embedded(&wrapper_path)?;

    println!("✅ Created wrapper at: {}", wrapper_path.display());
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: wraptor <folder-name>");
        eprintln!("       wraptor <source-path> (if exists, will wrap existing project)");
        std::process::exit(1);
    }

    let input = &args[1];
    let input_path = PathBuf::from(input);

    if input_path.exists() {
        wrap_existing(input)?;
    } else {
        create_empty(input)?;
    }

    Ok(())
}
