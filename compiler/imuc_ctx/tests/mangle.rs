#[cfg(test)]
mod tests {
    use imuc_ctx::ctx::mangle::*;

    #[test]
    fn test_tuple_name() {
        assert_eq!(tuple_name(["abc", "def", "ghi"]), "(abc,def,ghi)");
        assert_eq!(tuple_name(["abc", ""]), "(abc,)");
        assert_eq!(tuple_name([]), "()");
    }

    #[test]
    fn bench_tuple_name() {
        let mut arr = Vec::new();
        for i in 0..1000000 {
            arr.push(i.to_string());
        }
        let iter = arr.iter().map(|s| s.as_str());
        tuple_name(iter);
    }
}
