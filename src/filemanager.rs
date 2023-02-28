use std::fs::{self};
use std::path::PathBuf;

use crate::config::Filters;

pub struct FileManager {}

impl FileManager {
    /// Copy files in specified directory into destination directory
    pub fn copyto(
        from: &str,
        to: &str,
        filters: &Filters,
        keep_parent: bool,
    ) -> Result<(), std::io::Error> {
        let src_root = PathBuf::from(from);
        let src_root_levels = src_root.components().count();
        let dest_root = PathBuf::from(to);
        let mut path_iterator = IterablePath::new(src_root);

        while let Some(path) = path_iterator.next() {
            println!("process: {:?}", &path);

            let relative_path: PathBuf = path.components().skip(src_root_levels).collect();
            let dest_path = if keep_parent {
                // Keep_parent is true, file name is already in relative path if path is file
                dest_root.join(&relative_path)
            } else {
                if path.is_file() {
                    // File need file name to be joined
                    dest_root.clone().join(path.file_name().unwrap())
                } else {
                    // Directory don't need file name to be joined
                    dest_root.clone()
                }
            };

            if path.is_dir() {
                FileManager::mkdir(&dest_path);
                continue;
            }

            if !FileManager::filter(&path, filters) {
                // Filter not passed, not processed
                continue;
            }

            println!("  move: {:?} -> {:?}", &path, &dest_path);
            fs::copy(&path, &dest_path)?;
        }

        Ok(())
    }

    /// Move files in specified directory into destination directory
    pub fn moveto(
        from: &str,
        to: &str,
        filters: &Filters,
        keep_parent: bool,
    ) -> Result<(), std::io::Error> {
        let src_root = &PathBuf::from(from);
        let src_root_levels = src_root.components().count();
        let dest_root = PathBuf::from(to);
        let mut path_iterator = IterablePath::new(src_root.clone());

        while let Some(path) = path_iterator.next() {
            println!("process: {:?}", &path);

            let relative_path: PathBuf = path.components().skip(src_root_levels).collect();
            let dest_path = if keep_parent {
                // Keep_parent is true, file name is already in relative path if path is file
                dest_root.join(&relative_path)
            } else {
                if path.is_file() {
                    // File need file name to be joined
                    dest_root.clone().join(path.file_name().unwrap())
                } else {
                    // Directory don't need file name to be joined
                    dest_root.clone()
                }
            };

            if path.is_dir() {
                FileManager::mkdir(&dest_path);
                continue;
            }

            if !FileManager::filter(&path, filters) {
                // Filter not passed, not processed
                continue;
            }

            println!("  move: {:?} -> {:?}", &path, &dest_path);
            if &path.to_str().unwrap().chars().next().unwrap() ==
                &dest_path.to_str().unwrap().chars().next().unwrap()
            {
                fs::rename(&path, &dest_path)?;
            } else {
                fs::copy(&path, &dest_path)?;
                fs::remove_file(&path)?;
            }
        }

        // Remove empty directories after files moved
        FileManager::delete_empty_dirs(&src_root);

        Ok(())
    }

    pub fn delete(root_path: &str, filters: &Filters) -> std::io::Result<()> {
        let path = PathBuf::from(root_path);
        let mut path_iterator = IterablePath::new(path.clone());

        while let Some(current_path) = path_iterator.next() {
            if current_path.is_dir() {
                continue;
            }

            if FileManager::filter(&current_path, filters) {
                if let Err(err) = fs::remove_file(&current_path) {
                    println!("remove file failed: {:?}", err);
                }
            }
        }

        FileManager::delete_empty_dirs(&path);

        Ok(())
    }

    /// Recursive directory to delete empty folders
    /// The root directory will not be deleted
    fn delete_empty_dirs(root: &PathBuf) {
        let mut path_iterator = IterablePath::new(root.clone());
        while let Some(path) = path_iterator.next() {
            if path.is_dir() && path.read_dir().unwrap().next().is_none() {
                match fs::remove_dir(&path) {
                    Ok(_) => (),
                    Err(e) => println!("remove file failed: {:?}", e),
                }
            }
        }
    }

    /// Check whether a directory exists, if not, create it
    fn mkdir(path: &PathBuf) {
        if fs::metadata(path).is_err() {
            println!(" mkdir: {:?}", path);
            let warning = format!("cannot create directory:{}", path.to_str().unwrap());
            fs::create_dir_all(path).expect(&warning);
        }
    }

    /// Check all filters, if any of them failed, return false
    fn filter(path: &PathBuf, filters: &Filters) -> bool {
        if let Some(formats) = &filters.formats {
            if !FileManager::filter_formats(path, formats) {
                return false;
            }
        }

        if let Some(size) = &filters.size {
            if !FileManager::filter_size(path, size) {
                return false;
            }
        }

        true
    }

    /// Check formats filter
    fn filter_formats(path: &PathBuf, formats: &Vec<String>) -> bool {
        match path.extension() {
            None => false,
            Some(extension) => formats.iter().any(|f| f == extension.to_str().unwrap()),
        }
    }

    /// Check size filter
    fn filter_size(path: &PathBuf, size: &str) -> bool {
        let parts: Vec<&str> = size.split(" ").collect::<Vec<&str>>();

        if parts.len() != 2 {
            println!("bad size filter: {:?}", size);
            return false;
        }

        match FileManager::parse_filter_size(parts[1]) {
            None => {
                println!("bad size filter: {:?}", size);
                return false;
            }
            Some(size_limit) => {
                let file_size = fs::metadata(path).unwrap().len();

                match parts[0] {
                    ">" => {
                        return file_size > size_limit;
                    }
                    ">=" => {
                        return file_size >= size_limit;
                    }
                    "<" => {
                        return file_size < size_limit;
                    }
                    "<=" => {
                        return file_size <= size_limit;
                    }
                    "==" => {
                        return file_size == size_limit;
                    }
                    _ => {
                        println!("bad size filter: {:?}", size);
                        return false;
                    }
                }
            }
        }
    }

    /// Convert size limit from string to bytes
    fn parse_filter_size(size: &str) -> Option<u64> {
        let unit = size.chars().last().unwrap().to_ascii_uppercase();

        match (&size[0..size.len() - 1]).parse::<u64>() {
            Ok(num) => match unit {
                'B' => Some(num * 1),
                'K' => Some(num * 1024),
                'M' => Some(num * 1024 * 1024),
                'G' => Some(num * 1024 * 1024 * 1024),
                'T' => Some(num * 1024 * 1024 * 1024 * 1024),
                _ => None,
            },
            _ => None,
        }
    }
}

/// Build path iterator with root path, the iterator will loop through all paths under root
pub struct IterablePath {
    tmp_paths: Vec<PathBuf>,
}

impl IterablePath {
    pub fn new(root: PathBuf) -> IterablePath {
        IterablePath {
            tmp_paths: vec![root],
        }
    }
}

impl Iterator for IterablePath {
    type Item = PathBuf;

    fn next(&mut self) -> Option<PathBuf> {
        match self.tmp_paths.pop() {
            None => return None,
            Some(current_path) => {
                if !current_path.exists() {
                    // Current path is not valid(may be deleted by someone), fetch next item
                    return self.next();
                }

                if current_path.is_dir() {
                    // If current path is directory, add sub paths into temp list
                    for entry in fs::read_dir(&current_path).unwrap() {
                        let path = entry.unwrap().path();
                        self.tmp_paths.push(path);
                    }
                }

                Some(current_path)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // use tracing::error;

    use crate::{config::Config, util::log_builder};

    use super::*;
    use core::panic;
    use std::path::PathBuf;

    use log::error;
    use log4rs;

    #[test]
    fn test_iterable_path() {
        let mut path_iter = IterablePath::new(PathBuf::from(r"d:\mv"));
        while let Some(path) = path_iter.next() {
            println!("{:?}", path);
        }
    }

    #[test]
    fn test_parse_filter_size() {
        assert_eq!(None, FileManager::parse_filter_size("xxx"));
        assert_eq!(1, FileManager::parse_filter_size("1b").unwrap());
        assert_eq!(1024, FileManager::parse_filter_size("1k").unwrap());
        assert_eq!(
            5 * 1024 * 1024,
            FileManager::parse_filter_size("5M").unwrap()
        );
        assert_eq!(
            8 * 1024 * 1024 * 1024,
            FileManager::parse_filter_size("8G").unwrap()
        );
        assert_eq!(
            21 * 1024 * 1024 * 1024 * 1024,
            FileManager::parse_filter_size("21T").unwrap()
        );
    }

    #[test]
    fn test_filter_size() {
        let path_3B = PathBuf::from(r"C:\Users\刘东\Downloads\test.txt");
        let path_37M = PathBuf::from(r"C:\Users\刘东\Downloads\codelldb-x86_64-windows.vsix");

        // Correct format test
        assert_eq!(true, FileManager::filter_size(&path_3B, "< 1K"));
        assert_eq!(true, FileManager::filter_size(&path_3B, "<= 1K"));
        assert_eq!(true, FileManager::filter_size(&path_3B, "== 3B"));
        assert_eq!(true, FileManager::filter_size(&path_37M, ">= 10m"));
        assert_eq!(true, FileManager::filter_size(&path_37M, "< 1G"));

        // Wrong format test
        assert_eq!(false, FileManager::filter_size(&path_37M, "== 1G"));
        assert_eq!(false, FileManager::filter_size(&path_37M, "= 1G"));
        assert_eq!(false, FileManager::filter_size(&path_37M, "> 2P"));
        assert_eq!(false, FileManager::filter_size(&path_37M, ">2M"));
    }

    #[test]
    fn test_filter_formats() {
        let formats = vec!["txt".to_string(), "vsix".to_string()];
        let path_txt = PathBuf::from(r"C:\Users\刘东\Downloads\test.txt");
        let path_vsix = PathBuf::from(r"C:\Users\刘东\Downloads\codelldb-x86_64-windows.vsix");
        let path_zip = PathBuf::from(r"C:\Users\刘东\Downloads\HBuilderX.3.6.14.20221215.zip");
        let path_dir = PathBuf::from(r"C:\Users\刘东\Downloads");

        assert_eq!(true, FileManager::filter_formats(&path_txt, &formats));
        assert_eq!(true, FileManager::filter_formats(&path_vsix, &formats));

        assert_eq!(false, FileManager::filter_formats(&path_zip, &formats));
        assert_eq!(false, FileManager::filter_formats(&path_dir, &formats));
    }

    #[test]
    fn test_mkdir() {
        let path_dir = PathBuf::from(r"d:\\test_make_dir");

        if fs::metadata(&path_dir).is_ok() {
            // If directory exists, delete first
            fs::remove_dir_all(&path_dir).unwrap();
        }

        // Test first time create
        FileManager::mkdir(&path_dir);
        assert_eq!(true, fs::metadata(&path_dir).is_ok());

        // Test repeated create
        FileManager::mkdir(&path_dir);
        assert_eq!(true, fs::metadata(&path_dir).is_ok());
    }

    #[test]
    fn test_delete_empty_dirs() {
        let path_dir = PathBuf::from(r"d:\\test_make_dir");
        FileManager::delete_empty_dirs(&path_dir);
    }

    #[test]
    fn test_move_to() {
        // log_builder::load_logger(log::LevelFilter::Warn);

        let conf = Config::load();
        assert_ne!(None, conf);

        let uw_conf = conf.unwrap();
        let pr = std::panic::catch_unwind(|| {
            let r = FileManager::moveto(
                &uw_conf.mv.from,
                &uw_conf.mv.to,
                &uw_conf.mv.filters,
                uw_conf.mv.keep_parent_dir,
            );

            if let Err(err) = r {
                error!("{}", err);
            }
        });

        if let Err(err) = pr {
            error!("{:?}", err);
        }
    }

    #[test]
    fn test_delete() {
        let conf = Config::load();
        assert_ne!(None, conf);

        let uw_conf = conf.unwrap();
        assert_eq!(
            true,
            FileManager::delete(&uw_conf.del.path, &uw_conf.del.filters).is_ok()
        );
    }
}
