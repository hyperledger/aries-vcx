// TODO: Somehow enforce using both
#[must_use]
pub struct TransitionResult<T, U> {
    pub state: T,
    pub output: U,
}
