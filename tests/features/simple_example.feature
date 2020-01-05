Feature: SimpleExample

  Scenario: Adding a custom config file
    Given a new project
    When I add a custom config field
    Then I can access the config value
