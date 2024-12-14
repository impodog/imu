#[cfg(test)]
mod tests {
    use imuc_ctx::Dag;

    #[test]
    fn test_linear() {
        let mut dag = Dag::new();
        let x = dag.push(1);
        let y = dag.push(2);
        let z = dag.push(3);
        dag.add(x, y);
        dag.add(y, z);
        let sorted = dag.topo_sort().unwrap();
        assert_eq!(sorted, [1, 2, 3]);
    }

    #[test]
    fn test_loop() {
        let mut dag = Dag::new();
        let x = dag.push(1);
        let y = dag.push(2);
        let z = dag.push(3);
        dag.add(z, x);
        dag.add(x, y);
        dag.add(y, z);
        assert_eq!(dag.topo_sort(), None);
    }

    #[test]
    fn test_rhombus() {
        let mut dag = Dag::new();
        let x = dag.push(1);
        let y = dag.push(2);
        let z = dag.push(3);
        let w = dag.push(4);
        dag.add(y, x);
        dag.add(y, z);
        dag.add(x, z);
        dag.add(x, w);
        dag.add(z, w);
        let sorted = dag.topo_sort().unwrap();
        assert_eq!(sorted, [2, 1, 3, 4]);
    }

    #[test]
    fn test_disconnected() {
        let mut dag = Dag::new();
        let x = dag.push(1);
        let y = dag.push(2);
        let z = dag.push(3);
        let w = dag.push(4);
        dag.add(x, y);
        dag.add(z, w);
        dag.add(x, w);
        let sorted = dag.topo_sort().unwrap();
        assert_eq!(sorted, [1, 3, 2, 4]);
    }

    #[test]
    fn test_large_linear() {
        let mut dag = Dag::new();
        let mut prev: Option<usize> = None;
        for i in 0..1000000 {
            let next = dag.push(i);
            if let Some(prev) = prev {
                dag.add(next, prev);
            }
            prev = Some(next);
        }
        dag.topo_sort().unwrap();
    }
}
