pub mod receiver;
pub mod sender;

#[derive(Debug, Clone, PartialEq)]
pub enum GenericOutOfBand {
    Receiver(receiver::OutOfBandReceiver),
    Sender(sender::OutOfBandSender),
}
