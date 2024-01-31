use bitflags::bitflags;

bitflags! {
    /// Flags specifying a collection of [`PresentMode`]s.
    ///
    /// The description of the variants is written for the variants of [`PresentMode`].
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PresentModes: u32 {
        const IMMEDIATE = 1 << 0;
        const MAILBOX = 1 << 1;
        const FIFO = 1 << 2;
        const FIFO_RELAXED = 1 << 3;
    }
}

impl From<PresentMode> for PresentModes {
    fn from(value: PresentMode) -> Self {
        match value {
            PresentMode::Immediate => Self::IMMEDIATE,
            PresentMode::Mailbox => Self::MAILBOX,
            PresentMode::Fifo => Self::FIFO,
            PresentMode::FifoRelaxed => Self::FIFO_RELAXED,
        }
    }
}

// TODO: add documentation on the variants.
/// A mode that the presentation engine can operate in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PresentMode {
    Immediate,
    Mailbox,
    Fifo,
    FifoRelaxed,
}
