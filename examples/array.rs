#![feature(generic_const_exprs)]
#![allow(incomplete_features)]
use ferrum_hdl::prelude::*;

pub fn top_module(signals: Signal<TD8, Array<4, U<4>>>) -> Signal<TD8, Array<4, U<4>>> {
    signals.map(|signals| {
        let [start, .., end] = signals.cast::<[U<4>; 4]>();

        [
            start.clone(),
            start.clone() + end.clone(),
            start - end.clone(),
            end,
        ]
    })
}

#[cfg(test)]
mod tests {
    use ferrum_hdl::{signal::SignalIterExt, Cast};

    use super::*;

    #[test]
    fn signals() {
        let s = [[0, 1, 2, 3], [1, 2, 3, 4], [2, 3, 4, 5], [3, 4, 5, 6]]
            .into_iter()
            .map(Cast::cast::<Array<4, Unsigned<4>>>)
            .into_signal();

        let res = top_module(s);

        assert_eq!(res.iter().take(4).collect::<Vec<_>>(), [
            [0, 3, 13, 3],
            [1, 5, 13, 4],
            [2, 7, 13, 5],
            [3, 9, 13, 6]
        ]);
    }
}
