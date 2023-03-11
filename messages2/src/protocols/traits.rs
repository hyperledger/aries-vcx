pub trait ConcreteMessage {
    type Kind;

    fn kind() -> Self::Kind;
}
