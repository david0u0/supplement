#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Error {
    DashNotAllowed,
    ConsecutiveDashes,
    TooManyEquals,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ParsedFlag<'a> {
    Empty,
    SingleDash,
    DoubleDash,
    Short {
        body: char,
        equal: Option<&'a str>,
    },
    MultiShort {
        body: &'a str,
        equal: Option<&'a str>,
    },
    Long {
        body: &'a str,
        equal: Option<&'a str>,
    },
    NotFlag,
    Error(Error),
}
impl<'a> ParsedFlag<'a> {
    fn is_valid_flag(s: &str, allow_single_dash: bool) -> Option<Error> {
        let s = &s[1..];
        let mut last_is_dash = false;
        let mut equal_appeard = false;
        for ch in s.chars() {
            if ch == '-' {
                if !allow_single_dash {
                    log::warn!("a dash appears when none is allowed");
                    return Some(Error::DashNotAllowed);
                }
                if last_is_dash {
                    log::warn!("consecutive dashes");
                    return Some(Error::ConsecutiveDashes);
                }
                last_is_dash = true;
                continue;
            }
            last_is_dash = false;

            if ch == '=' {
                if equal_appeard {
                    log::warn!("more than one `=`");
                    return Some(Error::TooManyEquals);
                }
                equal_appeard = true;
                continue;
            }

            // NOTE: we may want to check characters e.g. '/' or '#' shouldn't be allowed.
            // But in practice, it will probably just cause a "flag not found" error, so no need to bother
        }
        None
    }

    pub fn new(s: &'a str) -> Self {
        if s.is_empty() {
            return Self::Empty;
        }
        if !s.starts_with('-') {
            return Self::NotFlag;
        }
        match s.chars().nth(1) {
            None => Self::SingleDash,
            Some('-') => {
                if s.len() == 2 {
                    return Self::DoubleDash;
                }
                if let Some(err) = Self::is_valid_flag(s, true) {
                    return Self::Error(err);
                }
                let mut flag_part = &s[2..];
                let equal = if let Some(equal_pos) = flag_part.chars().position(|c| c == '=') {
                    let equal = &flag_part[equal_pos + 1..];
                    flag_part = &flag_part[..equal_pos];
                    Some(equal)
                } else {
                    None
                };

                Self::Long {
                    body: flag_part,
                    equal,
                }
            }
            _ => {
                let flag_part = &s[1..];
                if let Some(err) = Self::is_valid_flag(s, false) {
                    return Self::Error(err);
                }
                if flag_part.len() == 1 {
                    Self::Short {
                        body: flag_part.chars().nth(0).unwrap(),
                        equal: None,
                    }
                } else {
                    if let Some(equal_pos) = flag_part.chars().position(|c| c == '=') {
                        let equal = Some(&flag_part[equal_pos + 1..]);
                        if equal_pos == 1 {
                            Self::Short {
                                body: flag_part.chars().nth(0).unwrap(),
                                equal,
                            }
                        } else {
                            Self::MultiShort {
                                body: &flag_part[..equal_pos],
                                equal,
                            }
                        }
                    } else {
                        Self::MultiShort {
                            body: flag_part,
                            equal: None,
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple() {
        assert_eq!(ParsedFlag::new(""), ParsedFlag::Empty);
        assert_eq!(ParsedFlag::new("-"), ParsedFlag::SingleDash);
        assert_eq!(ParsedFlag::new("--"), ParsedFlag::DoubleDash);
        assert_eq!(
            ParsedFlag::new("-s"),
            ParsedFlag::Short {
                body: 's',
                equal: None
            }
        );
        assert_eq!(
            ParsedFlag::new("--long"),
            ParsedFlag::Long {
                body: "long",
                equal: None
            }
        );
        assert_eq!(
            ParsedFlag::new("-long"),
            ParsedFlag::MultiShort {
                body: "long",
                equal: None
            }
        );

        assert_eq!(
            ParsedFlag::new("--l"),
            ParsedFlag::Long {
                body: "l",
                equal: None
            },
            "This may seem strange, but I don't want to be too strict. It probably will not find the flag anyways"
        );
    }

    #[test]
    fn test_equal() {
        assert_eq!(
            ParsedFlag::new("-s="),
            ParsedFlag::Short {
                body: 's',
                equal: Some("")
            }
        );
        assert_eq!(
            ParsedFlag::new("--long=x"),
            ParsedFlag::Long {
                body: "long",
                equal: Some("x")
            }
        );
        assert_eq!(
            ParsedFlag::new("-long=x"),
            ParsedFlag::MultiShort {
                body: "long",
                equal: Some("x")
            }
        );

        assert_eq!(
            ParsedFlag::new("--l=x"),
            ParsedFlag::Long {
                body: "l",
                equal: Some("x")
            },
            "This may seem strange, but I don't want to be too strict. It probably will not find the flag anyways"
        );
    }

    #[test]
    fn test_invalid() {
        use Error::*;

        assert_eq!(ParsedFlag::new("-s-b"), ParsedFlag::Error(DashNotAllowed));
        assert_eq!(
            ParsedFlag::new("---long"),
            ParsedFlag::Error(ConsecutiveDashes)
        );
        assert_eq!(
            ParsedFlag::new("--long--and"),
            ParsedFlag::Error(ConsecutiveDashes)
        );
        assert_eq!(ParsedFlag::new("-a=b=c"), ParsedFlag::Error(TooManyEquals));
    }
}
