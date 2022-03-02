use super::Radio;
use anyhow::Result;

pub trait ProgMode: Copy {
    fn open<T: ProgMode>(&mut self, radio: &Radio<T>) -> Result<()>;
    fn get_vid_pid_eps() -> Vec<(u16, u16, u8)>;
    fn get_chunk_sz() -> usize;
    fn read<T: ProgMode>(self, radio: &Radio<T>, x: &mut [u8]) -> Result<()>;
    fn write<T: ProgMode>(self, radio: &Radio<T>, payload: &[u8]) -> Result<()>;
    fn drop<T: ProgMode>(self, radio: &Radio<T>);
}
