use std::collections::VecDeque;
use std::fs::{self, DirEntry};
use std::{io, path::{Path, PathBuf}};

struct DynHashFile {
    prefix: String,
    hash: String,
    ext: String,
    path: String,
}

fn main() -> io::Result<()> {
    // Define an absolute path for cwd
    let cwd = fs::canonicalize(".")?;

    // Set the starting point at the dist directory
    // this is where vite will push our build files.
    let root_dir = "./dist";

    // Vector which is used to collect dynamic hash files
    let mut dyn_hash_files: Vec<DynHashFile> = Vec::new();

    // Walk through all the build files
    let mut stack: VecDeque<(PathBuf, VecDeque<io::Result<DirEntry>>)> = VecDeque::new();
    let initial_dir = Path::new(root_dir).to_path_buf();
    let dir_entries = fs::read_dir(&initial_dir)?
        .map(|entry| entry)
        .collect::<VecDeque<_>>();
    stack.push_back((initial_dir, dir_entries));
    while let Some((_, mut entries)) = stack.pop_back() {
        while let Some(entry_result) = entries.pop_front() {
            let entry = entry_result?;
            let path = entry.path();
            if path.is_dir() {
                let sub_entries = fs::read_dir(&path)?
                    .map(|entry| entry)
                    .collect::<VecDeque<_>>();
                stack.push_back((path, sub_entries));
            } else {
                // Check if it's a dynamic hash file
                let path = path.to_str().unwrap_or("").to_string();
                if path.contains("-") {
                    // Parse the path
                    let dash_split: Vec<&str> = path.split("-").collect();
                    let slash_split: Vec<&str> = dash_split[0].split("/").collect();
                    let dot_split: Vec<&str> = dash_split[1].split(".").collect();

                    // Extract target data
                    let prefix = slash_split[slash_split.len() - 1].to_string();
                    let hash = dot_split[0].to_string();
                    let ext = dot_split[1].to_string();

                    // Collect the dynamic hash file
                    dyn_hash_files.push(DynHashFile {
                        prefix,
                        hash,
                        ext,
                        path,
                    });     
                }
            }
        }
    }

    Ok(())
}

