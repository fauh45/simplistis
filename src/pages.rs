use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use crate::content::Content;

#[derive(Debug)]
pub struct Page {
    pub(crate) path: String,
    content: Option<Content>,
    template: String,

    pub(crate) child: Vec<Page>,
}

impl Page {
    /// This function assume `path` is the root of a page.
    fn parse_one_page<P: AsRef<Path>>(path: &P) -> Option<Self> {
        let mut current_path = PathBuf::new();
        current_path.push(path.as_ref());

        let current_dir = current_path.file_name();

        if current_dir.is_none() {
            println!("[Page::parse_one_page] Cannot get dir name!");

            return None;
        }

        let mut template_file = current_path.clone();
        template_file.push("template.html");

        if !template_file.exists() {
            println!("[Page::parse_one_page] No template.html on the page!");

            return None;
        }

        let mut template_content = String::new();

        // Safe to unwrap as we check if path exist or not, though there could be problem
        // where the file is not allowed to be open by the user
        // TODO: Handle file open error
        if let Err(err) = (File::open(template_file))
            .unwrap()
            .read_to_string(&mut template_content)
        {
            println!("[Page::parse_one_page] Cannot open template.html file! Error: {err:#?}");

            return None;
        }

        let mut current_root = Self {
            path: current_dir.unwrap().to_os_string().into_string().unwrap(),
            content: None,
            template: template_content.clone(),
            child: vec![],
        };

        // Start contents search
        current_path.push("contents");

        if current_path.exists() {
            let contents = Content::from_dir(&current_path);

            for content in contents {
                current_root.child.push(Self {
                    // This should be safe as `slug` is guaranteed to always be there
                    path: content.metadata.slug.as_ref().unwrap().into(),
                    template: template_content.clone(),
                    content: Some(content),
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
    /// ........contents (:dir) ->
    /// ............(contents:file).md
    /// ........template.html (:file)
    /// ....templates.html (root page "/" template:file)
    /// ```
    pub fn from_dir<P: AsRef<Path>>(path: &P) -> Option<Self> {
        let mut index_template_path = PathBuf::new();
        index_template_path.push(path);
        index_template_path.push("template.html");

        if !index_template_path.exists() {
            println!("[Page::from_dir] Index template.html does not exist!");

            return None;
        }

        let mut index_template_content = String::new();
        // Safe to unwrap as we check if path exist or not, though there could be problem
        // where the file is not allowed to be open by the user
        // TODO: Handle file open error
        let mut index_template_file = File::open(index_template_path).unwrap();

        // TODO: Handle error while reading template
        index_template_file
            .read_to_string(&mut index_template_content)
            .unwrap();

        let mut root_page = Self {
            path: "/".into(),
            content: None,
            template: index_template_content,
            child: vec![],
        };

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();

                if entry.file_type().is_ok_and(|file_type| file_type.is_dir()) {
                    let current_page = Self::parse_one_page(&entry_path);

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
        assert!(
            page_root_unwrapped.content.is_none(),
            "There should not be any content on root!"
        );

        // Asserted with the length before
        let blog_page = page_root_unwrapped.child.first().unwrap();

        assert_eq!(
            blog_page.child.len(),
            1,
            "There should exactly 1 child (content) of blog page!"
        );
        assert_eq!(
            blog_page.path, "blog",
            "Blog page should have path of blog!"
        );
        assert!(
            blog_page.content.is_none(),
            "There should not be any content on the blog sub page!"
        );

        // Asserted with the length before
        let test_content = blog_page.child.first().unwrap();

        assert!(
            test_content.content.is_some(),
            "There should be some content on the test content page!"
        );

        // Asserted with `is_some` before
        let test_content_content = test_content.content.as_ref().unwrap();

        assert_eq!(
            test_content.path,
            test_content_content
                .metadata
                .slug
                .as_ref()
                .unwrap()
                .to_string(),
            "The path of a content node should be equal to the slug!"
        );
    }
}
