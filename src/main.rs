mod error;
mod utils;
mod structs;

use error::DynHashError;
use std::{
    fs::{self, File},
    io::Write,
    result::Result,
};

fn main() -> Result<(), error::DynHashError> {
    // Define an absolute path for cwd
    let cwd = fs::canonicalize(".")?;

    // Set the starting point at the dist directory
    // this is where vite will push our build files.
    let build_dir = cwd.join("dist");

    // Walk through all the build directory collecting all the
    // hash files as a Vec<DynHashFiles>, I abstracted this away
    // into seperate utils.rs to clean up the main function.
    let dyn_hash_files = utils::collect_hash_files(&build_dir)?;

    // Read the current web routes file content by line
    let updated_web_routes = if let Ok(lines) = utils::read_web_routes_by_line(&cwd) {
        // Create a new mutable string to collect our updated web routes file
        // content and iterate over each line in the current web routes file
        let mut updated_routes = String::new();
        for line_result in lines {
            // if line, then define a mutable variable for line
            if let Ok(mut line) = line_result {
                for dhf in dyn_hash_files.iter() {
                    // Extract dynamic values
                    let prefix = format!("{}-", dhf.prefix);
                    let ext = format!(".{}", dhf.ext);
    
                    // Check if this line should be updated or not
                    // by seeing if we find the prefix and ext and 
                    // getting the index of where the prefix starts.
                    if let (Some(i), Some(_)) = (line.find(&prefix), line.find(&ext)) {
                        // Replace the hash with the new one
                        let start = i + &prefix.len();
                        line.replace_range(start..start + 8, &dhf.hash);
                        break;
                    }
                }
    
                // Collect line for updated web routes file content string
                updated_routes.push_str(&line);
                updated_routes.push('\n');
            } else if let Err(e) = line_result {
                eprintln!("{:?}", e);
            }
        }
        Some(updated_routes)
    } else {
        None
    };

    // Convert the string to bytes to write to file
    let updated_web_routes_data = updated_web_routes
        .expect("updated_web_routes not to be None?");

    // Write the updated web routes file
    let mut web_routes_file = File::create(cwd.join("../routes/web.rs"))?;
    match web_routes_file.write_all(updated_web_routes_data.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{e:?}");
            Err(DynHashError::FailedToWriteUpdated)
        }
    }
}
