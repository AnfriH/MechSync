#[derive(Debug)]
pub(crate) struct MidiData {
    pub instruction: u8,
    pub channel: u8,
    pub note: u8,
    pub velocity: u8
}

impl MidiData {
    pub(crate) fn from_slice(data: &[u8]) -> MidiData {
        let (instruction, channel) = if let Some(inst) = data.get(0) {
            ((*inst & 0b1111_0000) >> 4, *inst & 0b0000_1111)
        } else {
            (0u8, 0u8)
        };
        let note = *data.get(1).unwrap_or(&0);
        let velocity = *data.get(2).unwrap_or(&0);
        MidiData {
            instruction,
            channel,
            note,
            velocity,
        }
    }

    pub(crate) fn to_array(&self) -> [u8; 3] {
        [
            (self.instruction << 4) | self.channel,
            self.note,
            self.velocity
        ]
    }
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