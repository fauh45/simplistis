use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use handlebars::{to_json, Handlebars};
use serde::Serialize;
use serde_json::{value::Value, Map};

use crate::content::Content;

#[derive(Debug, Serialize)]
pub struct Page {
    pub(crate) path: String,
    content: Content,
    template: String,

    /// If true this represent the page to be the root of a directory
    pub(crate) is_dir_root: bool,
    pub(crate) child: Vec<Page>,
}

impl Page {
    fn get_template_content<P: AsRef<Path>>(path: &P, path_file_name: &str) -> Option<String> {
        let mut template_path = PathBuf::new();
        template_path.push(path.as_ref());
        template_path.push(path_file_name);

        if !template_path.exists() {
            return None;
        }

        let mut template_content = String::new();

        // Safe to unwrap as we check if path exist or not, though there could be problem
        // where the file is not allowed to be open by the user
        // TODO: Handle file open error
        if let Err(err) = (File::open(template_path))
            .unwrap()
            .read_to_string(&mut template_content)
        {
            println!("[Page::parse_one_page] Cannot open {path_file_name} file! Error: {err:#?}");

            return None;
        }

        Some(template_content)
    }
    /// This function assume `path` is the root of a page.
    fn parse_one_page<P: AsRef<Path>, BP: AsRef<Path>>(base_path: &BP, path: &P) -> Option<Self> {
        let mut current_path = PathBuf::new();
        current_path.push(path.as_ref());

        let Some(current_dir) = current_path.file_name() else {
            println!("[Page::parse_one_page] Cannot get dir name!");

            return None;
        };

        let mut index_content_path = current_path.clone();
        index_content_path.push("_index.md");

        if !index_content_path.exists() {
            println!("[Page::parse_one_page] No _index.md on directory {current_dir:?}!");

            return None;
        }

        let template_content = Self::get_template_content(path, "template.hbs")?;
        let content_template_content = Self::get_template_content(path, "content.hbs");

        let Ok(index_content) = Content::from_file(&index_content_path) else {
            println!(
                "[Page::parse_one_page] Failed to parse _index.md on directory {current_dir:?}!"
            );

            return None;
        };

        let mut current_root_path = String::from("/");
        current_root_path.push_str(
            current_path
                .strip_prefix(base_path)
                .unwrap()
                .to_str()
                .unwrap(),
        );

        let mut current_root = Self {
            path: current_root_path.clone(),
            content: index_content,
            template: template_content.clone(),
            is_dir_root: true,
            child: vec![],
        };

        let contents = Content::from_dir(&current_path);

        if !contents.is_empty() && content_template_content.is_none() {
            println!("[Page::parse_one_page] There's no content template (content.hbs) but there's content, skipping parsing content on directory {current_dir:?}!");
        } else {
            for content in contents {
                let mut child_path = current_root_path.clone();
                // Add another sub-path
                child_path.push('/');

                // This should be safe as `slug` is guaranteed to always be there
                child_path.push_str(content.metadata.slug.as_ref().unwrap());

                current_root.child.push(Self {
                    path: child_path,
                    template: content_template_content.as_ref().unwrap().to_string(),
                    content,
                    is_dir_root: false,
                    child: vec![],
                })
            }
        }

        Some(current_root)
    }

    /// This function assume that `path` is the root of all of the pages.
    /// The expected file structure are as follows,
    ///
    /// ```text
    /// (root:dir) ->
    /// ....(path:dir) ->
    /// ........(contents:file).md
    /// ........template.hbs (optional, path level template:file)
    /// ...._index.md (root page "/":file)
    /// ....templates.hbs (root level template:file)
    /// ```
    pub fn from_dir<P: AsRef<Path>>(root_path: &P) -> Option<Self> {
        let root_base_path = Path::new(root_path.as_ref());

        let mut index_dir_path = PathBuf::new();
        index_dir_path.push(root_path);

        let Some(mut root_page) = Self::parse_one_page(&root_base_path, &index_dir_path) else {
            println!("[Page::from_dir] Failed parsing root directory!");

            return None;
        };

        if let Ok(entries) = fs::read_dir(root_path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();

                if entry.file_type().is_ok_and(|file_type| file_type.is_dir()) {
                    let current_page = Self::parse_one_page(&root_base_path, &entry_path);

                    if let Some(current_page) = current_page {
                        root_page.child.push(current_page);
                    } else {
                        println!("[Page::from_dir] Cannot parse page: {entry:?}");
                    }
                }
            }
        } else {
            println!("[Page::from_dir] Error reading root websiter path!")
        }

        Some(root_page)
    }

    /// `output_dir` expects to be valid and already exist, and is the root of the file that will be rendered.
    pub fn render<P: AsRef<Path>>(self, output_dir: &P) -> Result<(), Box<dyn std::error::Error>> {
        let mut output_path = PathBuf::new();
        output_path.push(output_dir);

        let mut hbs_registry = Handlebars::new();
        hbs_registry.register_template_string(&self.path, self.template)?;

        let mut render_data = Map::<String, Value>::new();

        // Should be safe as slug are always there
        let mut slug = self.content.metadata.slug.as_ref().unwrap().to_string();
        slug.push_str(".html");
        output_path.push(slug);

        render_data.insert("content".into(), to_json(self.content.to_html()));

        let output_file = File::create(output_dir)?;
        hbs_registry.render_to_write(&self.path, &render_data, output_file)?;

        Ok(())
    }
}

#[cfg(test)]
mod page_test {
    use std::path::PathBuf;

    use crate::pages::Page;

    const MAIN_DIR: &str = env!("CARGO_MANIFEST_DIR");

    fn get_path_to_test_files() -> PathBuf {
        let mut root_path = PathBuf::from(MAIN_DIR);
        root_path.push("test_files");

        root_path
    }

    #[test]
    fn test_with_test_files() {
        let test_path = get_path_to_test_files();

        println!("[page_test::test_with_test_files] reading from file path: {test_path:#?}");

        let page_root = Page::from_dir(&test_path);

        assert!(
            page_root.is_some(),
            "Page should be able to parse with known correct directory structure!"
        );

        let page_root_unwrapped = page_root.unwrap();

        println!("[page_test::test_with_test_files] Page root result: {page_root_unwrapped:#?}");

        assert_eq!(
            page_root_unwrapped.child.len(),
            1,
            "There should be one page other than index!"
        );
        assert_eq!(
            page_root_unwrapped.path, "/",
            "Page root should always be '/'!"
        );
        assert!(
            !page_root_unwrapped.template.is_empty(),
            "Page root template should not be empty!"
        );

        // Asserted with the length before
        let blog_page = page_root_unwrapped.child.first().unwrap();

        assert_eq!(
            blog_page.child.len(),
            1,
            "There should exactly 1 child (content) of blog page!"
        );
        assert_eq!(
            blog_page.path, "/blog",
            "Blog page should have path of blog!"
        );

        // Asserted with the length before
        let test_content = blog_page.child.first().unwrap();

        let mut content_path = String::from("/blog/");
        content_path.push_str(test_content.content.metadata.slug.as_ref().unwrap());

        assert_eq!(
            test_content.path, content_path,
            "The path of a content node should be equal to the slug!"
        );
    }
}
