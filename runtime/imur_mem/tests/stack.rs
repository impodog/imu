#[cfg(test)]
mod tests {
    #[cfg(feature = "std")]
    #[test]
    fn test_stack() {
        let mem = imur_mem::alloc::VecMem::new();
        let mut stack = imur_mem::stack::Stack::new(mem);
        let int_ptr = stack.push(8i32).unwrap();
        let float_ptr = stack.push(std::f64::consts::PI).unwrap();
        let str_ptr = stack.push("Hello, world!").unwrap();
        for _ in 0..3 {
            assert_eq!(8, unsafe { stack.access::<i32>(int_ptr).unwrap() });
            assert_eq!(std::f64::consts::PI, unsafe {
                stack.access::<f64>(float_ptr).unwrap()
            });
            assert_eq!("Hello, world!", unsafe {
                stack.access::<&'static str>(str_ptr).unwrap()
            });
            // Add additional disruption to make sure it doesn't change
            stack.push(120u64);
        }
        // Now reset to just one i32
        stack.reset(4);
        assert_eq!(8, unsafe { stack.access::<i32>(int_ptr).unwrap() });
        // The float does not exist anymore
        assert_eq!(None, unsafe { stack.access::<f64>(float_ptr) });

        // Duplicate the stack
        let second_int_ptr = stack.duplicate(0, 4).unwrap();
        assert_eq!(8, unsafe { stack.access::<i32>(second_int_ptr).unwrap() });
    }
}
