#[cfg(test)]
mod tests {
    #[cfg(feature = "vec")]
    #[test]
    fn test_vec() {
        let mem = imur_mem::alloc::vec_mem::VecMem::new();
        let heap = imur_mem::heap::bare_heap::Heap::new(mem, 10);
        let mut heap = imur_mem::heap::sync_heap::SyncHeap::new(&heap);
        heap.init();

        let mut vec = imur_mem::ds::vec::Vec::new(heap);
        assert!(vec.push(1i32));
        assert!(vec.push(2i32));
        assert!(vec.push(4i32));
        assert!(vec.access(0, |value| assert_eq!(*value, 1)).is_some());
        assert!(vec.access(2, |value| assert_eq!(*value, 4)).is_some());
        vec.pop();
        assert!(
            vec.access(2, |_value| panic!("this should not be run!"))
                .is_none()
        );
        assert!(vec.shrink_to_fit());
    }
}
