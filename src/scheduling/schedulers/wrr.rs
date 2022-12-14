use crate::scheduling::{
    flow::{FixedLengthFlow, Flow},
    Port, Schedulable, Tickable,
};

/// Weighted Round Robin (WRR) Scheduler
pub struct WRRScheduler {
    timer: usize,
    weights: Vec<usize>,
    current_weight: Vec<usize>,
    flows: Vec<FixedLengthFlow>,
    output_port: Port,
}

impl WRRScheduler {
    pub fn new(bandwidth: usize) -> WRRScheduler {
        WRRScheduler {
            timer: 0,
            weights: Vec::new(),
            current_weight: Vec::new(),
            flows: Vec::new(),
            output_port: Port::new(0, bandwidth),
        }
    }

    pub fn add_flow(&mut self, flow: FixedLengthFlow, weight: usize) {
        self.flows.push(flow);
        self.weights.push(weight);
        self.current_weight.push(weight);
    }

    pub fn run(&mut self) {
        while self.tick() {}
        self.output_port.proceed_rest()
    }
}

impl Tickable for WRRScheduler {
    fn tick(&mut self) -> bool {
        if self.flows.iter().all(|f| f.empty()) {
            return false;
        }

        if self.schedule() {
            self.current_weight = self.weights.clone();
        }

        self.timer += 1;
        self.output_port.tick();

        if self.timer > 100 {
            panic!("WRRScheduler::tick() is stuck in an infinite loop");
        }

        true
    }
}

impl Schedulable<bool> for WRRScheduler {
    fn schedule(&mut self) -> bool {
        for i in 0..self.flows.len() {
            if self.flows[i].empty() {
                continue;
            }
            if self.current_weight[i] > 0 {
                if let Some(_packet) = self.flows[i].peek_packet(self.timer) {
                    self.current_weight[i] -= 1;
                    self.output_port.submit(self.flows[i].pop_packet());
                }
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod test {
    use crate::scheduling::{flow::FixedLengthFlow, Packet};

    use super::WRRScheduler;

    #[test]
    fn wrr_test() {
        let mut wrr = WRRScheduler::new(1);

        let mut flow1 = FixedLengthFlow::new(1);
        flow1.add_packet("p1", 0);
        flow1.add_packet("p4", 1);
        flow1.add_packet("p6", 2);
        flow1.add_packet("p8", 4);
        flow1.add_packet("p11", 6);

        let mut flow2 = FixedLengthFlow::new(1);
        flow2.add_packet("p2", 0);
        flow2.add_packet("p5", 1);
        flow2.add_packet("p9", 5);
        flow2.add_packet("p13", 7);

        let mut flow3 = FixedLengthFlow::new(1);
        flow3.add_packet("p3", 0);
        flow3.add_packet("p7", 2);
        flow3.add_packet("p10", 5);
        flow3.add_packet("p12", 6);

        wrr.add_flow(flow1, 2);
        wrr.add_flow(flow2, 1);
        wrr.add_flow(flow3, 1);

        wrr.run();

        assert_eq!(wrr.timer, 16);

        let output = wrr.output_port.get_output();

        assert_eq!(output.len(), 13);
        assert_eq!(
            output,
            &vec![
                Packet::new("p1", 1),
                Packet::new("p4", 1),
                Packet::new("p2", 1),
                Packet::new("p3", 1),
                Packet::new("p6", 1),
                Packet::new("p8", 1),
                Packet::new("p5", 1),
                Packet::new("p7", 1),
                Packet::new("p11", 1),
                Packet::new("p9", 1),
                Packet::new("p10", 1),
                Packet::new("p13", 1),
                Packet::new("p12", 1),
            ]
        );
    }
}
