use anyhow::anyhow;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl TryFrom<String> for SubscriberName {
    type Error = anyhow::Error;

    fn try_from(value: String) -> anyhow::Result<Self> {
        let is_empty_or_whitespace = value.trim().is_empty();
        if is_empty_or_whitespace {
            return Err(anyhow!("Subscriber name cannot be empty."));
        }

        let is_too_long = value.graphemes(true).count() > 256;
        if is_too_long {
            return Err(anyhow!(
                "Subscriber name cannot be longer than 256 characters."
            ));
        }

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters =
            value.chars().any(|g| forbidden_characters.contains(&g));
        if contains_forbidden_characters {
            return Err(anyhow!(
                "Subscriber name cannot contain the following characters: {:?}",
                forbidden_characters
            ));
        }

        Ok(Self(value))
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::subscriber_name::SubscriberName;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "ë".repeat(256);
        assert_ok!(SubscriberName::try_from(name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "ë".repeat(257);
        assert_err!(SubscriberName::try_from(name));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(SubscriberName::try_from(name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::try_from(name));
    }

    #[test]
    fn names_with_invalid_characters_are_rejected() {
        for name in &[")", "(", "\"", "\\", "<", ">", "{", "}", "/", "ë>"] {
            assert_err!(SubscriberName::try_from(name.to_string()));
        }
    }

    #[test]
    fn valid_names_are_parsed_successfully() {
        let name = "Zoë O'Connor".to_string();
        assert_ok!(SubscriberName::try_from(name));
    }
}
