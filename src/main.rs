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

    // The projects error name, if found will be set to some string
    let mut error_name: Option<String> = None;

    // Read the current web routes file content by line
    let updated_web_routes = if let Ok(lines) = utils::read_web_routes_by_line(&cwd) {
        // Create a new mutable string to collect our updated web routes file
        // content and iterate over each line in the current web routes file
        let mut updated_routes = String::new();
        for line_result in lines {
            // if line, then define a mutable variable for line
            if let Ok(mut line) = line_result {

                // If we haven't already found the error name 
                // then check if line is the error import
                if error_name.is_none() && line.contains("use crate::error::") {
                    // Collect the error name
                    error_name = Some(
                        line
                            .replace("use crate::error::", "")
                            .replace(";", "")
                    );
                }

                for dhf in dyn_hash_files.iter() {
                    // Extract & format values
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
    let mut updated_web_routes_data = updated_web_routes
        .expect("updated_web_routes not to be None?");

    // Extract error name or panic out
    let proj_error_name = error_name.expect("Project error name was not found");

    // Create web hash routes if not already present
    for dhf in dyn_hash_files.iter() {
        if !utils::has_route_yet(&updated_web_routes_data, &dhf.prefix, &dhf.ext) {
            // Create 2 new lines
            updated_web_routes_data.push_str("\n");

            // Create the Actix web route macro
            updated_web_routes_data.push_str(&dhf.web_route);
            updated_web_routes_data.push_str("\n");

            // Create the function "definition"
            let function_def_code = format!(
                "pub async fn get_{}_{}() -> Result<NamedFile, {}> {{\n", 
                &dhf.prefix, 
                &dhf.ext, 
                &proj_error_name
            );
            updated_web_routes_data.push_str(&function_def_code);

            // Create the response with file content
            let response_code = format!("    Ok(NamedFile::open(\"src/web/dist/assets/{}\").unwrap())\n", &dhf.filename);
            updated_web_routes_data.push_str(&response_code);

            // Create the close function notation
            updated_web_routes_data.push_str("}\n")
        }
    }

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
