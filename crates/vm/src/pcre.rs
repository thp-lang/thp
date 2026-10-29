//! The bundled PCRE2 pattern and replacement operations.

use pcre2::bytes::{Captures, Regex, RegexBuilder};

pub fn compile(pattern: &[u8]) -> Result<Regex, String> {
    let pattern =
        std::str::from_utf8(pattern).map_err(|_| "regex pattern must be UTF-8".to_owned())?;
    let delimiter = pattern.chars().next().ok_or("regex pattern is empty")?;
    if !matches!(delimiter, '/' | '#' | '~') {
        return Err("regex pattern must use /, #, or ~ delimiters".to_owned());
    }
    let mut closing = None;
    let mut escaped = false;
    for (offset, character) in pattern.char_indices().skip(1) {
        if character == delimiter && !escaped {
            closing = Some(offset);
            break;
        }
        if character == '\\' {
            escaped = !escaped;
        } else {
            escaped = false;
        }
    }
    let closing = closing.ok_or("regex pattern has no closing delimiter")?;
    let body = &pattern[delimiter.len_utf8()..closing];
    let modifiers = &pattern[closing + delimiter.len_utf8()..];
    let mut builder = RegexBuilder::new();
    for modifier in modifiers.chars() {
        match modifier {
            'i' => {
                builder.caseless(true);
            }
            'm' => {
                builder.multi_line(true);
            }
            's' => {
                builder.dotall(true);
            }
            'x' => {
                builder.extended(true);
            }
            'u' => {
                builder.utf(true).ucp(true);
            }
            _ => return Err(format!("unsupported regex modifier `{modifier}`")),
        }
    }
    builder
        .build(body)
        .map_err(|error| format!("invalid PCRE2 pattern: {error}"))
}

pub fn replace(
    regex: &Regex,
    replacement: &[u8],
    subject: &[u8],
    limit: i64,
) -> Result<Vec<u8>, String> {
    if limit < -1 {
        return Err("regex replacement limit must be -1 or non-negative".to_owned());
    }
    let mut output = Vec::new();
    let mut previous = 0;
    for (count, capture) in regex.captures_iter(subject).enumerate() {
        if limit >= 0 && count >= usize::try_from(limit).unwrap_or(usize::MAX) {
            break;
        }
        let capture = capture.map_err(|error| format!("PCRE2 match failed: {error}"))?;
        let whole = capture.get(0).expect("PCRE2 includes the whole match");
        output.extend_from_slice(&subject[previous..whole.start()]);
        expand_replacement(replacement, &capture, &mut output);
        previous = whole.end();
    }
    output.extend_from_slice(&subject[previous..]);
    Ok(output)
}

fn expand_replacement(replacement: &[u8], captures: &Captures<'_>, output: &mut Vec<u8>) {
    let mut position = 0;
    while position < replacement.len() {
        if replacement[position] != b'$' {
            output.push(replacement[position]);
            position += 1;
            continue;
        }
        if replacement.get(position + 1) == Some(&b'$') {
            output.push(b'$');
            position += 2;
            continue;
        }
        let start = position + 1;
        if replacement.get(start) == Some(&b'{')
            && let Some(end) = replacement[start + 1..]
                .iter()
                .position(|byte| *byte == b'}')
        {
            let end = start + 1 + end;
            if let Ok(name) = std::str::from_utf8(&replacement[start + 1..end]) {
                if let Some(value) = captures.name(name) {
                    output.extend_from_slice(value.as_bytes());
                }
                position = end + 1;
                continue;
            }
        }
        let digits = replacement[start..]
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
        if digits > 0 {
            if let Ok(index) = std::str::from_utf8(&replacement[start..start + digits])
                .unwrap_or("")
                .parse::<usize>()
                && let Some(value) = captures.get(index)
            {
                output.extend_from_slice(value.as_bytes());
            }
            position = start + digits;
            continue;
        }
        output.push(b'$');
        position += 1;
    }
}

pub fn split(
    regex: &Regex,
    subject: &[u8],
    limit: i64,
    flags: i64,
) -> Result<Vec<(Vec<u8>, usize)>, String> {
    if limit < -1 {
        return Err("regex split limit must be -1 or non-negative".to_owned());
    }
    let mut output = Vec::new();
    let mut previous = 0;
    for (count, capture) in regex.captures_iter(subject).enumerate() {
        if limit >= 0 && count >= usize::try_from(limit.saturating_sub(1)).unwrap_or(usize::MAX) {
            break;
        }
        let capture = capture.map_err(|error| format!("PCRE2 match failed: {error}"))?;
        let whole = capture.get(0).expect("PCRE2 includes the whole match");
        let part = &subject[previous..whole.start()];
        if !part.is_empty() || flags & 1 == 0 {
            output.push((part.to_vec(), previous));
        }
        if flags & 2 != 0 {
            for index in 1..capture.len() {
                if let Some(group) = capture.get(index)
                    && (!group.as_bytes().is_empty() || flags & 1 == 0)
                {
                    output.push((group.as_bytes().to_vec(), group.start()));
                }
            }
        }
        previous = whole.end();
    }
    let tail = &subject[previous..];
    if !tail.is_empty() || flags & 1 == 0 {
        output.push((tail.to_vec(), previous));
    }
    Ok(output)
}
