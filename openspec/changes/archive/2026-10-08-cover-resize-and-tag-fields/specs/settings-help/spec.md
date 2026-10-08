# Spec Delta

## MODIFIED Requirements

### Requirement: Options help

The Options card help SHALL explain the quality selector, the concurrency control,
and the Cover art selector.

#### Scenario: Options explained

- **WHEN** the user opens the Options help
- **THEN** it describes the available quality tiers and notes that the delivered
  quality may be downgraded by the service, the meaning of the concurrency value and
  its allowed range, and what each Cover art choice embeds: nothing for Off, Qobuz's
  cover as delivered for 600 px, and that cover scaled down for 400 px and 500 px
