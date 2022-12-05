use async_trait::async_trait;

#[async_trait]
pub trait AsyncFnIterator : Send + Sync {
    type Item;
    
    async fn next(&mut self) -> Option<Self::Item>;
}