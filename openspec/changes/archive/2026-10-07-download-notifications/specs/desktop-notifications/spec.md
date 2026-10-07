# Spec Delta

## Purpose

Tell the user, outside the app window, when long-running background work
such as a download batch has ended and how it went, so they need not keep
checking the app.

## ADDED Requirements

### Requirement: Notify when a download batch finishes
When a download batch ends without being cancelled, the system SHALL post an
OS desktop notification stating that downloads finished, how many tracks
were downloaded, and how many failed when any did. It SHALL NOT notify for a
cancelled batch, nor while the app window has focus.

#### Scenario: All tracks downloaded
- **WHEN** a batch of 12 tracks finishes with every track downloaded while the app window does not have focus
- **THEN** a desktop notification says downloads finished and that 12 tracks were downloaded

#### Scenario: Some tracks failed
- **WHEN** a batch finishes with 10 tracks downloaded and 2 failed while the app window does not have focus
- **THEN** a desktop notification says downloads finished with errors, 10 downloaded and 2 failed

#### Scenario: Cancelled batch
- **WHEN** the user cancels a batch and it stops
- **THEN** no desktop notification is posted

#### Scenario: App in focus
- **WHEN** a batch finishes while the app window has focus
- **THEN** no desktop notification is posted, and the status line reports the outcome as before

### Requirement: Notification setting
The settings SHALL offer a "Notify when downloads finish" option, on by
default and persisted with the other settings. When it is off, the system
SHALL NOT post download notifications.

#### Scenario: Default on
- **WHEN** the app runs with no saved preference for notifications
- **THEN** download notifications are enabled

#### Scenario: Turned off
- **WHEN** the user turns the option off, saves, and a batch then finishes while the window does not have focus
- **THEN** no desktop notification is posted

#### Scenario: Preference survives restart
- **WHEN** the user turns the option off, saves, and relaunches the app
- **THEN** the option is still off

### Requirement: Notification failures stay contained
A failure to post a notification, including on a platform or session where
notifications are unavailable, SHALL NOT affect the batch outcome, the
queue, or the status line, and SHALL NOT block the interface.

#### Scenario: Notification service unavailable
- **WHEN** a batch finishes and the system notification service rejects or cannot receive the notification
- **THEN** the queue and status line show the batch outcome as usual and the app stays responsive
