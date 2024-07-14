use std::cell::RefCell;
use std::iter::FusedIterator;

const WHEEL_PRIMES: [u64; 3] = [2, 3, 5];
const WHEEL_MODULUS: u64 = 30;
const WHEEL: [u64; 8] = [1, 7, 11, 13, 17, 19, 23, 29];

struct GlobalPrimes {
    primes: Vec<u64>,
    wheel_index: usize,
    wheel_base: u64,
}

impl GlobalPrimes {
    pub fn new() -> GlobalPrimes {
        let mut global_primes = GlobalPrimes {
            primes: Vec::with_capacity(1024),
            wheel_index: 0,
            wheel_base: 0,
        };

        global_primes.reset();

        global_primes
    }

    pub fn reset(&mut self) {
        self.primes.clear();
        self.primes.extend(WHEEL_PRIMES.iter().copied());
        self.wheel_index = 1;
        self.wheel_base = 0;
    }

    pub fn last_prime(&self) -> u64 {
        self.primes.last().copied().unwrap()
    }

    pub fn generate_upto(&mut self, max: u64) {
        if max <= self.last_prime() {
            return;
        }

        loop {
            let candidate = self.wheel_base + WHEEL[self.wheel_index];

            self.wheel_index += 1;
            if self.wheel_index == WHEEL.len() {
                self.wheel_index = 0;
                self.wheel_base += WHEEL_MODULUS;
            }

            if self.primes.iter().all(|&prime| candidate % prime != 0) {
                self.primes.push(candidate);

                if candidate >= max {
                    return;
                }
            }
        }
    }

    pub fn generate_count(&mut self, count: usize) {
        while self.primes.len() < count {
            let candidate = self.wheel_base + WHEEL[self.wheel_index];

            self.wheel_index += 1;
            if self.wheel_index == WHEEL.len() {
                self.wheel_index = 0;
                self.wheel_base += WHEEL_MODULUS;
            }

            if self.primes.iter().all(|&prime| candidate % prime != 0) {
                self.primes.push(candidate);
            }
        }
    }
}

thread_local! {
    static GLOBAL_PRIMES: RefCell<GlobalPrimes> = RefCell::new(GlobalPrimes::new());
}

pub struct Primes {
    prime_index: usize,
}

impl Primes {
    pub fn new() -> Primes {
        Primes { prime_index: 0 }
    }
}

impl Iterator for Primes {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        GLOBAL_PRIMES.with(|global_primes| {
            let mut global_primes = global_primes.borrow_mut();

            if self.prime_index >= global_primes.primes.len() {
                let last_prime = global_primes.last_prime();
                global_primes.generate_upto((last_prime * 3) / 2);
            }

            let prime = global_primes.primes[self.prime_index];
            self.prime_index += 1;

            Some(prime)
        })
    }
}

impl FusedIterator for Primes {}

pub fn primes() -> Primes {
    Primes::new()
}

pub fn primes_upto(max: u64) -> impl FusedIterator<Item = u64> {
    Primes::new().take_while(move |&p| p <= max)
}

pub fn nth_prime(k: usize) -> u64 {
    GLOBAL_PRIMES.with(|global_primes| {
        let mut global_primes = global_primes.borrow_mut();

        global_primes.generate_count(k + 1);
        global_primes.primes[k]
    })
}

pub fn factorize(n: u64, factors: &mut Vec<u64>) {
    let mut k = n;

    factors.clear();

    for p in primes() {
        while k % p == 0 {
            factors.push(p);
            k /= p;
        }

        if k == 1 {
            break;
        }
    }

    if factors.is_empty() {
        factors.push(n);
    }
}

pub fn is_prime(n: u64) -> bool {
    GLOBAL_PRIMES.with(|global_primes| {
        let mut global_primes = global_primes.borrow_mut();

        global_primes.generate_upto(n);
        global_primes.primes.binary_search(&n).is_ok()
    })
}

pub fn clear_prime_cache() {
    GLOBAL_PRIMES.with(|global_primes| {
        global_primes.borrow_mut().reset();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dumb_prime_generator(max: u64) -> Vec<u64> {
        let mut ps = vec![];

        for candidate in 2..=max {
            if ps.iter().all(|&prime| candidate % prime != 0) {
                ps.push(candidate);
            }
        }

        ps
    }

    #[test]
    fn primes_iter_01() {
        const MAX: u64 = 1000;

        let ps = dumb_prime_generator(MAX);

        let mut a = Primes::new();
        let mut b = Primes::new();

        for &p in &ps {
            assert_eq!(a.next(), Some(p));
            assert_eq!(b.next(), Some(p));
        }

        let mut a = Primes::new();
        let mut b = Primes::new();

        for &p in &ps {
            assert_eq!(a.next(), Some(p));
        }

        for &p in &ps {
            assert_eq!(b.next(), Some(p));
        }
    }

    #[test]
    fn primes_upto_01() {
        const MAX: u64 = 1000;

        let ps = dumb_prime_generator(MAX);
        let a: Vec<_> = primes_upto(MAX).collect();

        assert_eq!(a, ps);

        let ps = dumb_prime_generator(101);
        let a: Vec<_> = primes_upto(101).collect();

        assert_eq!(a, ps);

        let ps = dumb_prime_generator(100);
        let a: Vec<_> = primes_upto(100).collect();

        assert_eq!(a, ps);
    }

    #[test]
    fn factorize_01() {
        let mut fs = vec![];
        let expected = vec![2, 3, 3, 5, 13, 101];
        let n = expected.iter().product();

        factorize(n, &mut fs);

        assert_eq!(fs, expected);
    }

    #[test]
    fn factorize_02() {
        let mut fs = vec![];
        let expected = vec![2, 3];
        let n = expected.iter().product();

        factorize(n, &mut fs);

        assert_eq!(fs, expected);
    }
}
