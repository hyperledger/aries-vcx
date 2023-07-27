use async_trait::async_trait;

use crate::errors::error::AriesVcxError;

use self::connection::ConnectionSM;

pub mod connection;

/// Enum that can represent any Aries state machine, in any of their states.
#[derive(Clone, Debug)]
pub enum AriesSM {
    Connection(ConnectionSM),
}

/// Interface for handling the storage and retrieval of [`AriesSM`].
#[async_trait]
pub trait StateMachineStorage {
    /// Type used for providing the necessary arguments from the environment
    /// to determine the [`Self::Id`] of the state machine that needs to be
    /// retrieved.
    type ResolveIdParams<'a>;
    /// Type is used for identifying a particular [`AriesSM`] instance.
    type Id;

    /// This method makes use of the parameters to provide a [`Self::Id`] that
    /// will then be used for retrieving and storing the state machine.
    async fn resolve_id(&self, id_params: Self::ResolveIdParams<'_>) -> Result<Self::Id, AriesVcxError>;

    /// Retrieves the state machine with the given id.
    /// This is intended to transfer the state machine's ownership, if possible.
    ///
    /// If, for instance, you (also) store your state machines in an
    /// in-memory cache, on a cache hit you should remove the instance
    /// from the cache and return the owned state machine (not clone it).
    ///
    /// Also see [`StateMachineStorage::put_different_state`] and [`StateMachineStorage::put_same_state`].
    async fn get(&self, id: &Self::Id) -> Result<Option<AriesSM>, AriesVcxError>;

    /// Used for storing a state machine in the event that its state *DID* change.
    /// This should update ALL places where you store your state machines.
    ///
    /// If, for instance, you store your state machines in a disk-based database
    /// and an in-memory cache, this should update both.
    ///
    /// Also see [`StateMachineStorage::get`] and [`StateMachineStorage::put_same_state`].
    async fn put_different_state(&self, id: Self::Id, sm: AriesSM) -> Result<(), AriesVcxError>;

    /// Used for storing a state machine in the event that its state *DID NOT* change.
    /// This is present to allow storage optimizations.
    ///
    /// If, for instance, you store your state machines in a disk-based database
    /// and an in-memory cache, this should ONLY update the in-memory cache.
    ///
    /// Also see [`StateMachineStorage::get`] and [`StateMachineStorage::put_different_state`].
    async fn put_same_state(&self, id: Self::Id, sm: AriesSM) -> Result<(), AriesVcxError>;
}
