
mod test {
    use wrap_functime::{init, timeit};
    init!{}

    #[timeit("test")]
    fn foo() {
        let mut k = 0;
        for i in 0..1000000000 {
            k += 1
        };
    }

    fn main() {
        foo();
    }
}
