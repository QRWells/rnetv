use crate::packet::Packet;

#[derive(Debug)]
pub struct Flow {
    pub packets: Vec<Packet>,
}

impl Flow {
    pub fn new() -> Flow {
        Flow {
            packets: Vec::new(),
        }
    }

    pub fn add_packet(&mut self, packet: Packet) {
        self.packets.push(packet);
        self.packets
            .sort_by(|a, b| a.arrival_time.cmp(&b.arrival_time));
    }

    pub fn pop_packet(&mut self) -> Packet {
        self.packets.remove(0)
    }

    pub fn peek_packet(&self, time: usize) -> Option<Packet> {
        if let Some(packet) = self.packets.first() {
            if packet.arrival_time <= time {
                Some(packet.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn empty(&self) -> bool {
        self.packets.len() == 0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn flow_test() {
        let mut flow = Flow::new();
        assert!(flow.empty());

        flow.add_packet(Packet::new(0, 10, 0));
        assert!(!flow.empty());
        assert!(flow.peek_packet(0).is_some());
    }
}
