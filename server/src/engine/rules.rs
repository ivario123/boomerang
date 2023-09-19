pub trait RuleSet {}
pub trait New {
    fn new() -> Self;
}
pub struct Austrailia<const CAPACITY: usize> {}

impl<const CAPACITY: usize> RuleSet for Austrailia<CAPACITY> {}
impl<const CAPACITY: usize> New for Austrailia<CAPACITY> {
    fn new() -> Self {
        Austrailia {}
    }
}
