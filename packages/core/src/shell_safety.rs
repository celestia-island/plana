pub fn contains_shell_metacharacters(command: &str) -> bool {
    let mut chars = command.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                let mut found_close = false;
                for nc in chars.by_ref() {
                    if nc == '\'' {
                        found_close = true;
                        break;
                    }
                }
                if !found_close {
                    return true;
                }
            },
            '"' => {
                let mut found_close = false;
                while let Some(nc) = chars.next() {
                    if nc == '"' {
                        found_close = true;
                        break;
                    }
                    if nc == '`' {
                        return true;
                    }
                    if nc == '$'
                        && let Some(&next) = chars.peek()
                        && (next == '('
                            || next == '{'
                            || next.is_alphanumeric()
                            || next == '_'
                            || next == '$'
                            || next == '?'
                            || next == '!'
                            || next == '@'
                            || next == '*'
                            || next == '#'
                            || next == '-')
                    {
                        return true;
                    }
                    if nc == '\\' {
                        chars.next();
                    }
                }
                if !found_close {
                    return true;
                }
            },
            ';' | '|' | '&' | '(' | ')' | '\n' | '\r' => return true,
            '$' => {
                if let Some(&next) = chars.peek()
                    && (next == '('
                        || next == '{'
                        || next.is_alphanumeric()
                        || next == '_'
                        || next == '$'
                        || next == '?'
                        || next == '!'
                        || next == '@'
                        || next == '*'
                        || next == '#'
                        || next == '-'
                        || next == '\'')
                {
                    return true;
                }
            },
            '`' => return true,
            '>' | '<' => return true,
            _ => {},
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_injection() {
        assert!(contains_shell_metacharacters("ls; rm -rf /"));
        assert!(contains_shell_metacharacters(
            "cat /etc/hosts && echo pwned"
        ));
        assert!(contains_shell_metacharacters("echo $(cat /etc/passwd)"));
        assert!(contains_shell_metacharacters("echo ${IFS}rm"));
        assert!(contains_shell_metacharacters("echo $((1+1))"));
        assert!(contains_shell_metacharacters("ls `rm -rf /`"));
        assert!(contains_shell_metacharacters(
            "cat /etc/hosts | tee /tmp/out"
        ));
        assert!(contains_shell_metacharacters("ls > /tmp/out"));
        assert!(contains_shell_metacharacters("ls\nrm -rf /"));
        assert!(contains_shell_metacharacters("echo 'unclosed"));
        assert!(contains_shell_metacharacters("echo $HOME/.ssh/id_rsa"));
        assert!(contains_shell_metacharacters("echo $_SECRET"));
        assert!(contains_shell_metacharacters("echo $$"));
        assert!(contains_shell_metacharacters("echo $?"));
        assert!(contains_shell_metacharacters("echo $!"));
        assert!(contains_shell_metacharacters("echo $@"));
        assert!(contains_shell_metacharacters("echo \"$(cat /etc/passwd)\""));
        assert!(contains_shell_metacharacters("echo \"`hostname`\""));
        assert!(contains_shell_metacharacters("echo \"$HOME\""));
        assert!(contains_shell_metacharacters("(rm -rf /)"));
        assert!(contains_shell_metacharacters("eval $'\\x72\\x6d'"));
    }

    #[test]
    fn allows_safe_commands() {
        assert!(!contains_shell_metacharacters("ls -la /workspace"));
        assert!(!contains_shell_metacharacters("cat /etc/hosts"));
        assert!(!contains_shell_metacharacters("git log --oneline -10"));
        assert!(!contains_shell_metacharacters("grep -r pattern /src"));
        assert!(!contains_shell_metacharacters("echo hello world"));
        assert!(!contains_shell_metacharacters("echo 'hello world'"));
        assert!(!contains_shell_metacharacters("echo \"hello world\""));
        assert!(!contains_shell_metacharacters("echo '$(cat /etc/passwd)'"));
    }
}
