## MODIFIED Requirements

### Requirement: Concurrency and retry
The system SHALL download multiple queued items with a configurable bounded
concurrency and SHALL retry transient failures with exponential backoff.
Transient failures SHALL be classified as rate-limit responses (HTTP 429),
network errors (including timeouts), and HTTP 5xx responses; all other errors
SHALL be treated as permanent and SHALL NOT be retried. On each retry attempt
that requires a signed file URL, the system SHALL request a **fresh** signed URL
rather than reusing a previously fetched (possibly expired) one. Backoff SHALL be
exponential with added jitter, and SHALL honor a `Retry-After` header when the
API provides one on a rate-limit response. A failed or aborted download SHALL NOT
leave a stale partial file behind, and the system SHALL NOT re-download a track
whose destination file already exists.

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

#### Scenario: Skip already-downloaded track
- **WHEN** a track's destination file already exists at the time its download begins
- **THEN** the system skips re-downloading it and treats the item as complete

#### Scenario: Failure isolated
- **WHEN** one track in a multi-track download fails permanently
- **THEN** the system marks that item as errored and continues downloading the remaining items
