/// A mode for sharing a resource between multiple queues.
#[derive(Debug, Clone)]
pub enum SharingMode<I> {
    /// The resource is exclusive to a single queue.
    Exclusive,
    /// The resource is shared between multiple queue families.
    Concurrent(I),
}
