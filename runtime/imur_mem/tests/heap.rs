#[cfg(test)]
mod tests {
    #[cfg(feature = "std")]
    #[test]
    fn test_heap() {
        let mem = imur_mem::alloc::VecMem::new();
        let mut heap = imur_mem::heap::Heap::new(mem, 10);
        heap.init();
        let index1 = heap.alloc(8).unwrap();
        println!("index = {}", index1);
        unsafe {
            assert!(heap.copy(index1, &1234u64));
            assert_eq!(Some(1234u64), heap.access(index1, |value: &u64| { *value }));
        }

        let index2 = heap.alloc(64).unwrap();
        println!("index2 = {}", index2);
        unsafe {
            for value in 0..8u64 {
                assert!(heap.copy(index2 + value as usize * 8, &value));
            }
        }
        unsafe {
            assert_eq!(
                Some(2u64),
                heap.access(index2 + 16, |value: &u64| { *value })
            );
            assert_eq!(
                Some(4u64),
                heap.access(index2 + 32, |value: &u64| { *value })
            );
        }

        assert!(heap.free(index1));
        assert!(heap.free(index2));

        // Reusing
        let index3 = heap.alloc(7).unwrap();
        assert_eq!(index1, index3); // Reusing guaranteed
        unsafe {
            assert!(heap.copy(index3, &0xabcdefu32));
        }

        // Moving to another reused chunk
        let index4 = heap.realloc(index3, 60).unwrap();
        assert_eq!(index4, index2);
        unsafe {
            // Test if the data is shipped too
            assert_eq!(Some(0xabcdefu32), heap.access(index4, |value: &u32| *value));
        }
    }
}
