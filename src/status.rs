use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::scan;
use crate::Error;

pub fn run() -> Result<(), Error> {
    let specs_dir = Path::new("specs");
    if !specs_dir.exists() {
        eprintln!("No specs/ directory found. Run `specdev init` first.");
        return Ok(());
    }

    let core_files = ["overview.md", "ctx.md", "ideas.md"];
    let mut total_open = 0;
    let mut total_resolved = 0;

    println!("Core spec files:");
    for name in &core_files {
        let path = specs_dir.join(name);
        if path.exists() {
            let meta = fs::metadata(&path)?;
            let content = fs::read_to_string(&path)?;
            let lines = content.lines().count();
            let (open, resolved) = scan::count_markers(&content);
            total_open += open;
            total_resolved += resolved;
            let age = format_age(meta.modified()?);
            let markers = open + resolved;
            if markers > 0 {
                println!(
                    "  [OK] {:<14} {:>4} lines  {:<10} {} markers ({} open)",
                    name, lines, age, markers, open
                );
            } else {
                println!("  [OK] {:<14} {:>4} lines  {:<10}", name, lines, age);
            }
        } else {
            println!("  [--] {:<14} missing", name);
        }
    }

    let mut extra: Vec<String> = Vec::new();
    for entry in fs::read_dir(specs_dir)? {
        let entry = entry?;
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.ends_with(".md") && !core_files.contains(&fname.as_str()) {
            extra.push(fname);
        }
    }

    if !extra.is_empty() {
        extra.sort();
        println!("\nAdditional specs:");
        for name in &extra {
            let path = specs_dir.join(name);
            let content = fs::read_to_string(&path)?;
            let lines = content.lines().count();
            let (open, resolved) = scan::count_markers(&content);
            total_open += open;
            total_resolved += resolved;
            let markers = open + resolved;
            if markers > 0 {
                println!(
                    "  {:<15} {:>4} lines  {} markers ({} open)",
                    name, lines, markers, open
                );
            } else {
                println!("  {:<15} {:>4} lines", name, lines);
            }
        }
    }

    println!(
        "\nMarkers: {} open, {} resolved",
        total_open, total_resolved
    );

    Ok(())
}

fn format_age(modified: SystemTime) -> String {
    let now = SystemTime::now();
    match now.duration_since(modified) {
        Ok(dur) => {
            let secs = dur.as_secs();
            if secs < 60 {
                format!("{}s ago", secs)
            } else if secs < 3600 {
                format!("{}m ago", secs / 60)
            } else if secs < 86400 {
                format!("{}h ago", secs / 3600)
            } else {
                format!("{}d ago", secs / 86400)
            }
        }
        Err(_) => "future".to_string(),
    }
}
