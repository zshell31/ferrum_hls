use std::{fmt::Debug, mem, ops::Index};

use crate::{
    const_asserts::{Assert, IsTrue},
    domain::ClockDomain,
    signal::{Bundle, Signal, SignalValue},
    Cast,
};

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Array<const N: usize, T>([T; N]);

impl<const N: usize, T, U: Copy> Cast<[U; N]> for Array<N, T>
where
    T: Cast<U>,
{
    fn cast(self) -> [U; N] {
        unsafe {
            let res = mem::transmute::<*const T, *const U>(self.0.as_ptr());
            *(res as *const [U; N])
        }
    }
}

impl<const N: usize, T: Copy, U> Cast<Array<N, T>> for [U; N]
where
    U: Cast<T>,
{
    fn cast(self) -> Array<N, T> {
        unsafe {
            let res = mem::transmute::<*const U, *const T>(self.as_ptr());
            *(res as *const [T; N])
        }
        .into()
    }
}

impl<const N: usize, T: SignalValue> SignalValue for Array<N, T> {}

impl<const N: usize, T> From<[T; N]> for Array<N, T> {
    fn from(value: [T; N]) -> Self {
        Self(value)
    }
}

impl<const N: usize, T: PartialEq> PartialEq<Array<N, T>> for Array<N, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<const N: usize, U, T: PartialEq<U>> PartialEq<[U; N]> for Array<N, T> {
    fn eq(&self, other: &[U; N]) -> bool {
        self.0 == *other
    }
}

impl<const N: usize, T: Eq> Eq for Array<N, T> {}

impl<const N: usize, T> Array<N, T> {
    pub fn into_inner(self) -> [T; N] {
        self.0
    }

    pub fn at<const M: usize>(self) -> T
    where
        Assert<{ M < N }>: IsTrue,
        T: Clone,
    {
        self.0[M].clone()
    }

    pub fn slice<const S: usize, const M: usize>(self) -> Array<M, T>
    where
        Assert<{ M > 0 }>: IsTrue,
        Assert<{ S + M - 1 < N }>: IsTrue,
        for<'a> [T; M]: TryFrom<&'a [T]>,
    {
        Array::from(match <[T; M]>::try_from(&self.0[S .. (S + M)]) {
            Ok(res) => res,
            Err(_) => unreachable!(),
        })
    }

    pub fn reverse(self) -> Self
    where
        T: Clone,
    {
        let mut values = self.0.clone();
        for i in 0 .. N {
            values[N - i - 1] = self.0[i].clone();
        }
        values.into()
    }
}

impl<const N: usize, T> Index<usize> for Array<N, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<const N: usize, D: ClockDomain, T: SignalValue> Bundle<D, T> for Array<N, T> {
    type Unbundled = Array<N, Signal<D, T>>;

    fn bundle(mut signals: Self::Unbundled) -> Signal<D, Self> {
        Signal::new(move || {
            let values = signals
                .0
                .iter_mut()
                .map(|signal| signal.next())
                .collect::<Vec<T>>();

            Array::from(match <[T; N]>::try_from(values) {
                Ok(res) => res,
                Err(_) => unreachable!(),
            })
        })
    }

    fn unbundle(signal: Signal<D, Self>) -> Self::Unbundled {
        let signals = (0 .. N)
            .map(|ind| signal.clone().map(move |s| s[ind].clone()))
            .collect::<Vec<_>>();

        Array::from(match <[Signal<D, T>; N]>::try_from(signals) {
            Ok(res) => res,
            Err(_) => unreachable!(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bit::{H, L},
        signal::SignalIterExt,
    };

    pub struct TestSystem;

    impl ClockDomain for TestSystem {
        const FREQ: usize = 4;
    }

    #[test]
    fn at() {
        assert_eq!(Array::from([3, 2, 1, 0]).at::<3>(), 0);
    }

    #[test]
    fn slice() {
        assert_eq!(
            Array::from([3, 2, 1, 0]).slice::<1, 2>(),
            Array::from([2, 1])
        );
    }

    #[test]
    fn unbundle() {
        let s = [[H, H, L], [L, H, L], [H, L, H], [L, L, H]]
            .into_iter()
            .map(Array::from)
            .into_signal::<TestSystem>();

        let res = Array::<3, _>::unbundle(s);

        let extract = |ind| res[ind].clone().iter().take(4).collect::<Vec<_>>();

        assert_eq!(extract(0), [H, L, H, L]);
        assert_eq!(extract(1), [H, H, L, L]);
        assert_eq!(extract(2), [L, L, H, H]);
    }

    #[test]
    fn bundle() {
        let s = Array::from([
            [H, L, H, L].into_signal::<TestSystem>(),
            [H, H, L, L].into_signal::<TestSystem>(),
            [L, L, H, H].into_signal::<TestSystem>(),
        ]);

        let res = Array::<3, _>::bundle(s);

        assert_eq!(res.iter().take(4).collect::<Vec<_>>(), [
            [H, H, L],
            [L, H, L],
            [H, L, H],
            [L, L, H]
        ]);
    }
}
