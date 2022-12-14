pub mod flow;
pub mod schedulers;

/// A trait for objects that can be ticked.
trait Tickable {
    /// Tick the object.
    /// Returns false if the object is done.
    fn tick(&mut self) -> bool;
}

trait Schedulable<T>: Tickable {
    fn schedule(&mut self) -> T;
}

#[derive(Debug)]
pub struct Port {
    pub id: usize,
    rate: usize,
    in_queue: Vec<Packet>,
    out_queue: Vec<Packet>,

    current_processed: usize,
}

impl Port {
    pub fn new(id: usize, rate: usize) -> Port {
        Port {
            id,
            rate,
            current_processed: 0,
            in_queue: Vec::new(),
            out_queue: Vec::new(),
        }
    }

    pub fn empty(&self) -> bool {
        self.in_queue.is_empty()
    }

    pub fn submit(&mut self, packet: Packet) {
        self.in_queue.push(packet);
    }

    pub fn get_output(&mut self) -> &Vec<Packet> {
        &self.out_queue
    }

    pub fn proceed_rest(&mut self) {
        while let Some(packet) = self.in_queue.first() {
            self.current_processed = 0;
            self.out_queue.push(self.in_queue.remove(0));
        }
        self.current_processed = 0;
    }

    pub fn get_bandwidth(&self) -> usize {
        self.rate
    }
}

impl Tickable for Port {
    fn tick(&mut self) -> bool {
        if let Some(packet) = self.in_queue.first() {
            self.current_processed += self.rate;
            if self.current_processed >= packet.len {
                self.current_processed = 0;
                self.out_queue.push(self.in_queue.remove(0));
            }
        }
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Packet {
    pub name: &'static str,
    pub len: usize,
}

impl Packet {
    pub fn new(name: &'static str, len: usize) -> Packet {
        Packet { name, len }
    }
}
