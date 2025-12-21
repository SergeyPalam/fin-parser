pub fn remove_quotes(input: &str) -> String {
    input.trim_matches('"').to_owned()
}
