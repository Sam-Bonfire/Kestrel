# ==============================================================================
# Intent-Based Hurl Suite: 02_mail_lifecycle.hurl
# Comprehensive Mail Triage, Search, Actions, Labels, Snooze & EML Export
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


# 2. List messages (Default pagination: page 1, 20 per page)
GET {{base_url}}/api/v1/messages?page=1&per_page=20
Authorization: Bearer {{token}}

HTTP 200
[Asserts]
jsonpath "$.messages" isCollection
jsonpath "$.total" >= 1
jsonpath "$.messages[0].id" exists
jsonpath "$.messages[0].subject" exists
jsonpath "$.messages[0].sender_email" exists
[Captures]
msg_1_id: jsonpath "$.messages[0].id"
msg_2_id: jsonpath "$.messages[1].id"
msg_1_thread_id: jsonpath "$.messages[0].thread_id"


# 3. List messages with page 2 offset and small page size
GET {{base_url}}/api/v1/messages?page=2&per_page=5
Authorization: Bearer {{token}}

HTTP 200
[Asserts]
jsonpath "$.messages" isCollection
jsonpath "$.messages" count <= 5


# 4. Filter messages by folder: 'inbox'
GET {{base_url}}/api/v1/messages?folder=inbox&per_page=10
Authorization: Bearer {{token}}

HTTP 200
[Asserts]
jsonpath "$.messages" isCollection


# 5. Filter messages by virtual folder: 'trash'
GET {{base_url}}/api/v1/messages?folder=trash&per_page=10
Authorization: Bearer {{token}}

HTTP 200
[Asserts]
jsonpath "$.messages" isCollection


# 6. Fetch single message full detail
GET {{base_url}}/api/v1/messages/{{msg_1_id}}
Authorization: Bearer {{token}}

HTTP 200
[Asserts]
jsonpath "$.id" == "{{msg_1_id}}"
jsonpath "$.sender_email" exists
jsonpath "$.date_received" exists
jsonpath "$.body_html" exists


# 7. Negative Test: Fetch non-existent message ID
GET {{base_url}}/api/v1/messages/00000000-0000-0000-0000-000000000000
Authorization: Bearer {{token}}

HTTP 404


# 8. Mark message as Read
POST {{base_url}}/api/v1/messages/{{msg_1_id}}/read
Authorization: Bearer {{token}}
Content-Type: application/json
{
  "read": true
}

HTTP 204


# 9. Toggle Star: Star the message
POST {{base_url}}/api/v1/messages/{{msg_1_id}}/star
Authorization: Bearer {{token}}
Content-Type: application/json
{
  "is_starred": true
}

HTTP 204


# 10. Update Custom Labels
POST {{base_url}}/api/v1/messages/{{msg_1_id}}/labels
Authorization: Bearer {{token}}
Content-Type: application/json
{
  "labels": ["INBOX", "PROJECT_ROADMAP", "URGENT"]
}

HTTP 204


# 11. List all user labels catalog
GET {{base_url}}/api/v1/labels
Authorization: Bearer {{token}}

HTTP 200
[Asserts]
jsonpath "$" isCollection


# 12. Patch Label Metadata (Color & Name)
PATCH {{base_url}}/api/v1/labels
Authorization: Bearer {{token}}
Content-Type: application/json
{
  "name": "PROJECT_ROADMAP",
  "color": "#FF5722",
  "icon": "folder"
}

HTTP 200


# 13. Snooze Message for 24 Hours
POST {{base_url}}/api/v1/messages/{{msg_1_id}}/snooze
Authorization: Bearer {{token}}
Content-Type: application/json
{
  "snoozed_until": 1893456000
}

HTTP 204


# 14. Unsnooze Message
POST {{base_url}}/api/v1/messages/{{msg_1_id}}/unsnooze
Authorization: Bearer {{token}}
Content-Type: application/json
{}

HTTP 204


# 15. Export Raw EML MIME Format
GET {{base_url}}/api/v1/messages/{{msg_1_id}}/raw
Authorization: Bearer {{token}}

HTTP 200
[Asserts]
header "Content-Type" contains "message/rfc822"
header "Content-Disposition" contains "attachment"


# 16. Mute Email Thread
POST {{base_url}}/api/v1/messages/{{msg_1_id}}/mute
Authorization: Bearer {{token}}
Content-Type: application/json
{}

HTTP 204


# 17. Report Phishing
POST {{base_url}}/api/v1/messages/{{msg_1_id}}/report-phishing
Authorization: Bearer {{token}}
Content-Type: application/json
{}

HTTP 204


# 18. Full-Text Search: Query across indexed messages
GET {{base_url}}/api/v1/search?q=Invoice&limit=10
Authorization: Bearer {{token}}

HTTP 200
[Asserts]
jsonpath "$.messages" isCollection


# 19. Full-Text Search: Query with no matches returns empty collection
GET {{base_url}}/api/v1/search?q=xyznonexistentterm99999
Authorization: Bearer {{token}}

HTTP 200
[Asserts]
jsonpath "$.messages" isCollection


# 20. Bulk Action: Archive multiple messages simultaneously
POST {{base_url}}/api/v1/messages/bulk
Authorization: Bearer {{token}}
Content-Type: application/json
{
  "message_ids": ["{{msg_1_id}}", "{{msg_2_id}}"],
  "action": "archive"
}

HTTP 204


# 21. Bulk Action: Star multiple messages simultaneously
POST {{base_url}}/api/v1/messages/bulk
Authorization: Bearer {{token}}
Content-Type: application/json
{
  "message_ids": ["{{msg_1_id}}", "{{msg_2_id}}"],
  "action": "star",
  "action_value": true
}

HTTP 204


# 22. Block Sender
POST {{base_url}}/api/v1/senders/block
Authorization: Bearer {{token}}
Content-Type: application/json
{
  "email": "spammer@external-bad-domain.com"
}

HTTP 204


# 23. Move Message to Trash
POST {{base_url}}/api/v1/messages/{{msg_2_id}}/trash
Authorization: Bearer {{token}}
Content-Type: application/json
{}

HTTP 204
