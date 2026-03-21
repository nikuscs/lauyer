/// Collapse multiple consecutive blank lines into one, strip leading/trailing
/// whitespace per line, remove residual HTML tags, and collapse internal runs
/// of whitespace to a single space.
pub fn compact_text(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut consecutive_blank = 0u32;

    for line in input.lines() {
        let stripped = strip_html_tags(line.trim());
        let collapsed = collapse_spaces(&stripped);

        if collapsed.is_empty() {
            consecutive_blank += 1;
            if consecutive_blank <= 1 {
                output.push('\n');
            }
        } else {
            consecutive_blank = 0;
            output.push_str(&collapsed);
            output.push('\n');
        }
    }

    // Strip leading/trailing blank lines from the result.
    let trimmed = output.trim().to_owned();

    // Remove well-known boilerplate lines from Portuguese legal documents.
    strip_boilerplate(&trimmed)
}

/// Remove basic HTML tags using a simple state machine (no regex dependency).
pub fn strip_html_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut inside_tag = false;

    for ch in input.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

/// Collapse runs of whitespace characters (space, tab) to a single space.
fn collapse_spaces(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_space = false;

    for ch in input.chars() {
        if ch == ' ' || ch == '\t' {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out.trim().to_owned()
}

/// Common Portuguese legal boilerplate header patterns to strip.
static BOILERPLATE_PREFIXES: &[&str] = &[
    "acordam no tribunal da relação",
    "acordam os juízes",
    "acordam na secção",
    "acordam, em conferência,",
    "acordam os senhores juízes",
];

/// Remove well-known boilerplate lines from Portuguese legal documents.
pub fn strip_boilerplate(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for line in input.lines() {
        let lower = line.to_lowercase();
        let is_boilerplate = BOILERPLATE_PREFIXES.iter().any(|prefix| lower.starts_with(prefix));
        if !is_boilerplate {
            out.push_str(line);
            out.push('\n');
        }
    }
    out.trim().to_owned()
}

/// Portuguese stop-words that are safe to remove.
static STOP_WORDS: &[&str] = &[
    "o", "a", "os", "as", "um", "uma", "uns", "umas", "de", "em", "por", "para", "com", "e", "ou",
    "que", "mas",
];

/// Words that must never be removed, even though they are short.
/// These carry legal meaning (negation / exclusion / restriction).
static NEVER_REMOVE: &[&str] = &[
    "não", "sem", "nem", "nunca", "nenhum", "nenhuma", "jamais", "salvo", "excepto", "apenas",
    "somente",
];

/// Remove stop-words from `input` in a word-boundary-aware manner.
/// Words in `NEVER_REMOVE` are always kept regardless of case.
pub fn strip_stopwords(input: &str) -> String {
    // Build the result word-by-word, preserving surrounding punctuation /
    // whitespace runs as-is.
    let mut out = String::with_capacity(input.len());
    // Walk character-by-character, collecting word runs.
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i].is_alphabetic() {
            // Collect a full word (including accented Unicode letters).
            let start = i;
            while i < len && chars[i].is_alphabetic() {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let lower = word.to_lowercase();

            // Keep if it is in the never-remove list.
            let protected = NEVER_REMOVE.contains(&lower.as_str());
            // Remove if it is a stop-word and not protected.
            let is_stop = !protected && STOP_WORDS.contains(&lower.as_str());

            if !is_stop {
                out.push_str(&word);
            }
            // If the word was removed we simply don't push it; the surrounding
            // space/punctuation that follows will be emitted in the next loop
            // iteration. This can leave a leading space — we tidy that after.
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }

    // Collapse double spaces that arise from removed words.
    collapse_spaces(out.trim())
}
