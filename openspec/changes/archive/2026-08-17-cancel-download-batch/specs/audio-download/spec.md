## ADDED Requirements

### Requirement: Cancel a download batch
The system SHALL allow a caller to cancel a running download batch. On
cancellation, tracks that have not started SHALL NOT begin, and tracks that are
mid-transfer SHALL stop transferring. Cancellation SHALL take effect promptly:
the system SHALL NOT wait out a pending retry backoff, however long the
remaining delay is, before honouring it. Cancelled work SHALL be reported
distinctly from failed work, so a caller can tell a track the user stopped from
a track that could not be downloaded. A cancelled batch SHALL signal its
completion to the caller exactly as an uncancelled batch does, and SHALL leave
no download work running once it has done so.

#### Scenario: In-flight transfers stop
- **WHEN** a batch is cancelled while tracks are transferring
- **THEN** those transfers stop and are reported as cancelled rather than as failed

#### Scenario: Not-yet-started tracks never begin
- **WHEN** a batch is cancelled while tracks are still waiting for a concurrency slot
- **THEN** those tracks are reported as cancelled without any network request being made for them

#### Scenario: Cancellation interrupts retry backoff
- **WHEN** a batch is cancelled while a track is waiting out a retry backoff, including a long server-supplied `Retry-After` delay
- **THEN** the wait is abandoned immediately rather than run to completion

#### Scenario: Completed tracks are unaffected
- **WHEN** a batch is cancelled after some tracks have already finished
- **THEN** those tracks keep their downloaded files and their completed outcome

#### Scenario: No work outlives a cancelled batch
- **WHEN** a batch has signalled completion after being cancelled
- **THEN** no download continues in the background — nothing further is written to disk, tagged, or reported

## MODIFIED Requirements

### Requirement: Concurrency and retry
The system SHALL download multiple queued items with a configurable bounded
concurrency and SHALL retry transient failures with exponential backoff.
Transient failures SHALL be classified as rate-limit responses (HTTP 429),
network errors (including timeouts), and HTTP 5xx responses; all other errors
SHALL be treated as permanent and SHALL NOT be retried. Cancellation SHALL NOT
be treated as a transient failure and SHALL NOT be retried. On each retry
attempt that requires a signed file URL, the system SHALL request a **fresh**
signed URL rather than reusing a previously fetched (possibly expired) one.
Backoff SHALL be exponential with added jitter, and SHALL honor a `Retry-After`
header when the API provides one on a rate-limit response. A download that
fails, is cancelled, or is otherwise interrupted SHALL NOT leave a stale
partial file behind — the partial file SHALL be removed however the transfer
ends, not only when it ends by returning an error — and the system SHALL NOT
re-download a track whose destination file already exists.

#### Scenario: Bounded concurrency
- **WHEN** the queue contains more items than the configured concurrency limit
- **THEN** the system downloads at most the configured number simultaneously and queues the rest

#### Scenario: Retry on rate limit
- **WHEN** the API returns a rate-limit response
- **THEN** the system waits with jittered exponential backoff (honoring `Retry-After` when present) and retries before marking the item as failed

#### Scenario: Fresh signed URL on retry
- **WHEN** a streaming attempt fails transiently and the download is retried
- **THEN** the system requests a new signed file URL for the next attempt instead of reusing the earlier (potentially expired) URL

#### Scenario: Permanent failure not retried
- **WHEN** a download fails with a permanent error (e.g. authentication, missing file URL, a non-429 4xx response)
- **THEN** the system marks the item as errored immediately without further retry attempts

#### Scenario: Partial file cleaned up on failure
- **WHEN** a download attempt errors after having created a partial (`.part`) file
- **THEN** the system removes the partial file so no incomplete or orphaned file remains

#### Scenario: Partial file cleaned up when a transfer is interrupted
- **WHEN** a transfer that has created a partial file is abandoned part-way rather than returning an error, as happens when the batch is cancelled
- **THEN** the partial file is still removed, and repeated interrupted attempts do not accumulate partial files beside the destination

#### Scenario: Skip already-downloaded track
- **WHEN** a track's destination file already exists at the time its download begins
- **THEN** the system skips re-downloading it and treats the item as complete

#### Scenario: Failure isolated
- **WHEN** one track in a multi-track download fails permanently
- **THEN** the system marks that item as errored and continues downloading the remaining items
