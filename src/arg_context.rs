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
        let next = args.iter().next()?;
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

    const ARG1: Arg<u32> = Arg {
        id: Some(line!()),
        seen_id: id::SingleVal::new(line!()).into(),
        max_values: 1,
        possible_values: &[],
    };
    const ARG2: Arg<u32> = Arg {
        id: Some(line!()),
        seen_id: id::SingleVal::new(line!()).into(),
        max_values: 1,
        possible_values: &[],
    };
    #[test]
    fn test_empty_arg_ctx() {
        let mut ctx = ArgsContext::<u32>::new(&[]);
        assert!(!ctx.has_seen_arg());
        assert!(ctx.next_arg().is_none());
        assert!(!ctx.has_seen_arg());
    }
    #[test]
    fn test_simple_arg_ctx() {
        let mut ctx = ArgsContext::new(&[ARG1, ARG2]);
        assert!(!ctx.has_seen_arg());
        assert_eq!(ctx.next_arg().unwrap().id, ARG1.id);
        assert!(ctx.has_seen_arg());
        assert_eq!(ctx.next_arg().unwrap().id, ARG2.id);
        assert!(ctx.next_arg().is_none());
    }

    const ARG3: Arg<u32> = Arg {
        id: Some(line!()),
        seen_id: id::MultiVal::new(line!()).into(),
        max_values: 2,
        possible_values: &[],
    };
    const ARG4: Arg<u32> = Arg {
        id: Some(line!()),
        seen_id: id::MultiVal::new(line!()).into(),
        max_values: 3,
        possible_values: &[],
    };
    #[test]
    fn test_var_arg_ctx() {
        let mut ctx = ArgsContext::new(&[ARG1, ARG3, ARG4]);
        assert!(!ctx.has_seen_arg());
        assert_eq!(ctx.next_arg().unwrap().id, ARG1.id);
        assert!(ctx.has_seen_arg());

        assert_eq!(ctx.next_arg().unwrap().id, ARG3.id);
        assert_eq!(ctx.next_arg().unwrap().id, ARG3.id);

        assert_eq!(ctx.next_arg().unwrap().id, ARG4.id);
        assert_eq!(ctx.next_arg().unwrap().id, ARG4.id);
        assert_eq!(ctx.next_arg().unwrap().id, ARG4.id);

        assert!(ctx.next_arg().is_none());
    }
}
