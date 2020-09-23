use std::time::{Instant, Duration};
use std::mem::MaybeUninit;
use std::rc::Rc;

pub enum Memoized<'a, T> {
    UnInitialized(Box<'a + FnMut() -> T>),
    Data(T),
}

impl<'a, T> Memoized<'a, T> {
    pub fn new<F: 'a + FnMut() -> T>(f: F) -> Self {
        Self::UnInitialized(Box::new(f))
    }

    pub fn get(&mut self) -> &T {
        match self {
            Self::UnInitialized(ref mut f) => {
                let t: T = f();
                *self = Self::Data(t);
                self.get()
            }
            Self::Data(ref t) => {
                t
            }
        }
    }
}

pub struct MemoizedWithExpiration<'a, T> {
    duration: Duration,
    last: Instant,
    f: Box<'a + FnMut() -> T>,
    t: Option<Rc<T>>,
}

impl<'a, T> MemoizedWithExpiration<'a, T> {
    pub fn new<F: 'a + FnMut() -> T>(f: F, duration: Duration) -> Self {
        Self {
            duration,
            last: Instant::now(),
            f: Box::new(f),
            t: None,
        }
    }

    pub fn get(&mut self) -> Rc<T> {
        if self.t.is_none() || self.last.elapsed() > self.duration {
            self.t = Some(Rc::new((self.f)()));
        }
        Rc::clone(self.t.as_ref().unwrap())
    }
}

pub fn memoize<'a, T, F: 'a + FnMut() -> T>(f: F) -> Memoized<'a, T> {
    Memoized::new(f)
}

pub fn memoize_with_expiration<'a, T, F: 'a + FnMut() -> T>(f: F, duration: Duration) -> MemoizedWithExpiration<'a, T> {
    MemoizedWithExpiration::new(f, duration)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_memoize() {
        let mut called = 0;
        let mut m = memoize(move || {
            called += 1;
            called
        });
        assert_eq!(m.get(), &1);
        assert_eq!(m.get(), &1);
    }

    fn test_memoize_with_expiration() {
        let mut called = 0;
        let mut m = memoize_with_expiration(move || {
            called += 1;
            called
        }, Duration::from_secs(1));
        for _ in 0..1000 {
            assert_eq!(m.get(), Rc::new(1));
        }
        sleep(Duration::from_secs(1));
        for _ in 0..1000 {
            assert_eq!(m.get(), Rc::new(2));
        }
    }
}
