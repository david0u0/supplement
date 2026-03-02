use crate::core::Arg;

pub(crate) struct ArgsContext<'a, ID> {
    args: &'a [Arg<ID>],
    cur_arg_values_count: usize,
    start_idx: usize,
}
impl<'a, ID> ArgsContext<'a, ID> {
    pub fn new(args: &'a [Arg<ID>]) -> Self {
        Self {
            args,
            start_idx: 0,
            cur_arg_values_count: 0,
        }
    }
    pub fn has_seen_arg(&self) -> bool {
        self.start_idx != 0 || self.cur_arg_values_count != 0
    }
    pub fn next_arg(&mut self) -> Option<&Arg<ID>> {
        log::debug!("next arg called");
        let args = &self.args[self.start_idx..];
        let Some(next) = args.iter().next() else {
            return None;
        };
        if next.max_values == self.cur_arg_values_count + 1 {
            self.start_idx += 1;
            self.cur_arg_values_count = 0;
        } else {
            self.cur_arg_values_count += 1;
        }

        Some(next)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::id;

    const ARG1: Arg<()> = Arg {
        id: id::Valued::Single(id::SingleVal::new(())),
        max_values: 1,
    };
    const ARG2: Arg<()> = Arg {
        id: id::Valued::Single(id::SingleVal::new(())),
        max_values: 1,
    };
    #[test]
    fn test_empty_arg_ctx() {
        let mut ctx = ArgsContext::<()>::new(&[]);
        assert_eq!(ctx.has_seen_arg(), false);
        assert!(ctx.next_arg().is_none());
        assert_eq!(ctx.has_seen_arg(), false);
    }
    #[test]
    fn test_simple_arg_ctx() {
        let mut ctx = ArgsContext::new(&[ARG1, ARG2]);
        assert_eq!(ctx.has_seen_arg(), false);
        assert_eq!(ctx.next_arg().unwrap().id, ARG1.id);
        assert_eq!(ctx.has_seen_arg(), true);
        assert_eq!(ctx.next_arg().unwrap().id, ARG2.id);
        assert!(ctx.next_arg().is_none());
    }

    const ARG3: Arg<()> = Arg {
        id: id::Valued::Single(id::SingleVal::new(())),
        max_values: 2,
    };
    const ARG4: Arg<()> = Arg {
        id: id::Valued::Single(id::SingleVal::new(())),
        max_values: 3,
    };
    #[test]
    fn test_var_arg_ctx() {
        let mut ctx = ArgsContext::new(&[ARG1, ARG3, ARG4]);
        assert_eq!(ctx.has_seen_arg(), false);
        assert_eq!(ctx.next_arg().unwrap().id, ARG1.id);
        assert_eq!(ctx.has_seen_arg(), true);

        assert_eq!(ctx.next_arg().unwrap().id, ARG3.id);
        assert_eq!(ctx.next_arg().unwrap().id, ARG3.id);

        assert_eq!(ctx.next_arg().unwrap().id, ARG4.id);
        assert_eq!(ctx.next_arg().unwrap().id, ARG4.id);
        assert_eq!(ctx.next_arg().unwrap().id, ARG4.id);

        assert!(ctx.next_arg().is_none());
    }
}
