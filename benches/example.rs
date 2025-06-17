#![feature(test)]
extern crate test;

#[cfg(test)]
mod tests {
    use test::Bencher;

    fn fib(n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        if n == 1 {
            return 1;
        }
        let mut a = 0;
        let mut b = 1;
        for _ in 2..=n {
            let c = a + b;
            a = b;
            b = c;
        }
        b
    }

    #[bench]
    fn bench_fib(b: &mut Bencher) {
        b.iter(|| fib(30));
    }
}
