#[derive(Debug, Clone, Copy)]
pub struct Packet {
    pub index: usize,
    pub len: usize,
    pub arrival_time: usize,
}

impl Packet {
    pub fn new(index: usize, len: usize, arrival_time: usize) -> Packet {
        Packet {
            index,
            len,
            arrival_time,
        }
    }
}
