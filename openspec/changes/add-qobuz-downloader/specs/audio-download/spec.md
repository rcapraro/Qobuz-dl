## ADDED Requirements

### Requirement: Select download quality
The system SHALL let the user choose a download quality tier — MP3 320 (5),
FLAC 16/44.1 (6), FLAC 24/≤96 (7), or FLAC 24/≤192 (27) — and SHALL request the
corresponding `format_id`.

#### Scenario: Requested quality delivered
- **WHEN** the user selects FLAC 24/≤96 for a track available at that tier
- **THEN** the system requests `format_id` 7 and downloads the hi-res FLAC

#### Scenario: Graceful downgrade
- **WHEN** the requested tier is not available for a track
- **THEN** the system downloads the best available tier and reports the actually delivered `bit_depth`/`sampling_rate`

### Requirement: Signed file URL request
The system SHALL request a signed file URL from `track/getFileUrl` using a
`request_ts` and an MD5 `request_sig` computed from the request parameters and
the `app_secret`, immediately before downloading.

#### Scenario: Fresh URL per download
- **WHEN** a track download begins
- **THEN** the system requests a new signed file URL just-in-time rather than reusing a previously fetched (expired) URL

### Requirement: Stream to disk with progress
The system SHALL stream downloaded audio to disk without buffering the entire
file in memory and SHALL emit progress updates during the transfer.

#### Scenario: Progress reported
- **WHEN** a large FLAC track is downloading
- **THEN** the system reports incremental progress (bytes/percentage) to the UI

### Requirement: Concurrency and retry
The system SHALL download multiple queued items with a configurable bounded
concurrency and SHALL retry with exponential backoff on rate-limit responses.

#### Scenario: Bounded concurrency
- **WHEN** the queue contains more items than the configured concurrency limit
- **THEN** the system downloads at most the configured number simultaneously and queues the rest

#### Scenario: Retry on rate limit
- **WHEN** the API returns a rate-limit response
- **THEN** the system waits with exponential backoff and retries before marking the item as failed

#### Scenario: Failure isolated
- **WHEN** one track in a multi-track download fails permanently
- **THEN** the system marks that item as errored and continues downloading the remaining items
