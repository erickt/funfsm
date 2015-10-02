use fsm::{Fsm, FsmContext, FsmHandler};
use channel::Msg;
use local_fsm::LocalFsm;
use constraints::Constraints;

pub struct Checker<T: FsmHandler> {
    fsm: LocalFsm<T>,
    constraints: Constraints<T::Context>
}

impl<T: FsmHandler> Checker<T> {
    pub fn new(constraints: Constraints<T::Context>) -> Checker<T> {
        Checker {
            fsm: LocalFsm::<T>::new(),
            constraints: constraints
        }
    }

    // TODO: Use quickcheck and a generator for messages here
    pub fn check(&mut self, msgs: Vec<Msg>) -> Result<(), String> {
        for msg in msgs {
            let (from, ctx) = self.fsm.get_state();
            try!(self.constraints.check_preconditions(from, &ctx));
            self.fsm.send_msg(msg);
            let (to, ctx) = self.fsm.get_state();
            try!(self.constraints.check_postconditions(from, &ctx));
            try!(self.constraints.check_invariants(&ctx));
            try!(self.constraints.check_transitions(from, to, &ctx));
        }
        Ok(())
    }
}
