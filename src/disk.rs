use colored::Colorize;
use std::path::{Path, PathBuf};

use crate::banner;
use crate::downloader::human_size;

#[cfg(windows)]
pub fn get_path_free_space(path: &Path) -> Option<u64> {
    use std::os::windows::ffi::OsStrExt;
    let root_str = path.components().next()?.as_os_str();
    let mut root_path = PathBuf::from(root_str);
    root_path.push("\\");
    let wide: Vec<u16> = root_path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();

    let mut free_bytes_available: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_free_bytes: u64 = 0;

    extern "system" {
        fn GetDiskFreeSpaceExW(
            lpDirectoryName: *const u16,
            lpFreeBytesAvailableToCaller: *mut u64,
            lpTotalNumberOfBytes: *mut u64,
            lpTotalNumberOfFreeBytes: *mut u64,
        ) -> i32;
    }

    unsafe {
        if GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut free_bytes_available,
            &mut total_bytes,
            &mut total_free_bytes,
        ) != 0
        {
            Some(free_bytes_available)
        } else {
            None
        }
    }
}

#[cfg(not(windows))]
pub fn get_path_free_space(_path: &Path) -> Option<u64> {
    None
}

#[cfg(windows)]
pub fn get_available_drives() -> Vec<(String, u64)> {
    let mut drives = Vec::new();
    for letter in b'A'..=b'Z' {
        let drive_str = format!("{}:\\", letter as char);
        let path = PathBuf::from(&drive_str);
        if let Some(free_bytes) = get_path_free_space(&path) {
            if free_bytes > 0 {
                drives.push((drive_str, free_bytes));
            }
        }
    }
    drives
}

#[cfg(not(windows))]
pub fn get_available_drives() -> Vec<(String, u64)> {
    Vec::new()
}

/// Checks if out_dir has enough free space for required_bytes.
/// If not, prompts the user to select another drive with enough space.
/// Returns the chosen output directory PathBuf.
pub fn ensure_sufficient_space(out_dir: &PathBuf, required_bytes: u64) -> PathBuf {
    let current_free = get_path_free_space(out_dir);

    if let Some(free) = current_free {
        // If current drive has enough space (plus 10MB margin), return out_dir as is
        if free >= required_bytes + 10 * 1024 * 1024 {
            return out_dir.clone();
        }

        let curr_drive = out_dir
            .components()
            .next()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .unwrap_or_else(|| "current drive".to_string());

        banner::print_warning(&format!(
            "Insufficient space on {} (Available: {}, Required: {})",
            curr_drive,
            human_size(free),
            human_size(required_bytes)
        ));

        let available_drives = get_available_drives();
        let suitable_drives: Vec<(String, u64)> = available_drives
            .into_iter()
            .filter(|(_, free_bytes)| *free_bytes >= required_bytes + 10 * 1024 * 1024)
            .collect();

        if suitable_drives.is_empty() {
            banner::print_error("No available drives have enough free space for this download!");
            return out_dir.clone();
        }

        println!();
        println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
        println!("  {}", "SELECT ALTERNATIVE DRIVE WITH AVAILABLE SPACE".yellow().bold());
        println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
        println!();

        for (i, (drive_letter, free_bytes)) in suitable_drives.iter().enumerate() {
            let rec = if i == 0 { " ✓ Recommended".yellow() } else { "".normal() };
            println!(
                "    {} {} ({} free){}",
                format!("{}", i + 1).cyan().bold(),
                drive_letter.green(),
                human_size(*free_bytes).dimmed(),
                rec
            );
        }
        println!("    {} Enter custom directory path", format!("{}", suitable_drives.len() + 1).cyan().bold());
        println!();
        println!("  {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
        println!();

        print!("  {} ", "Select drive (number):".dimmed());
        std::io::Write::flush(&mut std::io::stdout()).ok();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();

        if let Ok(idx) = input.trim().parse::<usize>() {
            if idx >= 1 && idx <= suitable_drives.len() {
                let chosen_drive = &suitable_drives[idx - 1].0;
                let new_dir = PathBuf::from(chosen_drive).join("Downloads");
                std::fs::create_dir_all(&new_dir).ok();
                banner::print_success(&format!("Output directory switched to: {}", new_dir.display()));
                return new_dir;
            } else if idx == suitable_drives.len() + 1 {
                print!("  {} ", "Enter custom folder path:".dimmed());
                std::io::Write::flush(&mut std::io::stdout()).ok();
                let mut custom_path = String::new();
                std::io::stdin().read_line(&mut custom_path).ok();
                let trimmed = custom_path.trim();
                if !trimmed.is_empty() {
                    let custom_dir = PathBuf::from(trimmed);
                    std::fs::create_dir_all(&custom_dir).ok();
                    banner::print_success(&format!("Output directory set to: {}", custom_dir.display()));
                    return custom_dir;
                }
            }
        }
    }

    out_dir.clone()
}
