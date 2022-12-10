use crate::scheduling::{flow::Flow, packet::Packet};

#[derive(Debug)]
pub struct OutputLink {
    capacity: usize,
    current_load: usize,
    history: Vec<(usize, usize)>,
}

impl OutputLink {
    pub fn add(&mut self, flow: usize, packet: &Packet) {
        self.current_load += packet.len;
        self.history.push((flow, packet.index));
    }

    pub fn tick(&mut self) {
        if self.current_load >= self.capacity {
            self.current_load -= self.capacity;
        }
    }

    pub fn empty(&self) -> bool {
        self.current_load == 0
    }
}

#[derive(Debug)]
pub struct DRRScheduler {
    timer: usize,
    flows: Vec<Flow>,
    weights: Vec<usize>,
    deficit_counters: Vec<usize>,
    output_link: OutputLink,
}

impl DRRScheduler {
    pub fn new(capacity: usize) -> DRRScheduler {
        DRRScheduler {
            timer: 0,
            flows: Vec::new(),
            weights: Vec::new(),
            deficit_counters: Vec::new(),
            output_link: OutputLink {
                capacity,
                current_load: 0,
                history: Vec::new(),
            },
        }
    }

    pub fn add_flow(&mut self, flow: Flow, weight: usize) {
        self.flows.push(flow);
        self.weights.push(weight);
        self.deficit_counters.push(weight);
    }

    fn schedule(&mut self) -> bool {
        if !self.output_link.empty() {
            return false;
        }
        for i in 0..self.flows.len() {
            if let Some(p) = self.flows[i].peek_packet(self.timer) {
                if self.deficit_counters[i] >= p.len {
                    self.deficit_counters[i] -= p.len;
                    self.output_link.add(i, &p);
                    self.flows[i].pop_packet();
                }
            } else {
                self.deficit_counters[i] = 0;
            }
        }
        true
    }

    pub fn tick(&mut self) -> bool {
        if self.flows.iter().all(|f| f.empty()) {
            return false;
        }
        self.timer += 1;
        self.output_link.tick();
        if !self.output_link.empty() {
            return true;
        }

        assert!(
            self.flows.len() == self.weights.len()
                && self.weights.len() == self.deficit_counters.len()
        );

        // Add back if scheduled
        if self.schedule() {
            for i in 0..self.flows.len() {
                self.deficit_counters[i] += self.weights[i];
            }
        }

        true
    }

    pub fn run(&mut self) {
        while self.tick() {}
    }

    pub fn get_history(&self) -> &Vec<(usize, usize)> {
        &self.output_link.history
    }
}

#[cfg(test)]
mod test {
    use crate::{
        scheduling::{flow, packet, schedulers::drr::DRRScheduler},
        *,
    };

    #[test]
    fn ddr_test() {
        let mut scheduler = DRRScheduler::new(1);

        let mut flow = flow::Flow::new();
        flow.add_packet(packet::Packet::new(0, 3, 0));
        flow.add_packet(packet::Packet::new(1, 4, 8));
        scheduler.add_flow(flow, 3);

        let mut flow = flow::Flow::new();
        flow.add_packet(packet::Packet::new(0, 3, 0));
        flow.add_packet(packet::Packet::new(1, 1, 12));
        scheduler.add_flow(flow, 2);

        let mut flow = flow::Flow::new();
        flow.add_packet(packet::Packet::new(0, 6, 0));
        flow.add_packet(packet::Packet::new(1, 1, 11));
        scheduler.add_flow(flow, 5);

        scheduler.run();

        assert_eq!(scheduler.timer, 15);
        assert_eq!(scheduler.get_history().len(), 6);
        assert_eq!(
            scheduler.get_history(),
            &vec![(0, 0), (1, 0), (2, 0), (1, 1), (2, 1), (0, 1),]
        );
    }
}
