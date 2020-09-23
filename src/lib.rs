use std::time::{Instant, Duration};
use std::mem::MaybeUninit;
use std::rc::Rc;

pub enum Memoized<T, F: FnMut() -> T> {
    UnInitialized(Box<F>),
    Data(T),
}

impl<T, F: FnMut() -> T> Memoized<T, F> {
    pub fn new(f: F) -> Self {
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

pub struct MemoizedWithExpiration<T, F: FnMut() -> T> {
    duration: Duration,
    last: Instant,
    f: F,
    t: Option<Rc<T>>,
}

impl<T, F: FnMut() -> T> MemoizedWithExpiration<T, F> {
    pub fn new(f: F, duration: Duration) -> Self {
        Self {
            duration,
            last: Instant::now(),
            f,
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

pub fn memoize<T, F: FnMut() -> T>(f: F) -> Memoized<T, F> {
    Memoized::new(f)
}

pub fn memoize_with_expiration<T, F: FnMut() -> T>(f: F, duration: Duration) -> MemoizedWithExpiration<T, F> {
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
