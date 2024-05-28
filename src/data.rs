#[derive(Debug)]
pub(crate) struct MidiData {
    pub ts: u64,
    pub data: [u8; 3]
}

impl MidiData {
    pub(crate) fn from_slice(ts: u64, data: &[u8]) -> MidiData {
        let mut md = MidiData {
            ts,
            data: [0u8; 3],
        };
        for (x, y) in md.data.iter_mut().zip(data) {
            *x = *y;
        }
        md
    }
}

#[macro_export] macro_rules! rwlock_get {
    ($expression:expr) => {
        &*($expression.read().unwrap())
    };
}

#[macro_export] macro_rules! rwlock_get_mut {
    ($expression:expr) => {
        *($expression.write().unwrap())
    };
}

// TODO: determine whether this is required
// trait ToArray<T> {
//     fn try_array<const S: usize>(&mut self) -> Option<[T; S]>;
// }
//
// impl<T> ToArray<T> for dyn Iterator<Item=T> {
//     fn try_array<const S: usize>(&mut self) -> Option<[T; S]> {
//         let data: [T; S] = [Default::default(); S];
//         data
//     }
// }