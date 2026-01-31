#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    DashNotAllowed,
    ConsecutiveDashes,
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
    MultiShort(&'a str),
    Long {
        body: &'a str,
        equal: Option<&'a str>,
    },
    NotFlag,
}
impl<'a> ParsedFlag<'a> {
    fn validate(s: &str, allow_single_dash: bool) -> Result<(), Error> {
        let s = &s[1..];
        let mut last_is_dash = false;
        for ch in s.chars() {
            if ch == '-' {
                if !allow_single_dash {
                    return Err(Error::DashNotAllowed);
                }
                if last_is_dash {
                    return Err(Error::ConsecutiveDashes);
                }
                last_is_dash = true;
                continue;
            }
            last_is_dash = false;

            // NOTE: we may want to check characters e.g. '/' or '#' shouldn't be allowed.
            // But in practice, it will probably just cause a "flag not found" error, so no need to bother
        }
        Ok(())
    }

    pub fn new(s: &'a str) -> Result<Self, Error> {
        if s.is_empty() {
            return Ok(Self::Empty);
        }
        if !s.starts_with('-') {
            return Ok(Self::NotFlag);
        }
        let ret = match s.chars().nth(1) {
            None => Self::SingleDash,
            Some('-') => {
                if s.len() == 2 {
                    return Ok(Self::DoubleDash);
                }
                Self::validate(s, true)?;
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
                Self::validate(s, false)?;
                if flag_part.len() == 1 {
                    Self::Short {
                        body: flag_part.chars().nth(0).unwrap(),
                        equal: None,
                    }
                } else {
                    if flag_part.chars().nth(1) == Some('=') {
                        Self::Short {
                            body: flag_part.chars().nth(0).unwrap(),
                            equal: Some(&flag_part[2..]),
                        }
                    } else {
                        Self::MultiShort(flag_part)
                    }
                }
            }
        };
        Ok(ret)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple() {
        assert_eq!(ParsedFlag::new("").unwrap(), ParsedFlag::Empty);
        assert_eq!(ParsedFlag::new("-").unwrap(), ParsedFlag::SingleDash);
        assert_eq!(ParsedFlag::new("--").unwrap(), ParsedFlag::DoubleDash);
        assert_eq!(
            ParsedFlag::new("-s").unwrap(),
            ParsedFlag::Short {
                body: 's',
                equal: None
            }
        );
        assert_eq!(
            ParsedFlag::new("--long").unwrap(),
            ParsedFlag::Long {
                body: "long",
                equal: None
            }
        );
        assert_eq!(
            ParsedFlag::new("-long").unwrap(),
            ParsedFlag::MultiShort("long",)
        );

        assert_eq!(
            ParsedFlag::new("--l").unwrap(),
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
            ParsedFlag::new("-s=").unwrap(),
            ParsedFlag::Short {
                body: 's',
                equal: Some("")
            }
        );
        assert_eq!(
            ParsedFlag::new("--long=x").unwrap(),
            ParsedFlag::Long {
                body: "long",
                equal: Some("x")
            }
        );
        assert_eq!(
            ParsedFlag::new("-long=x=b").unwrap(),
            ParsedFlag::MultiShort("long=x=b",)
        );

        assert_eq!(
            ParsedFlag::new("--l=x").unwrap(),
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

        assert_eq!(ParsedFlag::new("-s-b"), Err(DashNotAllowed));
        assert_eq!(ParsedFlag::new("---long"), Err(ConsecutiveDashes));
        assert_eq!(ParsedFlag::new("--long--and"), Err(ConsecutiveDashes));
    }
}
