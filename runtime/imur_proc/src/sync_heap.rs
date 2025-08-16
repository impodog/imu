use crate::*;
use core::pin::Pin;

/// A wrapper around the bare heap implementation,
/// providing thread safety around the heap using spin locks.
///
/// Spin locks provide fast access when there are few threads.
///
/// The must be pinned and initialized before use.
pub struct SyncHeap<'h, H: Memory>
where
    H: 'h,
{
    heap: imur_mem::heap::Heap<H>,
    ptr: HeapPtr<'h, H>,
    heap_lock: RwLock<()>,
    node_locks: Vec<RwLock<()>, HeapPtr<'h, H>>,
    _pinned: core::marker::PhantomPinned,
}

impl<'h, H: Memory> SyncHeap<'h, H> {
    /// Creates a new sync heap with standard heap args.
    ///
    /// You must pin this before use.
    pub fn new(mem: H, max_block: u8) -> Self {
        Self {
            heap: imur_mem::heap::Heap::<H>::new(mem, max_block),
            ptr: HeapPtr::null(),
            heap_lock: Default::default(),
            node_locks: Vec::new(HeapPtr::null()),
            _pinned: Default::default(),
        }
    }

    /// Initializes the heap, resolving self-reference and allocated internal heap free list.
    ///
    /// You must call this before any other actions, otherwise those methods will panic (implemented
    /// by internal heap).
    ///
    /// # Panics
    ///
    /// Panics if there is not enough memory for free list.
    pub fn init(self: Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        this.ptr.ptr = &mut this.heap as _;
        this.node_locks.replace_heap(this.ptr.clone());
        this.heap.init();
    }
}

/// A pointer reference to `imur_mem::heap::Heap`.
/// This is highly unsafe, and you must guarantee that the heap outlives this pointer,
/// and that it is pinned to one pointer.
struct HeapPtr<'h, H: Memory>
where
    H: 'h,
{
    ptr: *mut imur_mem::heap::Heap<H>,
    _phantom: core::marker::PhantomData<&'h H>,
}
impl<'h, H: Memory> Clone for HeapPtr<'h, H> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'h, H: Memory> Copy for HeapPtr<'h, H> {}

impl<'h, H: Memory> HeapPtr<'h, H> {
    /// Creates a new null heap pointer
    fn null() -> Self {
        Self {
            ptr: ptr::null_mut(),
            _phantom: Default::default(),
        }
    }

    /// Converts the internal pointer to an immutable reference with a panic message if it is null.
    ///
    /// # Safety
    ///
    /// The pointer must not be dangling.
    unsafe fn as_ref(self) -> &'h mut imur_mem::heap::Heap<H> {
        unsafe {
            self.ptr
                .as_mut()
                .expect("SyncHeap is uninitialized before use, use `Self::init` to initialize it")
        }
    }

    /// Converts the internal pointer to a mutable reference with a panic message if it is null.
    ///
    /// # Safety
    ///
    /// The pointer must not be dangling.
    unsafe fn as_mut(self) -> &'h mut imur_mem::heap::Heap<H> {
        unsafe {
            self.ptr
                .as_mut()
                .expect("SyncHeap is uninitialized before use, use `Self::init` to initialize it")
        }
    }
}

impl<'h, H: Memory> HeapAlloc for HeapPtr<'h, H> {
    fn alloc(&mut self, size: usize) -> Option<usize> {
        unsafe { self.as_mut().alloc(size) }
    }

    fn free(&mut self, alloc_index: usize) -> bool {
        unsafe { self.as_mut().free(alloc_index) }
    }

    fn realloc(&mut self, old_alloc_index: usize, new_size: usize) -> Option<usize> {
        unsafe { self.as_mut().realloc(old_alloc_index, new_size) }
    }

    unsafe fn access<F, R>(&self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(NonNull<()>) -> R,
    {
        unsafe { self.as_ref().access(index, f) }
    }

    unsafe fn access_mut<F, R>(&self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(NonNull<()>) -> R,
    {
        unsafe { self.as_ref().access(index, f) }
    }
}
