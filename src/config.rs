// use log::{error, info, warn};
use std::{
    fs::{File, OpenOptions},
    io::{LineWriter, Write},
    path::Path,
};

extern crate serde;
extern crate serde_yaml;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Config {
    /// Options for move action
    pub mv: MoveOptions,
    /// Options for copy action
    pub cp: CopyOptions,
    /// Options for delete action
    pub del: DeleteOptions,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CopyOptions {
    /// Source address of the folder or file.
    pub from: String,
    /// Destination address of the folder or file.
    pub to: String,
    /// If value is set to true, also copy directories.
    pub keep_parent_dir: bool,
    /// The filters, only files satisfied the condition in filters will be copied.
    pub filters: Filters,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct MoveOptions {
    /// Source address of the folder or file.
    pub from: String,
    /// Destination address of the folder or file.
    pub to: String,
    /// If value is set to false, only files will be moved, directories will be deleted.
    pub keep_parent_dir: bool,
    /// The filters, only files satisfied the condition in filters will be moved.
    pub filters: Filters,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct DeleteOptions {
    /// Path of directory or file to delete.
    pub path: String,
    /// The fileters, only files satisfied the condition in filters will be deleted.
    pub filters: Filters,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Filters {
    /// Indicate the formats of files that will be chosen
    pub formats: Option<Vec<String>>,
    /// Size filter, formatted as "> 100M", "<=  10G" or "== 5K".
    /// The units available are "B", "K", "M", "G", "T"
    pub size: Option<String>,
}

impl Config {
    pub fn load() -> Option<Config> {
        let f = File::open("config.yml").unwrap();
        match serde_yaml::from_reader(f) {
            Ok(config) => config,
            Err(e) => {
                if !Path::new("main.log").try_exists().unwrap() {
                    File::create("main.log").unwrap();
                }

                let mut f = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open("main.log")
                    .unwrap();
                if writeln!(f, "failed to parse config.yml: {}", e).is_ok() {
                    f.flush().unwrap();
                }

                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {
        let conf = Config::load();
        assert_eq!(true, conf.is_some());
    }
}
