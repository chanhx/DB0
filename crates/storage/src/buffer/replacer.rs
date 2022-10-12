use super::BufferId;

pub trait Replacer {
    fn pin(&mut self, buffer_id: BufferId);
    fn unpin(&mut self, buffer_id: BufferId);
    fn victim(&mut self) -> Option<BufferId>;
}
