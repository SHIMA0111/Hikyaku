use std::path::{Component, Path};
use log::error;
use regex::Regex;
use crate::errors::HikyakuError::InvalidArgumentError;
use crate::errors::HikyakuResult;

// This regex is used to parse the input path into namespace, and path components.
const FILE_SYSTEM_NAMESPACE_PATH_REGEX: &str = r"^/*([^/]+)/?(.*?[^/])?/*$";

#[derive(Debug)]
/// File path parser result.
pub(crate) struct FileSystemParseResult {
    prefix: String,
    namespace: Option<String>,
    path: String,
}

impl FileSystemParseResult {
    /// Get prefix(e.x. `s3://`, `file://`, and so)
    pub(crate) fn get_prefix(&self) -> &str {
        &self.prefix
    }

    /// Get namespace(S3 bucket name, Google Drive SharedDrive name).  
    /// If the file system has no namespace, return [None].
    pub(crate) fn get_namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    /// Get path except for namespace(`file://` and `gd://` has no namespace so all path is parsed)
    pub(crate) fn get_path(&self) -> &str {
        &self.path
    }
}

/// Parse the user input path to prefix and namespace, path.
/// 
/// # Arguments
/// - `input`: User input path. This should be start with valid prefixes.
/// 
/// # Prefixes
/// - `file://`: Local file system path
/// - `s3://`: Amazon S3 path
/// - `gd://`: Google Drive MyDrive path
/// - `gds://`: Google Drive Shared path (The first path is treated as SharedDrive name)  
/// ※ Originally, Google Drive has no concept of the path. In a pseudo manner, 
/// the file parent-child relationship uses as the path.
/// 
/// # Returns
/// - HikyakuResult<[FileSystemParseResult]>: `FileSystemParseResult` has the prefix, 
/// [Option] of namespace(a.k.a Amazon S3 bucket or Google Drive SharedDrive), path except the namespace.
/// When the path is invalid, returns [InvalidArgumentError].
pub(crate) fn file_system_prefix_parser(input: &str) -> HikyakuResult<FileSystemParseResult> {
    let (prefix, path) = if input.starts_with("file://") {
        // SAFETY: In this branch, the input always has 'file://' so the result is always Some.
        let (_, path) = input.split_once("file://").unwrap();

        ("file://", path)
    }
    else if input.starts_with("s3://") {
        let (_, path) = input.split_once("s3://").unwrap();

        ("s3://", path)
    }
    else if input.starts_with("gd://") {
        let (_, path) = input.split_once("gd://").unwrap();

        ("gd://", path)
    }
    else if input.starts_with("gds://") {
        let (_, path) = input.split_once("gds://").unwrap();

        ("gds://", path)
    }
    else {
        error!("Input path is invalid: {}", input);
        return Err(InvalidArgumentError(format!("Invalid Path: {} is invalid prefix. Support only 'file://', 's3://', 'gd://', 'gds://'", input)))
    };

    // s3 and SharedDrive needs namespace
    if ["s3://", "gds://"].contains(&prefix) {
        // SAFETY: The regex statement is const string so this is always Ok().
        let regex = Regex::new(FILE_SYSTEM_NAMESPACE_PATH_REGEX).unwrap();

        let path_capture = regex.captures(path)
            .ok_or_else(|| {
                error!("Input path is invalid due to not have namespace: {}", path);
                InvalidArgumentError(
                    format!("Invalid Path: {} is invalid path. 's3://' and 'gds://' must have namespace", input))
            })?;
        let namespace = path_capture.get(1)
            .ok_or_else(|| {
                error!("Input path is invalid due to not have namespace: {}", path);
                InvalidArgumentError(
                    format!("Invalid Path: {} is invalid path. 's3://' and 'gds://' must have namespace", input))
            })?
            .as_str()
            .to_string();
        let path = path_capture.get(2)
            .map(|c| c.as_str().to_string())
            .unwrap_or(String::new());

        // When the path has '//' or so, it can be ambiguous so refuse it.
        // path.starts_with("/") is for between namespace and path '//'.
        if path.contains("//") || path.starts_with("/") {
            error!("Input path is invalid due to have 2 or more chained slash: {}", input);
            return Err(
                InvalidArgumentError(
                    format!(
                        "Invalid Path: {} is invalid path. To avoid ambiguous process, \
                        '//' or more chain slash cannot use.", input)))
        }

        Ok(FileSystemParseResult {
            prefix: prefix.to_string(),
            namespace: Some(namespace),
            path,
        })
    }
    else {
        if path.contains("//") {
            error!("Input path is invalid due to have 2 or more chained slash: {}", input);
            return Err(
                InvalidArgumentError(
                    format!(
                        "Invalid Path: {} is invalid path. To avoid ambiguous process, \
                        '//' or more chain slash cannot use.", input)))
        }

        // To get a path without head slash, the regex extracts greedy.
        let regex = Regex::new(r"^/*(.*[^/])/*$").unwrap();
        // In MyDrive and Local file system, the path except prefix can be "/" or "".
        let path = if path.is_empty() || path == "/" {
            path.to_string()
        }
        else {
            // SAFETY: From the above `if` branch, this `capture` can't be None.
            // Because the greedy regex extract any value except for only "/" path or empty path.
            // The multiple slash like "////" in path was refused the first if-statement. 
            // And the above if-statement extract the empty or "/". Therefore, this should always Some.
            let capture = regex.captures(path).unwrap();
            let matches = capture.get(1).unwrap();
            matches.as_str().to_string()
        };
        
        Ok(FileSystemParseResult {
            prefix: prefix.to_string(),
            namespace: None,
            path,
        })
    }
}

pub(crate) fn path_to_names_vec(path: &str, allow_metacharacter: bool) -> HikyakuResult<Vec<String>> {
    let components = Path::new(path)
        .components()
        // Component::CurDir never contains root dir due to the parse input, the first '/'(slash) was eliminated.
        .collect::<Vec<_>>();

    if !allow_metacharacter && components.iter().any(|component| matches!(component, Component::CurDir | Component::ParentDir)) {
        return Err(InvalidArgumentError(format!("File path cannot contain metacharacter to avoid ambiguous path. got: {}", path)));
    }

    if components.iter().any(|component| component.as_os_str().to_str().is_none()) {
        return Err(InvalidArgumentError(format!("File path cannot contain non-ASCII character. but got: {}", path)));
    }

    let path_names = components
        .iter()
        // SAFETY: The components always can convert to String by the above validation.
        .map(|component| component.as_os_str().to_str().unwrap().to_string())
        .collect::<Vec<_>>();

    Ok(path_names)
}

#[cfg(test)]
mod tests {
    use crate::errors::HikyakuError::InvalidArgumentError;
    use super::file_system_prefix_parser;
    
    #[test]
    fn test_file_system_prefix_parser_no_namespace() {
        let result = file_system_prefix_parser("file:///test/test1/test2").unwrap();
        assert_eq!(result.get_prefix(), "file://");
        assert_eq!(result.get_namespace(), None);
        assert_eq!(result.get_path(), "test/test1/test2");
        
        let result = file_system_prefix_parser("gd://test1/test2/").unwrap();
        assert_eq!(result.get_prefix(), "gd://");
        assert_eq!(result.get_namespace(), None);
        assert_eq!(result.get_path(), "test1/test2");
    }
    
    #[test]
    fn test_file_system_prefix_parser_with_namespace() {
        let result = file_system_prefix_parser("s3://test/test1/test2/").unwrap();
        assert_eq!(result.get_prefix(), "s3://");
        assert_eq!(result.get_namespace(), Some("test"));
        assert_eq!(result.get_path(), "test1/test2");
        
        let result = file_system_prefix_parser("gds:///test_gd/test1/test2").unwrap();
        assert_eq!(result.get_prefix(), "gds://");
        assert_eq!(result.get_namespace(), Some("test_gd"));
        assert_eq!(result.get_path(), "test1/test2");
    }
    
    #[test]
    fn test_file_system_prefix_parser_invalid_prefix() {
        let result = file_system_prefix_parser("invalid_prefix:///test/test1/test2");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(
            error.to_string(), 
            InvalidArgumentError(
                "Invalid Path: invalid_prefix:///test/test1/test2 is invalid prefix. \
                Support only 'file://', 's3://', 'gd://', 'gds://'".to_string()).to_string());
    }
    
    #[test]
    fn test_file_system_prefix_parser_invalid_path() {
        let result = file_system_prefix_parser("file:///test/test1//test2/");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(
            error.to_string(), 
            InvalidArgumentError(
                "Invalid Path: file:///test/test1//test2/ is invalid path. \
                To avoid ambiguous process, '//' or more chain slash cannot use.".to_string()).to_string());
        
        let result = file_system_prefix_parser("s3:///test//test1/test2");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(
            error.to_string(), 
            InvalidArgumentError(
                "Invalid Path: s3:///test//test1/test2 is invalid path. \
                To avoid ambiguous process, '//' or more chain slash cannot use.".to_string()).to_string());
        
        let result = file_system_prefix_parser("file://////");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(
            error.to_string(),
            InvalidArgumentError(
                "Invalid Path: file:////// is invalid path. \
                 To avoid ambiguous process, '//' or more chain slash cannot use.".to_string()
            ).to_string())
    }
}