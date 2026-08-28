# ==============================================================================
# Intent-Based Hurl Suite: 03_calendar_and_events.hurl
# Calendar Discovery, Event Lifecycle, Window Queries, RSVP & Soft Deletion
# ==============================================================================

# 1. Login to obtain session token
POST {{base_url}}/api/v1/auth/token
Content-Type: application/json
{
  "username": "demo_user",
  "password": "password"
}

HTTP 200
[Captures]
token: jsonpath "$.token"
user_id: jsonpath "$.user_id"


# 2. List user calendars
GET {{base_url}}/api/v1/calendars
Authorization: Bearer {{token}}

HTTP 200
[Asserts]
jsonpath "$.calendars" isCollection
jsonpath "$.calendars" count >= 1
jsonpath "$.calendars[0].id" exists
jsonpath "$.calendars[0].name" exists
[Captures]
cal_id: jsonpath "$.calendars[0].id"
cal_name: jsonpath "$.calendars[0].name"


# 3. Fetch single calendar detail
GET {{base_url}}/api/v1/calendars/{{cal_id}}
Authorization: Bearer {{token}}

HTTP 200
[Asserts]
jsonpath "$.id" == "{{cal_id}}"
jsonpath "$.name" == "{{cal_name}}"


# 4. Negative Test: Fetch non-existent calendar ID
GET {{base_url}}/api/v1/calendars/00000000-0000-0000-0000-000000000000
Authorization: Bearer {{token}}

HTTP 404


# 5. Create new calendar event
POST {{base_url}}/api/v1/events
Authorization: Bearer {{token}}
Content-Type: application/json
{
  "calendar_id": "{{cal_id}}",
  "title": "Hurl Executive Product Review",
  "description": "Comprehensive review of Q3 milestones and Specta v2 integration.",
  "location": "HQ Conference Room 4A & Google Meet",
  "start_time": 1780000000,
  "end_time": 1780003600,
  "is_all_day": false,
  "recurrence_rules": "FREQ=WEEKLY;INTERVAL=1;COUNT=5",
  "attendees": "[{\"name\": \"Engineering Lead\", \"email\": \"lead@kestrel.dev\", \"status\": \"accepted\"}]"
}

HTTP 201
[Asserts]
jsonpath "$.id" exists
jsonpath "$.calendar_id" == "{{cal_id}}"
jsonpath "$.title" == "Hurl Executive Product Review"
[Captures]
event_id: jsonpath "$.id"


# 6. Fetch single event details
GET {{base_url}}/api/v1/events/{{event_id}}
Authorization: Bearer {{token}}

HTTP 200
[Asserts]
jsonpath "$.id" == "{{event_id}}"
jsonpath "$.title" == "Hurl Executive Product Review"
jsonpath "$.location" == "HQ Conference Room 4A & Google Meet"


# 7. Query events with specific time-window filter matching created event
GET {{base_url}}/api/v1/events?start_time=1779000000&end_time=1781000000&calendar_id={{cal_id}}
Authorization: Bearer {{token}}

HTTP 200
[Asserts]
jsonpath "$.events" isCollection
jsonpath "$.events" count >= 1


# 8. Query events with time window outside any events (empty list)
GET {{base_url}}/api/v1/events?start_time=1999990000&end_time=1999999000
Authorization: Bearer {{token}}

HTTP 200
[Asserts]
jsonpath "$.events" isCollection
jsonpath "$.events" count == 0


# 9. Patch event: update location, title, and end time
PATCH {{base_url}}/api/v1/events/{{event_id}}
Authorization: Bearer {{token}}
Content-Type: application/json
{
  "title": "Hurl Executive Review (Extended)",
  "location": "Virtual Google Meet",
  "end_time": 1780007200
}

HTTP 200
[Asserts]
jsonpath "$.id" == "{{event_id}}"
jsonpath "$.title" == "Hurl Executive Review (Extended)"
jsonpath "$.location" == "Virtual Google Meet"


# 10. Soft-delete calendar event
DELETE {{base_url}}/api/v1/events/{{event_id}}
Authorization: Bearer {{token}}

HTTP 204


# 11. Verify soft-deleted event title marked as deleted
GET {{base_url}}/api/v1/events/{{event_id}}
Authorization: Bearer {{token}}

HTTP 200
[Asserts]
jsonpath "$.id" == "{{event_id}}"
jsonpath "$.title" == "[deleted]"
