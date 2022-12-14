use crate::scheduling::{
    flow::{Flow, VariableLengthFlow},
    Packet, Port, Schedulable, Tickable,
};

/// Weighted Fair Queueing (WFQ) scheduler
pub struct WFQScheduler {
    timer: usize,
    weights: Vec<f64>,
    total_weight: f64,
    flows: Vec<VariableLengthFlow>,
    output_port: Port,
}

impl WFQScheduler {
    pub fn new(bandwidth: usize) -> WFQScheduler {
        WFQScheduler {
            timer: 0,
            weights: Vec::new(),
            total_weight: 0f64,
            flows: Vec::new(),
            output_port: Port::new(0, bandwidth),
        }
    }

    /// Add a flow to the scheduler with a weight.
    pub fn add_flow(&mut self, flow: VariableLengthFlow, weight: f64) {
        self.flows.push(flow);
        self.weights.push(weight);
        self.total_weight += weight;
    }

    pub fn run(&mut self) {
        while self.tick() {}
        self.output_port.proceed_rest();
    }

    fn estimate_time(&self, flow_idx: &usize, pakcet: &Packet) -> f64 {
        let assumed_rate = self.weights[*flow_idx] / self.total_weight;
        pakcet.len as f64 / assumed_rate
    }
}

impl Tickable for WFQScheduler {
    fn tick(&mut self) -> bool {
        if self.flows.iter().all(|f| f.empty()) {
            return false;
        }

        // Add back if scheduled
        if let Some(idx) = self.schedule() {
            self.output_port.submit(self.flows[idx].pop_packet());
        }

        self.timer += 1;
        self.output_port.tick();

        assert!(self.flows.len() == self.weights.len());

        true
    }
}

impl Schedulable<Option<usize>> for WFQScheduler {
    /// Schedule the next flow to be served.
    /// Return the index of the flow to be served
    /// else None.
    fn schedule(&mut self) -> Option<usize> {
        let mut min_time = f64::INFINITY;
        let mut min_flow_idx = 0;
        for (idx, flow) in self.flows.iter().enumerate() {
            if flow.empty() {
                continue;
            }
            if let Some(packet) = flow.peek_packet(self.timer) {
                let time = self.estimate_time(&idx, &packet);
                if time < min_time {
                    min_time = time;
                    min_flow_idx = idx;
                } else if time == min_time {
                    // randomly choose one
                    if rand::random() {
                        min_flow_idx = idx;
                    }
                }
            }
        }

        if min_time == f64::INFINITY {
            return None;
        }

        Some(min_flow_idx)
    }
}

#[cfg(test)]
mod test {
    use crate::scheduling::{
        flow::{self, Flow},
        Packet,
    };

    #[test]
    fn wfq_test() {
        let mut wfq = super::WFQScheduler::new(1);

        let mut flow1 = flow::VariableLengthFlow::new();
        flow1.packet_arrive(Packet::new("p1", 1), 0);
        flow1.packet_arrive(Packet::new("p4", 1), 2);
        flow1.packet_arrive(Packet::new("p6", 1), 5);
        wfq.add_flow(flow1, 0.5f64);

        let mut flow2 = flow::VariableLengthFlow::new();
        flow2.packet_arrive(Packet::new("p2", 1), 0);
        flow2.packet_arrive(Packet::new("p5", 1), 3);
        flow2.packet_arrive(Packet::new("p9", 1), 7);
        wfq.add_flow(flow2, 0.25f64);

        let mut flow3 = flow::VariableLengthFlow::new();
        flow3.packet_arrive(Packet::new("p3", 1), 0);
        flow3.packet_arrive(Packet::new("p7", 1), 5);
        flow3.packet_arrive(Packet::new("p8", 1), 6);
        wfq.add_flow(flow3, 0.25f64);

        wfq.run();

        assert_eq!(wfq.timer, 9);

        let output = wfq.output_port.get_output();

        assert_eq!(output.len(), 9);
        assert_eq!(
            output,
            &vec![
                Packet::new("p1", 1),
                Packet::new("p2", 1),
                Packet::new("p4", 1),
                Packet::new("p3", 1),
                Packet::new("p5", 1),
                Packet::new("p6", 1),
                Packet::new("p7", 1),
                Packet::new("p8", 1),
                Packet::new("p9", 1),
            ]
        );
    }
}
