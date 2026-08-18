pub fn slug(input: &str) -> String {
    let mut output = String::new();
    let mut previous_is_separator = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            output.push(ch.to_ascii_lowercase());
            previous_is_separator = false;
        } else if !previous_is_separator && !output.is_empty() {
            output.push('_');
            previous_is_separator = true;
        }
    }

    while output.ends_with('_') {
        output.pop();
    }

    output
}

pub fn kebab(input: &str) -> String {
    slug(input).replace('_', "-")
}

pub fn upper(input: &str) -> String {
    slug(input).to_ascii_uppercase()
}

pub fn title(input: &str) -> String {
    slug(input)
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut word = String::new();
                    word.push(first.to_ascii_uppercase());
                    word.push_str(chars.as_str());
                    word
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn pascal(input: &str) -> String {
    title(input).replace(' ', "")
}

pub fn is_c_identifier(input: &str) -> bool {
    let mut chars = input.chars();
    match chars.next() {
        Some(first) if first == '_' || first.is_ascii_alphabetic() => {}
        _ => return false,
    }

    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}
