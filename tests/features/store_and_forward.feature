Feature: Store and Forward Capability

  Background:
    Given an edge node with ID "edge1" is configured
    And the broker is online

  Scenario: Storing messages when network is unavailable
    Given the network is unavailable
    When a producer sends the following messages:
      | topic       | payload                 |
      | sensor_data | {"temperature": 25.5}   |
      | sensor_data | {"humidity": 60}        |
    Then 2 messages should be stored locally
    And the local storage should contain 2 pending messages

  Scenario: Syncing messages when network is restored
    Given the local storage contains 3 pending messages
    When the network becomes available
    And the edge node syncs with the broker
    Then all pending messages should be sent to the broker
    And the local storage should contain 0 pending messages

  Scenario: Preserving message order during sync
    Given the local storage contains the following messages:
      | topic | payload | timestamp |
      | temp  | 20.5    | 1000      |
      | temp  | 21.0    | 2000      |
      | temp  | 22.5    | 3000      |
    When the network becomes available
    And the edge node syncs with the broker
    Then the broker should receive the messages in the correct order

  Scenario: Handling network interruption during sync
    Given the local storage contains 5 pending messages
    And the network becomes unstable
    When the edge node attempts to sync with the broker
    Then at least 1 message should be sent to the broker
    And the remaining messages should still be in local storage

  Scenario: Configuring edge node sync interval
    When the sync interval is set to 5 minutes
    Then the edge node should attempt to sync every 5 minutes

  Scenario: Handling message expiration
    Given a message with expiration time of 1 hour is stored
    When 2 hours have passed
    Then the expired message should be removed from local storage

  Scenario: Seamless Producer API usage
    Given a producer is configured to use an edge node
    When the producer sends a message
    Then the message should be handled by the edge node
    And the producer should receive a success status

  Scenario: Handling full local storage
    Given the local storage is near capacity
    When a producer sends a large message
    Then the edge node should reject the message
    And an error should be returned to the producer

  Scenario: Prioritizing message sync based on importance
    Given the local storage contains messages with different priorities
    When the edge node syncs with limited bandwidth
    Then high priority messages should be synced first

  Scenario: Recovering from broker failure during sync
    Given the local storage contains 10 pending messages
    And the broker fails during sync after 5 messages
    When the broker comes back online
    And the edge node resumes sync
    Then all 10 messages should eventually be sent to the broker