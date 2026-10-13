pub mod fir;
mod kernels;
pub mod org_playback;
pub mod organya;
pub mod pixtone;
pub mod pixtone_sfx;
mod polyphase;
pub mod stuff;
pub mod wav;
pub mod wave_bank;

#[derive(Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InterpolationMode {
    Nearest,
    Linear,
    Cosine,
    Cubic,
    Polyphase,
}
