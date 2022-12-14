use crate::scheduling::Packet;

pub trait Flow {
    /// Add a packet to the flow.
    fn packet_arrive(&mut self, packet: Packet, time: usize);

    /// Pop a packet from the flow.
    fn pop_packet(&mut self) -> Packet;

    /// Peek at the next packet in the flow at a given time.
    /// If there is no packet available, return None.
    fn peek_packet(&self, time: usize) -> Option<Packet>;

    /// Check if the flow is empty.
    fn empty(&self) -> bool;
}

/// A flow with variable-length packets.
#[derive(Debug)]
pub struct VariableLengthFlow {
    pub packet_states: Vec<(Packet, usize)>,
}

/// A flow with fixed-length packets.
#[derive(Debug)]
pub struct FixedLengthFlow {
    pub packet_len: usize,
    pub packet_states: Vec<(Packet, usize)>,
}

impl VariableLengthFlow {
    pub fn new() -> VariableLengthFlow {
        VariableLengthFlow {
            packet_states: Vec::new(),
        }
    }
}

impl Flow for VariableLengthFlow {
    fn packet_arrive(&mut self, packet: Packet, time: usize) {
        self.packet_states.push((packet, time));
        self.packet_states.sort_by(|a, b| a.1.cmp(&b.1));
    }

    fn pop_packet(&mut self) -> Packet {
        self.packet_states.remove(0).0
    }

    fn peek_packet(&self, time: usize) -> Option<Packet> {
        if let Some((packet, arrive_time)) = self.packet_states.first() {
            if arrive_time <= &time {
                Some(packet.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn empty(&self) -> bool {
        self.packet_states.len() == 0
    }
}

impl FixedLengthFlow {
    /// Create a new flow with fixed-length packets.
    pub fn new(packet_len: usize) -> FixedLengthFlow {
        FixedLengthFlow {
            packet_len,
            packet_states: Vec::new(),
        }
    }

    fn ensure_packet_len(&mut self, packet: Packet) -> Packet {
        if packet.len != self.packet_len {
            Packet {
                len: self.packet_len,
                ..packet
            }
        } else {
            packet
        }
    }

    fn ensure_packet_order(&mut self) {
        self.packet_states.sort_by(|a, b| a.1.cmp(&b.1));
    }

    pub fn add_packet(&mut self, name: &'static str, arrive_time: usize) {
        self.packet_states
            .push((Packet::new(name, self.packet_len), arrive_time));
        self.ensure_packet_order();
    }
}

impl Flow for FixedLengthFlow {
    /// Add a packet to the flow.
    ///
    /// If the packet length is different from the flow's packet length,
    /// the packet will be resized to the flow's packet length.
    fn packet_arrive(&mut self, packet: Packet, time: usize) {
        if packet.len != self.packet_len {
            let packet = Packet {
                len: self.packet_len,
                ..packet
            };
            self.packet_states.push((packet, time));
        } else {
            self.packet_states.push((packet, time));
        }
        self.ensure_packet_order();
    }

    fn pop_packet(&mut self) -> Packet {
        self.packet_states.remove(0).0
    }

    fn peek_packet(&self, time: usize) -> Option<Packet> {
        if let Some((packet, arrive_time)) = self.packet_states.first() {
            if arrive_time <= &time {
                return Some(packet.clone());
            }
        }
        None
    }

    fn empty(&self) -> bool {
        self.packet_states.len() == 0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn flow_test() {
        let mut flow = VariableLengthFlow::new();
        assert!(flow.empty());

        flow.packet_arrive(Packet::new("test", 10), 0);
        assert!(!flow.empty());
        assert!(flow.peek_packet(0).is_some());
    }
}
