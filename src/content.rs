use std::{
    fs::{self, File},
    io::{Read, Result},
    path::{Path, PathBuf},
};

use pulldown_cmark::Parser;
use serde::Serialize;

use crate::preparser::ContentMetadata;

#[derive(Debug, Serialize)]
pub struct Content {
    #[serde(skip_serializing)]
    raw_content: String,
    pub metadata: ContentMetadata,
}

impl Content {
    fn get_clean_list_of_content_paths<P: AsRef<Path>>(content_root_dir: &P) -> Vec<PathBuf> {
        let mut list_of_contents = Vec::<PathBuf>::new();

        if let Ok(entries) = fs::read_dir(content_root_dir) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry.file_type().is_ok_and(|file_type| {
                    file_type.is_file()
                        && entry_path
                            .extension()
                            .is_some_and(|extension| extension == "md")
                        && entry_path
                            .file_name()
                            // Index content should always be processed separately
                            .is_some_and(|file_name| file_name != "_index.md")
                }) {
                    list_of_contents.push(entry_path);
                }
            }
        }

        list_of_contents
    }

    pub fn from_file<P: AsRef<Path>>(file: &P) -> Result<Self> {
        let mut opened_file = File::open(file)?;
        let mut raw_content = String::new();
        opened_file.read_to_string(&mut raw_content)?;

        // TODO: Make this more succint and better to read, because right now, wtf?
        let current_path: String = match Path::new(file.as_ref()).file_stem() {
            Some(osstr_file_name) => {
                osstr_file_name
                    .to_os_string()
                    .into_string()
                    .or(Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Cannot make OsString to string!",
                    )))?
            }
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Cannot stem file name",
                ))
            }
        };

        // The result should metadata and remove the meatadata part on the raw_content
        let (metadata, processed_content) =
            ContentMetadata::preprocess_content_metadata(raw_content, current_path);

        Ok(Self {
            raw_content: processed_content,
            metadata,
        })
    }

    /// This function assume that `path` is the root of `contents` path
    pub fn from_dir<P: AsRef<Path>>(dir: &P) -> Vec<Self> {
        let all_content_paths = Self::get_clean_list_of_content_paths(dir);
        let mut contents = Vec::<Self>::with_capacity(all_content_paths.len());

        println!("[Content::from_dir] Starting to parse: {contents:#?}");

        for content_path in all_content_paths {
            println!("[Content::from_dir] Parsing file '{content_path:#?}");

            match Self::from_file(&content_path) {
                Ok(content) => contents.push(content),
                // This should be unlikely but, who knows?
                Err(err) => println!("[Content::from_dir] Error parsing content on path '{content_path:#?}'! Error message: '{err:#?}'")
            };
        }

        // Save some space as all content paths could result in error
        contents.shrink_to_fit();

        contents
    }

    pub fn to_html(&self) -> String {
        let parser = Parser::new(&self.raw_content);
        let mut html_output = String::new();

        pulldown_cmark::html::push_html(&mut html_output, parser);

        html_output
    }
}

#[cfg(test)]
mod content_test {
    use std::path::PathBuf;

    use crate::content::Content;

    const MAIN_DIR: &str = env!("CARGO_MANIFEST_DIR");

    fn get_path_to_test_files() -> PathBuf {
        let mut root_path = PathBuf::from(MAIN_DIR);
        root_path.push("test_files/blog");

        root_path
    }

    #[test]
    fn test_content_file_parsing() {
        let mut test_dir = get_path_to_test_files();
        test_dir.push("test-hello.md");

        println!("[content_test::test_content_file_parsing] reading from file path: {test_dir:#?}");

        let content = Content::from_file(&test_dir);

        assert!(
            content.is_ok(),
            "Content from known file should NOT result in error!"
        );

        assert!(
            content
                .as_ref()
                .is_ok_and(|result| result.metadata.author == Some("fauh45".into())),
            "Known valid content SHOULD parse metadata succesfully!"
        );
        assert!(
            content
                .as_ref()
                .is_ok_and(|result| result.metadata.title == Some("Hello World!".into())),
            "Known valid content SHOULD parse metadata succesfully!"
        );
        assert!(
            content.as_ref().is_ok_and(|result| result
                .metadata
                .tags
                .as_ref()
                .is_some_and(|tags| tags.len() == 3)),
            "Known valid content SHOULD parse metadata succesfully!"
        );

        let html_ouput = content.unwrap().to_html();

        assert!(
            !html_ouput.is_empty(),
            "HTML output are expected to return something!"
        )
    }

    #[test]
    fn test_directory_parsing() {
        let test_dir = get_path_to_test_files();

        let contents = Content::from_dir(&test_dir);

        assert_eq!(
            contents.len(),
            1,
            "Content returned more or less than known at the test files!"
        );

        let first_result = contents.first();
        assert!(
            first_result
                .is_some_and(|content| content.metadata.title == Some("Hello World!".into())),
            "Content from directory did not match the known content!"
        );
    }
}
