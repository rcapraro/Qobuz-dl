## MODIFIED Requirements

### Requirement: Signed file URL request
The system SHALL request a signed file URL from `track/getFileUrl` using a
`request_ts` and an MD5 `request_sig` computed from the request parameters and
the `app_secret`, immediately before downloading. When multiple candidate app
secrets are configured (e.g. from auto-detection), the system SHALL treat a
rejected signature as a signal to try the next candidate, and SHALL succeed as
long as any candidate produces an accepted signature. Signature-rejection
detection SHALL be case-insensitive.

#### Scenario: Fresh URL per download
- **WHEN** a track download begins
- **THEN** the system requests a new signed file URL just-in-time rather than reusing a previously fetched (expired) URL

#### Scenario: Recover across candidate secrets
- **WHEN** the first candidate secret produces a signature the API rejects (an "Invalid Request Signature" / `request_sig` error, in any letter case)
- **THEN** the system classifies it as a signature rejection and retries with the remaining candidate secrets, and the download proceeds once one is accepted

#### Scenario: All candidates rejected
- **WHEN** every candidate secret's signature is rejected
- **THEN** the system reports a signature failure prompting the user to verify the `app_secret`
