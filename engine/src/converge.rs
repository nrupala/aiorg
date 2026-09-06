use anyhow::{bail, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct Cycle {
    pub number: u32,
    pub gap: f64,
}

pub struct Converger {
    max_cycles: u32,
    max_stall: u32,
    prior_gap: Option<f64>,
    stall: u32,
    cycles: Vec<Cycle>,
}
impl Converger {
    pub fn new(max_cycles: u32, max_stall: u32) -> Self {
        Self {
            max_cycles,
            max_stall,
            prior_gap: None,
            stall: 0,
            cycles: Vec::new(),
        }
    }
    pub fn observe(&mut self, gap: f64) -> Result<Cycle> {
        if !gap.is_finite() || !(0.0..=1.0).contains(&gap) {
            bail!("gap must be between 0 and 1")
        }
        if self.cycles.len() as u32 >= self.max_cycles {
            bail!("maximum corrective cycles exceeded")
        }
        if let Some(prior) = self.prior_gap {
            if gap > prior {
                bail!("delta-V violation: {:.6} > {:.6}", gap, prior);
            }
            if (gap - prior).abs() < f64::EPSILON {
                self.stall += 1;
            } else {
                self.stall = 0;
            }
            if self.stall >= self.max_stall {
                bail!("convergence stalled")
            }
        }
        let cycle = Cycle {
            number: self.cycles.len() as u32 + 1,
            gap,
        };
        self.prior_gap = Some(gap);
        self.cycles.push(cycle.clone());
        Ok(cycle)
    }
    pub fn converged(&self) -> bool {
        self.prior_gap == Some(0.0)
    }
    pub fn cycles(&self) -> &[Cycle] {
        &self.cycles
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn delta_v_never_widens() {
        let mut c = Converger::new(8, 2);
        c.observe(0.8).unwrap();
        c.observe(0.4).unwrap();
        assert!(c.observe(0.5).is_err());
    }
    #[test]
    fn convergence_reaches_zero() {
        let mut c = Converger::new(8, 2);
        c.observe(0.8).unwrap();
        c.observe(0.2).unwrap();
        c.observe(0.0).unwrap();
        assert!(c.converged());
    }
    #[test]
    fn stall_escalates() {
        let mut c = Converger::new(8, 2);
        c.observe(0.5).unwrap();
        c.observe(0.5).unwrap();
        assert!(c.observe(0.5).is_err());
    }
}
