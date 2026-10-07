//! Whether a finished download batch warrants a desktop notification, and its
//! wording. The OS call itself lives in `tasks::notify`.

#[derive(Debug, Clone, Copy)]
pub(super) struct BatchOutcome {
    pub(super) downloaded: usize,
    pub(super) failed: usize,
    pub(super) cancelled: bool,
}

/// The notification's summary and body, or `None` when there is nothing to
/// tell: the user cancelled (they just acted in the app), turned notifications
/// off, or is looking at the app, where the status line already says it.
pub(super) fn notification(
    outcome: BatchOutcome,
    enabled: bool,
    focused: bool,
) -> Option<(String, String)> {
    if outcome.cancelled || !enabled || focused {
        return None;
    }
    let tracks = |n: usize| {
        if n == 1 {
            "1 track".to_owned()
        } else {
            format!("{n} tracks")
        }
    };
    Some(if outcome.failed == 0 {
        (
            "Downloads finished".to_owned(),
            format!("{} downloaded", tracks(outcome.downloaded)),
        )
    } else {
        (
            "Downloads finished with errors".to_owned(),
            format!(
                "{} downloaded, {} failed",
                outcome.downloaded, outcome.failed
            ),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outcome(downloaded: usize, failed: usize) -> BatchOutcome {
        BatchOutcome {
            downloaded,
            failed,
            cancelled: false,
        }
    }

    fn texts(summary: &str, body: &str) -> Option<(String, String)> {
        Some((summary.to_owned(), body.to_owned()))
    }

    #[test]
    fn all_downloaded() {
        assert_eq!(
            notification(outcome(12, 0), true, false),
            texts("Downloads finished", "12 tracks downloaded")
        );
    }

    #[test]
    fn single_track_is_singular() {
        assert_eq!(
            notification(outcome(1, 0), true, false),
            texts("Downloads finished", "1 track downloaded")
        );
    }

    #[test]
    fn some_failed() {
        assert_eq!(
            notification(outcome(10, 2), true, false),
            texts("Downloads finished with errors", "10 downloaded, 2 failed")
        );
    }

    #[test]
    fn cancelled_batch_is_silent() {
        let cancelled = BatchOutcome {
            cancelled: true,
            ..outcome(3, 0)
        };
        assert_eq!(notification(cancelled, true, false), None);
    }

    #[test]
    fn disabled_is_silent() {
        assert_eq!(notification(outcome(3, 0), false, false), None);
    }

    #[test]
    fn focused_window_is_silent() {
        assert_eq!(notification(outcome(3, 1), true, true), None);
    }
}
