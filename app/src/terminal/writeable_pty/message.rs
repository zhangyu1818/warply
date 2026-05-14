use crate::terminal::SizeInfo;
use std::borrow::Cow;

/// Messages that may be sent to the `EventLoop`.
#[derive(Debug)]
pub enum Message {
    /// Data that should be written to the PTY.
    Input(Cow<'static, [u8]>),

    /// Indicates that the `EventLoop` should be shut down.
    Shutdown,

    /// Indicates that the child process has exited.
    ChildExited,

    /// Instruction to resize the PTY.
    Resize(SizeInfo),
}
