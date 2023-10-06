use std::{collections::HashMap, fmt::Debug};

use tokio::sync::mpsc;

pub struct MpscRegistry<T: Debug> {
    sending: HashMap<String, mpsc::Sender<T>>,
    receiving: HashMap<String, mpsc::Receiver<T>>,
}

impl<T: Debug> Default for MpscRegistry<T> {
    fn default() -> Self {
        Self {
            sending: HashMap::new(),
            receiving: HashMap::new(),
        }
    }
}

impl<T: Debug> MpscRegistry<T> {
    pub async fn register_receiver(&mut self, name: String, receiver: mpsc::Receiver<T>) {
        self.receiving.insert(name, receiver);
    }

    pub async fn register_sender(&mut self, name: String, sender: mpsc::Sender<T>) {
        self.sending.insert(name.clone(), sender);
    }

    pub async fn send_msg(&self, name: &str, message: T) {
        if let Some(sender) = self.sending.get(name) {
            sender
                .send(message)
                .await
                .expect("Channel is probably closed.")
        } else {
            panic!("Relationship {} not found.", name);
        }
    }

    pub async fn receive_msg(&mut self, name: &str) -> T {
        if let Some(receiver) = self.receiving.get_mut(name) {
            receiver.recv().await.expect("Channel is probably closed.")
        } else {
            panic!("Relationship {} not found.", name);
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::MpscRegistry;

    #[tokio::test]
    async fn subscribe() {
        let alice = "alice";
        let bob = "bob";
        let alice_x_bob = "alice_x_bob".to_string();
        let bob_x_alice = "bob_x_alice".to_string();

        let (sender_bob_to_alice, receiver_alice_from_bob) = mpsc::channel::<String>(1);
        let (sender_alice_to_bob, receiver_bob_from_alice) = mpsc::channel::<String>(1);

        let mut registry = MpscRegistry::default();
        registry
            .register_sender(alice_x_bob.clone(), sender_alice_to_bob)
            .await;
        registry
            .register_receiver(bob_x_alice.clone(), receiver_alice_from_bob)
            .await;
        registry
            .register_sender(alice_x_bob.clone(), sender_bob_to_alice)
            .await;
        registry
            .register_receiver(bob_x_alice.clone(), receiver_bob_from_alice)
            .await;

        let msg1 = "Hello".to_string();
        let msg2 = "World".to_string();

        registry.send_msg(&alice_x_bob, msg1.clone()).await;
        let res = registry.receive_msg(&bob_x_alice).await;
        assert_eq!(res, msg1);
    }
}
