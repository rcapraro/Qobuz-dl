//! The status line's message model: what happened, classified by kind so the
//! view can color it by meaning.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StatusKind {
    Info,
    Progress,
    Success,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Status {
    pub(super) kind: StatusKind,
    pub(super) text: String,
}

impl Status {
    fn new(kind: StatusKind, text: impl Into<String>) -> Self {
        Self {
            kind,
            text: text.into(),
        }
    }

    pub(super) fn info(text: impl Into<String>) -> Self {
        Self::new(StatusKind::Info, text)
    }

    pub(super) fn progress(text: impl Into<String>) -> Self {
        Self::new(StatusKind::Progress, text)
    }

    pub(super) fn success(text: impl Into<String>) -> Self {
        Self::new(StatusKind::Success, text)
    }

    pub(super) fn error(text: impl Into<String>) -> Self {
        Self::new(StatusKind::Error, text)
    }
}
