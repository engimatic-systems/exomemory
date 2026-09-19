use crate::{policy::Policy, types::*};
#[derive(Debug, Clone, Copy)]
pub struct Capabilities {
    pub read: bool,
    pub create: bool,
    pub overwrite: bool,
    pub feedback: bool,
    pub candidates: bool,
}
pub trait Storage {
    fn capabilities(&self) -> Capabilities;
    fn create(&mut self, policy: &Policy, context: &Context, draft: Draft) -> Result<Record>;
    fn read(&mut self, policy: &Policy, context: &Context, key: &str) -> Result<Record>;
    fn candidates(&mut self, _policy: &Policy, _event: &Event) -> Result<CandidateSnapshot> {
        Err(Error::new(
            ErrorCode::Unsupported,
            "candidate retrieval unavailable",
        ))
    }
}
