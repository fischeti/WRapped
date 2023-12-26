use crate::mail::Envelope;

#[derive(Debug)]
pub struct WR {
    // The Envelope of the WR that was sent
    pub sent: Envelope,
    // The Envelope of the WR reply that was received, if any
    pub reply: Option<Envelope>,
}

impl WR {
  pub fn new(sent: Envelope, reply: Option<Envelope>) -> Self {
      WR { sent, reply }
  }
}

#[derive(Debug)]
pub struct WRs {
  // All the WRs that were sent and received
  pub wrs: Vec<WR>,
}

impl WRs {
  pub fn new() -> Self {
      WRs {
          wrs: Vec::new(),
      }
  }

  pub fn push(&mut self, wr: WR) {
      self.wrs.push(wr);
  }

  pub fn pop(&mut self) -> Option<WR> {
      self.wrs.pop()
  }

  pub fn len(&self) -> usize {
      self.wrs.len()
  }

  pub fn is_empty(&self) -> bool {
      self.wrs.is_empty()
  }
}
