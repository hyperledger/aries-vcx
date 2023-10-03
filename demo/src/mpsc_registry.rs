use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct MpscRegistry<T: Debug> {
    sending: Arc<RwLock<HashMap<String, mpsc::Sender<T>>>>,
    receiving: Arc<RwLock<HashMap<String, mpsc::Receiver<T>>>>,
}

impl<T: Debug> MpscRegistry<T> {
    pub fn new() -> MpscRegistry<T> {
        MpscRegistry {
            sending: Arc::new(Default::default()),
            receiving: Arc::new(Default::default()),
        }
    }

    pub async fn register_sender(&self, relationship_key: String, sender: mpsc::Sender<T>) {
        let mut sending_lock = self.sending.write().await;
        sending_lock.insert(relationship_key, sender);
    }

    pub async fn register_receiver(&self, relationship_key: String, receiver: mpsc::Receiver<T>) {
        let mut receiving_lock = self.receiving.write().await;
        receiving_lock.insert(relationship_key, receiver);
    }

    // todo: don't unwrap, return result
    pub async fn send_msg(&self, relationship_key: &str, message: T) {
        let inner = self.sending.read().await;
        if let Some(sender) = inner.get(relationship_key) {
            sender.send(message).await.expect("Channel is probably closed.")
        } else {
            panic!("Relationship {} not found.", relationship_key);
        }
    }

    pub async fn receive_msg(&self, relationship_key: &str) -> T {
        let mut lock = self.receiving.write().await;
        if let Some(receiver) = lock.get_mut(relationship_key) {
            receiver.recv().await.expect("Channel is probably closed.")
        } else {
            panic!("Relationship {} not found.", relationship_key);
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::MpscRegistry;
//
//     #[tokio::test]
//     async fn subscribe() {
//         let alice = "alice";
//         let john = "john";
//         let registry = MpscRegistry::new();
//         let receiver_john = registry.create_channel(john).await;
//         let receiver_alice = registry.create_channel(alice).await;
//         let data_alice = vec![1, 1, 1];
//         let data_john = vec![2, 2, 2];
//
//         registry.send_msg(alice, data_alice.clone()).await;
//
//         let res = receiver_john.lock().await.try_recv();
//         assert!(res.is_err());
//         let res = receiver_alice.lock().await.try_recv();
//         assert_eq!(res.unwrap(), data_alice);
//
//         registry.send_msg(john, data_john.clone()).await;
//         let res = receiver_john.lock().await.try_recv();
//         assert_eq!(res.unwrap(), data_john);
//         let res = receiver_alice.lock().await.try_recv();
//         assert!(res.is_err());
//     }
// }
