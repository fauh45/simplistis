use serde::Deserialize;

#[derive(Default, Debug, Deserialize, PartialEq)]
pub struct ContentMetadata {
    pub(crate) title: Option<String>,
    pub(crate) slug: Option<String>,
    pub(crate) author: Option<String>,
    pub(crate) tags: Option<Vec<String>>,
    #[serde(with = "toml_datetime_compat", default = "chrono::Utc::now")]
    pub(crate) updated_at: chrono::DateTime<chrono::Utc>,
}

impl ContentMetadata {
    /// This function will process the raw content template content, then returns the metadata parsed
    /// from the file and returns the template without the metadata header.
    pub fn preprocess_content_metadata(content_md: String, file_name: String) -> (Self, String) {
        let mut is_have_metadata_lines = true;
        // Start by making sure that all whitespace or new-lines are removed at both end
        let mut remaining_content: String = content_md.trim().to_owned();
        let mut parsed_metadata = Self::default();

        let mut content_lines = remaining_content.lines();
        let first_line = content_lines.next();

        // If the first line of the content is not "---" just take it as non-existent
        if first_line.is_none() || first_line.is_some_and(|line| line != "---" || line.is_empty()) {
            is_have_metadata_lines = false;
        }

        if is_have_metadata_lines {
            let mut metadata_toml = String::new();

            for line in content_lines.by_ref() {
                // WARN: Need to add condition by EOF or something else, as there could be a case where
                // the file starts with the header "---" but does not close. Though it might just be a
                // "feature" to make sure that the user knows there's something wrong with the file,
                // and simplistis doesn't leak out their metadata.
                if line.is_empty() || line == "---" {
                    break;
                }

                metadata_toml.push_str(line);
                // This would make all CRLF file LF, though should be all fine right?
                metadata_toml.push('\n');
            }

            println!(
                "[ContentMetadata::preprocess_content_metadata] Metadata field:\n{metadata_toml}"
            );

            parsed_metadata = toml::from_str(&metadata_toml).unwrap_or(ContentMetadata::default());
            remaining_content = content_lines.collect::<Vec<&str>>().join("\n");
        }

        println!(
            "[ContentMetadata::preprocess_content_metadata] Parsed metadata:\n{parsed_metadata:#?}"
        );

        // Default value handling
        // NOTE: I'm not sure if it's the best way to do it, might need to
        // see it again at another time if there's a better way to do it,
        // as there should always be slug (the file name itself)
        // TODO: Make this better somehow?
        if parsed_metadata.slug.is_none() {
            println!("[ContentMetadata::preprocess_content_metadata] No slug, setting it with file_name:\n{file_name}");
            parsed_metadata.slug = Some(file_name.clone());
        }

        if parsed_metadata.title.is_none() {
            println!("[ContentMetadata::preprocess_content_metadata] No title, setting it with file_name:\n{file_name}");
            parsed_metadata.title = Some(file_name.clone())
        }

        // Also clean the returned value in case there's hanging spaces or new lines in the
        // remaining_content
        (parsed_metadata, remaining_content.trim().to_owned())
    }
}

#[cfg(test)]
mod preprocess_test {
    use crate::preparser::ContentMetadata;

    #[test]
    fn metadata_parsed() {
        let test_data = r#"---
title = "Halo!"
---

# Rest of it"#;

        let (metadata, rest_of_test_data) =
            ContentMetadata::preprocess_content_metadata(test_data.into(), "test-slug".into());

        assert_eq!(
            metadata.title,
            Some("Halo!".into()),
            "Failed to parse TOML!"
        );
        assert_eq!(
            metadata.author, None,
            "Failed to parse non non-existent field!"
        );
        assert_eq!(
            metadata.tags, None,
            "Failed to parse non non-existent field!"
        );
        assert_eq!(
            rest_of_test_data, "# Rest of it",
            "Failed to return the rest of the data!"
        );
    }

    #[test]
    fn empty_metadata_ignored() {
        let test_data = "# This is just a normal file right?\n\nYep definitely!";
        let test_slug: String = "test-empty-slug".into();

        let (metadata, rest_of_test_data) =
            ContentMetadata::preprocess_content_metadata(test_data.into(), test_slug.clone());

        println!("Output Metadata: {metadata:#?}");

        assert_eq!(
            metadata.title,
            Some(test_slug.clone()),
            "Empty metadata should use slug in the title!"
        );
        assert_eq!(
            metadata.slug,
            Some(test_slug.clone()),
            "Empty metadata should use file name as slug!"
        );
        assert_eq!(
            rest_of_test_data, test_data,
            "Failed to ignore metadata header!"
        );
    }
}
