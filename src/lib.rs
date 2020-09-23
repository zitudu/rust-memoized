use std::time::{Instant, Duration};
use std::mem::MaybeUninit;
use std::rc::Rc;
use std::marker::PhantomData;

pub enum Memoized<'a, I, T> {
    UnInitialized(Box<dyn 'a + FnMut(I) -> T>),
    Data(T),
}

impl<'a, I, T> Memoized<'a, I, T> {
    pub fn new<F: 'a + FnMut(I) -> T>(f: F) -> Self {
        Self::UnInitialized(Box::new(f))
    }

    pub fn get(&mut self, input: I) -> &T {
        match self {
            Self::UnInitialized(ref mut f) => {
                let t: T = f(input);
                *self = Self::Data(t);
                match self {
                    Self::Data(ref t) => {
                        t
                    }
                    _ => unreachable!()
                }
            }
            Self::Data(ref t) => {
                t
            }
        }
    }
}

pub fn memoize<'a, I, T, F: 'a + FnMut(I) -> T>(f: F) -> Memoized<'a, I, T> {
    Memoized::new(f)
}

pub struct MemoizedWithExpiration<'a, I, T> {
    duration: Duration,
    last: Instant,
    f: Box<'a + FnMut(I) -> T>,
    t: Option<Rc<T>>,
}

impl<'a, I, T> MemoizedWithExpiration<'a, I, T> {
    pub fn new<F: 'a + FnMut(I) -> T>(f: F, duration: Duration) -> Self {
        Self {
            duration,
            last: Instant::now(),
            f: Box::new(f),
            t: None,
        }
    }

    pub fn get(&mut self, input: I) -> Rc<T> {
        if self.t.is_none() || self.last.elapsed() > self.duration {
            self.t = Some(Rc::new((self.f)(input)));
            self.last = Instant::now();
        }
        Rc::clone(self.t.as_ref().unwrap())
    }
}

pub fn memoize_with_expiration<'a, I, T, F: 'a + FnMut(I) -> T>(f: F, duration: Duration) -> MemoizedWithExpiration<'a, I, T> {
    MemoizedWithExpiration::new(f, duration)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::ops::Shl;

    #[test]
    fn test_memoize() {
        let mut called = 0;
        let mut m = memoize(move |()| {
            called += 1;
            called
        });
        assert_eq!(m.get(()), &1);
        assert_eq!(m.get(()), &1);
    }

    #[test]
    fn test_memoize_with_expiration() {
        let mut called = 0;
        let mut m = memoize_with_expiration(move |()| {
            called += 1;
            called
        }, Duration::from_secs(1));
        for _ in 0..1000 {
            assert_eq!(m.get(()), Rc::new(1));
        }
        sleep(Duration::from_secs(1));
        for _ in 0..1000 {
            assert_eq!(m.get(()), Rc::new(2));
        }
    }

    #[test]
    fn test_memoize_in_struct() {
        struct M<'a> {
            d: i32,
            m: Memoized<'a, Box<i32>, i32>,
        }
        let mut m = M {
            d: 0,
            m: memoize(|d: Box<i32>| {
                *d + 1
            }),
        };
        assert_eq!(m.m.get(Box::new(m.d)), &1);
        m.d = 10;
        assert_eq!(m.m.get(Box::new(m.d)), &1);
    }

    #[test]
    fn test_memoize_with_expiration_in_struct() {
        struct M<'a> {
            d: i32,
            m: MemoizedWithExpiration<'a, Box<i32>, i32>,
        }
        let mut m = M {
            d: 0,
            m: memoize_with_expiration(|d: Box<i32>| {
                *d + 1
            }, Duration::from_secs(1)),
        };
        assert_eq!(m.m.get(Box::new(m.d)), Rc::new(1));
        m.d = 10;
        assert_eq!(m.m.get(Box::new(m.d)), Rc::new(1));
        sleep(Duration::from_secs(1));
        assert_eq!(m.m.get(Box::new(m.d)), Rc::new(11));
    }
}
