pub fn lowercase_alphanumeric_only(text: &str) -> String {
    let normalized_text = String::from_iter(text
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(|c| c.to_lowercase()));

    return normalized_text;
}
