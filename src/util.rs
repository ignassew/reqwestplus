use http::header::HeaderName;

use crate::header::{Entry, HeaderMap};

// xor-shift
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn fast_random() -> u64 {
    use std::cell::Cell;
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    use std::num::Wrapping;

    thread_local! {
        static RNG: Cell<Wrapping<u64>> = Cell::new(Wrapping(seed()));
    }

    fn seed() -> u64 {
        let seed = RandomState::new();

        let mut out = 0;
        let mut cnt = 0;
        while out == 0 {
            cnt += 1;
            let mut hasher = seed.build_hasher();
            hasher.write_usize(cnt);
            out = hasher.finish();
        }
        out
    }

    RNG.with(|rng| {
        let mut n = rng.get();
        debug_assert_ne!(n.0, 0);
        n ^= n >> 12;
        n ^= n << 25;
        n ^= n >> 27;
        rng.set(n);
        n.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    })
}

#[allow(unused)]
pub(crate) fn replace_headers(dst: &mut HeaderMap, src: HeaderMap) {
    // IntoIter of HeaderMap yields (Option<HeaderName>, HeaderValue).
    // The first time a name is yielded, it will be Some(name), and if
    // there are more values with the same name, the next yield will be
    // None.

    let mut prev_key: Option<HeaderName> = None;
    for (key, value) in src {
        match key {
            Some(key) => {
                match dst.entry(&key) {
                    Entry::Occupied(mut e) => _ = e.insert(value),
                    Entry::Vacant(e) => _ = e.insert(value),
                };

                prev_key = Some(key)
            },
            None => {
                let prev_key = prev_key.as_ref().expect("HeaderMap::into_iter won't yield None first");

                match dst.entry(prev_key) {
                    Entry::Occupied(mut e) => _ = e.append(value),
                    Entry::Vacant(e) => unreachable!("HeaderMap::into_iter yielded None first"),
                };
            },
        };

        
    }
}
