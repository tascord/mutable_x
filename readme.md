## MutableX
Create X-sized mutable signals for FutureSignals.

### Usage
From [tests/basic.rs](/tests/basic.rs)
```rs
#[cfg(test)]
pub mod test {
    use futures_signals::signal::{Mutable, SignalExt};
    use mutablex::mutable_x;

    #[test]
    pub fn basic() {
        mutable_x!(2);

        let stream_a = Mutable::new(1);
        let stream_b = Mutable::new(2);

        let combination = Mutable2::new(stream_a.clone(), stream_b.clone());

        let _ = combination.map(|(a, b)| {
            println!("{} {}", a, b);
        });

        stream_a.set(3);
        stream_b.set(4);
    }
}
```