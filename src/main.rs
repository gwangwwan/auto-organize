use clap::Parser;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// A CLI tool to automatically organize files into folders by type.
///
/// Moves unknown files to 'Others', apps to 'APPS', and loose folders to 'Folders'.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The directory to organize (defaults to current directory)
    path: Option<PathBuf>,

    /// Dry run: preview changes without moving files
    #[arg(short, long, default_value_t = false)]
    dry_run: bool,
}

fn main() {
    let args = Args::parse();
    let target_dir = args.path.unwrap_or_else(|| PathBuf::from("."));

    if !target_dir.is_dir() {
        eprintln!(
            "Error: '{}' is not a valid directory.",
            target_dir.display()
        );
        std::process::exit(1);
    }

    println!(
        "Target: {}",
        target_dir
            .canonicalize()
            .unwrap_or(target_dir.clone())
            .display()
    );
    if args.dry_run {
        println!("Mode:   DRY RUN (No changes will be made)");
    }
    println!("-----------------------------------------");

    // 1. Setup extension map and protected folder names
    let extension_map = get_extension_map();

    // These folders will NOT be moved if they already exist
    let protected_folders = get_protected_folder_names();

    // 2. Read directory
    let entries = match fs::read_dir(&target_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error reading directory: {}", e);
            std::process::exit(1);
        }
    };

    let mut files_count = 0;
    let mut dirs_count = 0;

    for entry in entries.flatten() {
        let path = entry.path();

        // --- Handle Directories ---
        if path.is_dir() {
            // Get the folder name (e.g., "images" from "/Downloads/images")
            if let Some(folder_name) = path.file_name().and_then(|n| n.to_str()) {
                // If the folder is one of our categories, SKIP it.
                if protected_folders.contains(folder_name) {
                    continue;
                }

                // Otherwise, it's a loose folder. Move it to "Folders"
                if process_directory(&path, &target_dir, "Folders", args.dry_run) {
                    dirs_count += 1;
                }
            }
            continue;
        }

        // --- Handle Files ---
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        // Check if extension is known
        let category = match extension_map.get(&ext) {
            Some(cat) => cat.clone(),     // Known category (images, apps, etc.)
            None => "Others".to_string(), // Unknown extension (ini, sw, meme) -> Others
        };

        if process_file(&path, &target_dir, &category, args.dry_run) {
            files_count += 1;
        }
    }

    println!("-----------------------------------------");
    println!(
        "Done. {} files and {} folders processed.",
        files_count, dirs_count
    );
}

/// Moves a file to a category folder
fn process_file(file_path: &Path, base_dir: &Path, category: &str, dry_run: bool) -> bool {
    let category_dir = base_dir.join(category);

    if !dry_run && !category_dir.exists() {
        if let Err(e) = fs::create_dir_all(&category_dir) {
            eprintln!("Error creating dir: {}", e);
            return false;
        }
    }

    let file_name = file_path.file_name().unwrap_or_default();
    let dest_path = category_dir.join(file_name);

    if dest_path.exists() {
        println!("[SKIP] {:?} (already exists in {})", file_name, category);
        return false;
    }

    println!("[{:<12}] {:?}", category, file_name);

    if !dry_run {
        if let Err(e) = fs::rename(file_path, &dest_path) {
            eprintln!("Error moving {:?}: {}", file_name, e);
            return false;
        }
    }
    true
}

/// Moves a directory into a parent folder (e.g., "Folders")
fn process_directory(
    dir_path: &Path,
    base_dir: &Path,
    dest_container: &str,
    dry_run: bool,
) -> bool {
    let container_dir = base_dir.join(dest_container);

    if !dry_run && !container_dir.exists() {
        if let Err(e) = fs::create_dir_all(&container_dir) {
            eprintln!("Error creating container dir: {}", e);
            return false;
        }
    }

    let dir_name = dir_path.file_name().unwrap_or_default();
    let dest_path = container_dir.join(dir_name);

    // Safety check: ensure we aren't trying to move the container into itself
    if dir_path == container_dir {
        return false;
    }

    if dest_path.exists() {
        println!(
            "[SKIP DIR] {:?} (already exists in {})",
            dir_name, dest_container
        );
        return false;
    }

    println!("[{:<12}] (Directory) {:?}", dest_container, dir_name);

    if !dry_run {
        if let Err(e) = fs::rename(dir_path, &dest_path) {
            eprintln!("Error moving directory {:?}: {}", dir_name, e);
            return false;
        }
    }
    true
}

/// Returns a set of folder names that should not be moved
fn get_protected_folder_names() -> HashSet<String> {
    let mut set = HashSet::new();
    set.insert("images".to_string());
    set.insert("documents".to_string());
    set.insert("spreadsheets".to_string());
    set.insert("presentations".to_string());
    set.insert("archives".to_string());
    set.insert("audio".to_string());
    set.insert("video".to_string());
    set.insert("code".to_string());
    set.insert("APPS".to_string()); // New category
    set.insert("Others".to_string()); // Catch-all for files
    set.insert("Folders".to_string()); // Catch-all for directories
    set
}

fn get_extension_map() -> HashMap<String, String> {
    let mut map = HashMap::new();

    let categories = [
        (
            "images",
            vec![
                "jpg", "jpeg", "png", "gif", "bmp", "svg", "webp", "ico", "tiff", "heic",
            ],
        ),
        (
            "documents",
            vec!["pdf", "doc", "docx", "txt", "rtf", "odt", "md"],
        ),
        ("spreadsheets", vec!["xls", "xlsx", "csv", "ods"]),
        ("presentations", vec!["ppt", "pptx", "odp", "key"]),
        (
            "archives",
            vec!["zip", "rar", "tar", "gz", "bz2", "7z", "iso"],
        ),
        ("audio", vec!["mp3", "wav", "flac", "aac", "ogg", "m4a"]),
        ("video", vec!["mp4", "mkv", "avi", "mov", "wmv", "webm"]),
        (
            "code",
            vec![
                "rs", "py", "js", "ts", "java", "c", "cpp", "go", "rb", "php", "html", "css",
                "json",
            ],
        ),
        // New "APPS" category for executables
        (
            "APPS",
            vec![
                "exe", "msi", "dmg", "app", "deb", "rpm", "apk", "appimage", "sh", "bat",
            ],
        ),
    ];

    for (category, extensions) in categories {
        for ext in extensions {
            map.insert(ext.to_string(), category.to_string());
        }
    }

    map
}
