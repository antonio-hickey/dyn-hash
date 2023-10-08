use crate::structs::DynHashFile;
use std::{
    collections::VecDeque,
    fs::{read_dir, File, DirEntry},
    io::{Result, Lines, BufRead, BufReader},
    path::PathBuf,
};

/// Read web routes file (`src/routes/web.rs`) returning an Iterator over the lines.
pub fn read_web_routes_by_line(cwd: &PathBuf) -> Result<Lines<BufReader<File>>>{
    let file = File::open(cwd.join("../routes/web.rs"))?;
    Ok(BufReader::new(file).lines())
}

/// Walk through the build directory and collect a vector of all the 
/// build files with a hash in the filename.
pub fn collect_hash_files(build_dir: &PathBuf) -> Result<Vec<DynHashFile>> {
    let mut dyn_hash_files: Vec<DynHashFile> = Vec::new();

    // Queue for managing directories during traversal
    let mut stack: VecDeque<(PathBuf, VecDeque<Result<DirEntry>>)> = VecDeque::new();
    let dir_entries = read_dir(&build_dir)?.map(|entry| entry).collect::<VecDeque<_>>();
    stack.push_back((build_dir.to_path_buf(), dir_entries));

    // Start traversing through the build directory and populating the
    // queue with newly discovered directories and their contents
    while let Some((_current_dir, mut entries)) = stack.pop_back() {
        while let Some(entry_result) = entries.pop_front() {
            let entry = entry_result?;
            let path = entry.path();

            // If the entry is a directory the add
            // its path and children to the stack
            if path.is_dir() {
                let sub_entries = read_dir(&path)?
                    .map(|entry| entry)
                    .collect::<VecDeque<_>>();
                stack.push_back((path, sub_entries));
            } else {
                // it's a file, so check if the filename has a hash
                // if so parse & collect as a `DynHashFile`
                if let Some(path_str) = path.to_str() {
                    if let Some(dhf) = parse_dyn_hash_file_path(path_str) {
                        dyn_hash_files.push(dhf);
                    }
                }
            }
        }
    }

    Ok(dyn_hash_files)
}

/// Convert a path into a `DynHashFile`
fn parse_dyn_hash_file_path(path: &str) -> Option<DynHashFile> {
    // Seperate the filename from the absolute path by
    // splitting the path by the last occurance of '/'
    let filename = path
        .rsplitn(2, '/')
        .collect::<Vec<&str>>()[0];

    // Seperate the prefix from hash and extension by splitting 
    // the filename by '-' creating [PREFIX_STR, HASH_AND_EXTENSION_STR]
    let parts: Vec<&str> = filename.rsplitn(2, '-').collect();
    if parts.len() == 2 {
        // Seperate hash and extension string creating [HASH_STR, EXTENSION_STR]
        let hash_and_ext: Vec<&str> = parts[0].split('.').collect();
        if hash_and_ext.len() == 2 {
            // Create and retrun a `DynHashFile`
            return Some(DynHashFile {
                prefix: parts[1].to_string(),
                hash: hash_and_ext[0].to_string(),
                ext: hash_and_ext[1].to_string(),
                web_route: format!("#[get(\"{}\")]", &filename),
                filename: filename.to_string(),
            })
        }
    }
    None
}

/// Check if route already exists
pub fn has_route_yet(source_code: &str, prefix: &str, ext: &str) -> bool {
    // Extract & format static code chunks
    let route_prefix = format!("#[get(\"{}-", prefix);
    let route_ext = format!(".{}\")]", ext);

    // Loop over all the lines
    for line in source_code.lines() {
        // Check if line is declaring route macro 
        if line.contains(&route_prefix) && line.contains(&route_ext) {
            return true
        }
    }

    // Didn't find the route macro declaration
    false
}
