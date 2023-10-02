mod error;
use crate::error::DynHashError;
use std::{
    collections::VecDeque,
    fs::{self, DirEntry, File},
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
    result::Result,
};

#[derive(Debug)]
struct DynHashFile {
    prefix: String,
    hash: String,
    ext: String,
}

/// Read web routes file (`src/routes/web.rs`) returning an Iterator over the lines.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn main() -> Result<(), error::DynHashError> {
    // Define an absolute path for cwd
    let cwd = fs::canonicalize(".")?;

    // Set the starting point at the dist directory
    // this is where vite will push our build files.
    let build_dir = cwd.join("dist");

    // Vector which is used to collect dynamic hash files
    let mut dyn_hash_files: Vec<DynHashFile> = Vec::new();

    // Walk through all the build files
    let mut stack: VecDeque<(PathBuf, VecDeque<io::Result<DirEntry>>)> = VecDeque::new();
    let dir_entries = fs::read_dir(&build_dir)?
        .map(|entry| entry)
        .collect::<VecDeque<_>>();
    stack.push_back((build_dir, dir_entries));
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
                let path = path
                    .to_str()
                    .unwrap_or("")
                    .replace(cwd.to_str().unwrap(), "")
                    .to_string();
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
                    dyn_hash_files.push(DynHashFile { prefix, hash, ext });
                }
            }
        }
    }

    // Create the new updated web routes file content
    let mut updated_web_routes = String::new();
    if let Ok(lines) = read_lines(cwd.join("../routes/web.rs")) {
        for line in lines {
            match line {
                Ok(line) => {
                    // Default new_line to a clone of line
                    let mut new_line = line.clone();

                    // check for each dynamic hash filename
                    // TODO: more efficient way to do this
                    for dhf in dyn_hash_files.iter() {
                        let prefix = format!("{}-", dhf.prefix);
                        let ext = format!(".{}", dhf.ext);

                        // Check if this line should be updated or not
                        if line.contains(&prefix) && line.contains(&ext) {
                            // index of where the prefix ends
                            let i = line.find(dhf.prefix.as_str()).unwrap() + dhf.prefix.len();

                            // Create the new line string and break out of loop
                            new_line = format!(
                                "{}{}{}",
                                line.get(..i + 1).unwrap(),
                                &dhf.hash,
                                line.get(i + 9..).unwrap()
                            );
                            break;
                        }
                    }

                    // push the new line to our web routes file content string
                    updated_web_routes.push_str(&new_line);
                    updated_web_routes.push_str("\n");
                }
                Err(e) => eprintln!("{:?}", e),
            }
        }
    }

    // Write the updated web routes file
    let mut web_routes_file = File::create(cwd.join("../routes/web.rs"))?;
    match web_routes_file.write_all(updated_web_routes.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{e:?}");
            Err(DynHashError::FailedToWriteUpdated)
        }
    }
}
