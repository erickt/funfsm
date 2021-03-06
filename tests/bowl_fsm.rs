//! This is a simple example of a fsm for feeding a cat. The states of the fsm are the states of
//! the cat food bowl. Our cat is very whiny and will always be fed when her bowl is empty and she
//! meows. If there is already food in the bowl, she will have to eat it before we give her more.

#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]

#[macro_use]
extern crate funfsm;

#[macro_use]
extern crate assert_matches;

use funfsm::{Fsm, StateFn, FsmTypes};
use funfsm::constraints::Constraints;
use funfsm::constraints;
use funfsm::fsm_check::Checker;

const MAX_RESERVES: u8 = 10;
const REFILL_THRESHOLD: u8 = 9;

// Currently the pub members exist because constraint checking happens outside the impl
// TODO: Do we move the constraints in?
#[derive(Debug, Clone, Default)]
pub struct Context {
    pub contents: u8, // % of the bowl that is full
    pub reserves: u8, // The amount of bowls of food left in the bag
}

impl Context {
    pub fn new() -> Context {
        Context {
            contents: 0, // The bowl starts off empty
            reserves: MAX_RESERVES,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CatMsg {
    Meow,
    Eat(u8) // % of food to eat
}

#[derive(Debug, Clone)]
pub enum StoreReq {
    Buy(u8)
}

#[derive(Debug, Clone)]
pub enum StoreRpy {
    Bowls(u8)
}

#[derive(Debug, Clone)]
pub enum BowlMsg {
    CatMsg(CatMsg),
    StoreRpy(StoreRpy)
}

#[derive(Debug)]
pub struct BowlTypes;

impl FsmTypes for BowlTypes {
    type Context = Context;
    type Msg = BowlMsg;
    type Output = StoreReq;
}

pub fn empty(ctx: &mut Context, msg: BowlMsg) -> (StateFn<BowlTypes>, Vec<StoreReq>) {
    if let BowlMsg::CatMsg(CatMsg::Meow) = msg {
        if ctx.reserves > 0 {
            // Fill the bowl
            ctx.contents = 100;
            ctx.reserves -= 1;
            if ctx.reserves <= REFILL_THRESHOLD {
                let output = vec![StoreReq::Buy(10)];
                return next!(full, output);
            }
            return next!(full);
        } else {
            return next!(empty);
        }
    }

    if let BowlMsg::StoreRpy(StoreRpy::Bowls(num)) = msg {
        ctx.reserves += num-1;
        ctx.contents = 100;
        return next!(full);
    }

    next!(empty)
}

pub fn full(ctx: &mut Context, msg: BowlMsg) -> (StateFn<BowlTypes>, Vec<StoreReq>) {
    if let BowlMsg::CatMsg(CatMsg::Eat(pct)) = msg {
        if pct >= ctx.contents {
            ctx.contents = 0;
            return next!(empty)
        } else {
            ctx.contents -= pct;
            return next!(full)
        }
    }

    if let BowlMsg::StoreRpy(StoreRpy::Bowls(num)) = msg {
        ctx.reserves += num;
    }
    next!(full)
}

#[test]
/// Note the blocks are to reduce the borrow window for `&ctx` returned from `fsm.get_state()`.
fn test_state_transitions() {
    let mut fsm = Fsm::<BowlTypes>::new(Context::new(), state_fn!(empty));
    {
        let (name, ctx) = fsm.get_state();
        assert_eq!(name, "empty");
        assert_eq!(ctx.contents, 0);
    }
    fsm.send(BowlMsg::CatMsg(CatMsg::Meow));
   {
        let (name, ctx) = fsm.get_state();
        assert_eq!(name, "full");
        assert_eq!(ctx.contents, 100);
    }
    fsm.send(BowlMsg::CatMsg(CatMsg::Eat(30)));
    {
        let (name, ctx) = fsm.get_state();
        assert_eq!(name, "full");
        assert_eq!(ctx.contents, 70);
    }
    fsm.send(BowlMsg::CatMsg(CatMsg::Meow));
    {
        let (name, ctx) = fsm.get_state();
        assert_eq!(name, "full");
        assert_eq!(ctx.contents, 70);
    }
    fsm.send(BowlMsg::CatMsg(CatMsg::Eat(75)));
    {
        let (name, ctx) = fsm.get_state();
        assert_eq!(name, "empty");
        assert_eq!(ctx.contents, 0);
    }
}

#[test]
fn test_check() {
    let msgs = vec![BowlMsg::CatMsg(CatMsg::Meow),
                 BowlMsg::CatMsg(CatMsg::Eat(30)),
                 BowlMsg::CatMsg(CatMsg::Eat(70)),
                 BowlMsg::CatMsg(CatMsg::Meow),
                 BowlMsg::CatMsg(CatMsg::Eat(50)),
                 BowlMsg::CatMsg(CatMsg::Meow)];
    check_constraints(msgs);
}

fn check_constraints(msgs: Vec<BowlMsg>) {
    let mut c = Constraints::new();
    precondition!(c, "empty", |ctx: &Context| ctx.contents == 0);
    precondition!(c, "full", |ctx: &Context| ctx.contents > 0 && ctx.contents <= 100);

    invariant!(c, |ctx: &Context| ctx.contents <= 100);

    transition!(c, "empty" => "full", empty_to_full);
    transition!(c, "full" => "empty", full_to_empty);

    let mut checker = Checker::<BowlTypes>::new(Context::new(), state_fn!(empty), c);
    for msg in msgs {
        assert_matches!(checker.check(msg), Ok(_));
    }
}

#[allow(unused_must_use)]
fn empty_to_full(init_ctx: &Context,
                 final_ctx: &Context,
                 msg: &BowlMsg,
                 _output: &[StoreReq]) -> Result<(), String>
{
   let s = "Transition from empty to full";
   check!(s, init_ctx.contents == 0);
   check!(s, final_ctx.contents == 100);
   check!(s, match *msg {
       BowlMsg::StoreRpy(_) | BowlMsg::CatMsg(CatMsg::Meow) => true,
       _ => false
   });
   Ok(())
}

#[allow(unused_must_use)]
fn full_to_empty(init_ctx: &Context,
                 final_ctx: &Context,
                 msg: &BowlMsg,
                 _output: &[StoreReq]) -> Result<(), String>
{
    let s = "Transition from full to empty";
    check!(s, init_ctx.contents > 0);
    check!(s, final_ctx.contents == 0);
    check!(s, { if let BowlMsg::CatMsg(CatMsg::Eat(_)) = *msg {
        true
    } else {
        false
    }});
   Ok(())
}
