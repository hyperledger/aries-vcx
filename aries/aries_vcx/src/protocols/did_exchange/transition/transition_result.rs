// TODO: Somehow enforce using both
#[must_use]
#[derive(Debug)]
pub struct TransitionResult<T, U> {
    pub state: T,
    pub output: U,
}
