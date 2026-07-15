use crate::constants::strings::*;

pub fn parse_clean_flag() -> bool {
    std::env::args().any(|a| a == CLI_ARG_CLEAN || a == CLI_ARG_CLEAN_SHORT)
}

pub fn parse_auto_approve_flag() -> bool {
    std::env::args().any(|a| a == CLI_ARG_AUTO_APPROVE || a == CLI_ARG_AUTO_APPROVE_SHORT)
}

pub fn unescape_message(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('0') => result.push('\0'),
                Some(c) => {
                    result.push('\\');
                    result.push(c);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}
