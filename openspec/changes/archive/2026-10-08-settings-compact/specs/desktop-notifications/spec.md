# Spec Delta

## MODIFIED Requirements

### Requirement: Notification setting
The settings SHALL offer a "Notify when done" option, on by
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
