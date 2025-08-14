#[cfg(test)]
mod tests {
    #[cfg(feature = "vec")]
    #[test]
    fn test_vec() {
        let mem = imur_mem::alloc::VecMem::new();
        let mut heap = imur_mem::heap::Heap::new(mem, 10);
        heap.init();

        let mut vec = imur_mem::ds::vec::Vec::new(heap);
        vec.push(1i32);
        vec.push(2i32);
        vec.push(4i32);
        assert!(vec.access(0, |value| assert_eq!(*value, 1)).is_some());
        assert!(vec.access(2, |value| assert_eq!(*value, 4)).is_some());
        vec.pop();
        assert!(
            vec.access(2, |_value| panic!("this should not be run!"))
                .is_none()
        );
        vec.shrink_to_fit();
    }
}
