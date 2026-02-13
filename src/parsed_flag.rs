#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ParsedFlag<'a> {
    Empty,
    SingleDash,
    DoubleDash,
    Shorts,
    Long {
        body: &'a str,
        equal: Option<&'a str>,
    },
    NotFlag,
}
impl<'a> ParsedFlag<'a> {
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
            _ => Self::Shorts,
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
        assert_eq!(ParsedFlag::new("-s"), ParsedFlag::Shorts);
        assert_eq!(
            ParsedFlag::new("--long"),
            ParsedFlag::Long {
                body: "long",
                equal: None
            }
        );
        assert_eq!(ParsedFlag::new("-long"), ParsedFlag::Shorts);

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
        assert_eq!(ParsedFlag::new("-s="), ParsedFlag::Shorts);
        assert_eq!(
            ParsedFlag::new("--long=x"),
            ParsedFlag::Long {
                body: "long",
                equal: Some("x")
            }
        );
        assert_eq!(ParsedFlag::new("-long=x=b"), ParsedFlag::Shorts);

        assert_eq!(
            ParsedFlag::new("--l=x"),
            ParsedFlag::Long {
                body: "l",
                equal: Some("x")
            },
            "This may seem strange, but I don't want to be too strict. It probably will not find the flag anyways"
        );
    }
}
