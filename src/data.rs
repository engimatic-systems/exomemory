use crate::{policy::Policy, types::*};
pub trait Storage {
    fn create(&mut self, policy: &Policy, context: &Context, draft: Draft) -> Result<Record>;
    fn read(&mut self, policy: &Policy, context: &Context, key: &str) -> Result<Record>;
    fn candidates(&mut self, _policy: &Policy, _event: &Event) -> Result<CandidateSnapshot> {
        Err(Error::new(
            ErrorCode::Unsupported,
            "candidate retrieval unavailable",
        ))
    }
}
