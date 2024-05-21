#[derive(Debug)]
pub(crate) struct MidiData {
    pub ts: u64,
    pub data: [u8; 3]
}

impl MidiData {
    pub(crate) fn from_slice(ts: u64, data: &[u8]) -> MidiData {
        let mut md = MidiData {
            ts,
            data: [0u8, 0u8, 0u8],
        };
        for (x, y) in md.data.iter_mut().zip(data) {
            *x = *y;
        }
        md
    }
}