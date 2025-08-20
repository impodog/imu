use crate::*;
use core::sync::atomic::AtomicUsize;

pub struct Arc<T> {
    inner: NonNull<ArcInner<T>>,
    _phantom: core::marker::PhantomData<T>,
}

pub struct ArcInner<T> {
    value: T,
    arc: AtomicUsize,
}

// TODO: Implement arc
