use super::FrameId;

pub trait Replacer {
    fn pin(&mut self, frame_id: FrameId);
    fn unpin(&mut self, frame_id: FrameId);
    fn victim(&mut self) -> Option<FrameId>;
}
