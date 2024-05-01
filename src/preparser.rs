use serde::Deserialize;

#[derive(Default, Debug, Deserialize, PartialEq)]
pub struct ContentMetadata {
    title: Option<String>,
    author: Option<String>,
    tags: Option<Vec<String>>,
}

/// This function will process the raw content template content, then returns the metadata parsed
/// from the file and returns the template without the metadata header.
pub fn preprocess_content_metadata(content_md: String) -> (ContentMetadata, String) {
    let mut content_lines = content_md.lines();

    // If the first line of the content is not "---" just take it as non-existent
    if Some("---") != content_lines.next() {
        return (ContentMetadata::default(), content_md);
    }

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

    println!("[Preprocessor] Metadata field:\n{metadata_toml}");

    let parsed_metadata: ContentMetadata =
        toml::from_str(&metadata_toml).unwrap_or(ContentMetadata::default());
    let remaining_line = content_lines.collect::<Vec<&str>>().join("\n");

    (parsed_metadata, remaining_line)
}

#[cfg(test)]
mod preprocess_test {
    use crate::preparser::ContentMetadata;

    use super::preprocess_content_metadata;

    #[test]
    fn metadata_parsed() {
        let test_data = "---\ntitle = \"Halo!\"\n---\n# Rest of it";

        let (metadata, rest_of_test_data) = preprocess_content_metadata(test_data.into());

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

        let (metadata, rest_of_test_data) = preprocess_content_metadata(test_data.into());

        assert_eq!(
            metadata,
            ContentMetadata::default(),
            "Failed to parse empty metadata!"
        );
        assert_eq!(
            rest_of_test_data, test_data,
            "Failed to ignore metadata header!"
        );
    }
}
