// rectiq-cli/src/utils/cow.rs
use std::borrow::Cow;

/// Converts an `Option<&str>` into an `Option<Cow<'_, str>>`, owning the string.
pub fn to_cow_opt(s: Option<&str>) -> Option<Cow<'_, str>> {
    s.map(Cow::Borrowed)
}
