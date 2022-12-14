use crate::scheduling::{
    flow::{Flow, VariableLengthFlow},
    Port, Schedulable, Tickable,
};

/// Deficit Round Robin (DRR) scheduler.
#[derive(Debug)]
pub struct DRRScheduler {
    timer: usize,
    flows: Vec<VariableLengthFlow>,
    weights: Vec<usize>,
    deficit_counters: Vec<usize>,
    output_port: Port,
}

impl DRRScheduler {
    pub fn new(capacity: usize) -> DRRScheduler {
        DRRScheduler {
            timer: 0,
            flows: Vec::new(),
            weights: Vec::new(),
            deficit_counters: Vec::new(),
            output_port: Port::new(0, capacity),
        }
    }

    pub fn add_flow(&mut self, flow: VariableLengthFlow, weight: usize) {
        self.flows.push(flow);
        self.weights.push(weight);
        self.deficit_counters.push(weight);
    }

    pub fn run(&mut self) {
        while self.tick() {}
        self.output_port.proceed_rest();
    }

    pub fn get_output_port(&mut self) -> &mut Port {
        &mut self.output_port
    }
}

impl Tickable for DRRScheduler {
    fn tick(&mut self) -> bool {
        if self.flows.iter().all(|f| f.empty()) {
            return false;
        }
        self.timer += 1;
        self.output_port.tick();
        if !self.output_port.empty() {
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
}

impl Schedulable<bool> for DRRScheduler {
    fn schedule(&mut self) -> bool {
        if !self.output_port.empty() {
            return false;
        }
        for i in 0..self.flows.len() {
            if let Some(p) = self.flows[i].peek_packet(self.timer) {
                if self.deficit_counters[i] >= p.len {
                    self.deficit_counters[i] -= p.len;
                    self.output_port.submit(p);
                    self.flows[i].pop_packet();
                }
            } else {
                self.deficit_counters[i] = 0;
            }
        }
        true
    }
}

#[cfg(test)]
mod test {
    use crate::scheduling::{
        flow::{self, Flow},
        schedulers::drr::DRRScheduler,
        Packet,
    };

    #[test]
    fn ddr_test() {
        let mut scheduler = DRRScheduler::new(1);

        let mut flow = flow::VariableLengthFlow::new();
        flow.packet_arrive(Packet::new("1_1", 3), 0);
        flow.packet_arrive(Packet::new("1_2", 4), 8);
        scheduler.add_flow(flow, 3);

        let mut flow = flow::VariableLengthFlow::new();
        flow.packet_arrive(Packet::new("2_1", 3), 0);
        flow.packet_arrive(Packet::new("2_2", 1), 12);
        scheduler.add_flow(flow, 2);

        let mut flow = flow::VariableLengthFlow::new();
        flow.packet_arrive(Packet::new("3_1", 6), 0);
        flow.packet_arrive(Packet::new("3_2", 1), 11);
        scheduler.add_flow(flow, 5);

        scheduler.run();

        assert_eq!(scheduler.timer, 15);

        let output = scheduler.output_port.get_output();

        assert_eq!(output.len(), 6);
        assert_eq!(
            output,
            &vec![
                Packet::new("1_1", 3),
                Packet::new("2_1", 3),
                Packet::new("3_1", 6),
                Packet::new("2_2", 1),
                Packet::new("3_2", 1),
                Packet::new("1_2", 4)
            ]
        );
    }
}
