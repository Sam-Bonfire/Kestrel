# ==============================================================================
# Intent-Based Hurl Suite: 04_settings_contacts_webhooks.hurl
# Settings, Preferences, Contacts Autocomplete, Sync Daemons & Webhook Handshakes
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


# 2. List connected provider accounts
GET {{base_url}}/api/v1/accounts
Authorization: Bearer {{token}}

HTTP 200
[Asserts]
jsonpath "$" isCollection
jsonpath "$[0].id" exists
jsonpath "$[0].provider" exists
[Captures]
account_id: jsonpath "$[0].id"


# 3. Retrieve user cross-device preferences
GET {{base_url}}/api/settings
Authorization: Bearer {{token}}

HTTP 200


# 4. Mutate user settings (theme, density, intervals)
PUT {{base_url}}/api/settings
Authorization: Bearer {{token}}
Content-Type: application/json
{
  "mailDenseMode": true,
  "mailDefaultLandingView": "inbox",
  "syncInterval": 120,
  "theme": "dark"
}

HTTP 200
[Asserts]
jsonpath "$.mailDenseMode" == true
jsonpath "$.mailDefaultLandingView" == "inbox"
jsonpath "$.syncInterval" == 120
jsonpath "$.theme" == "dark"


# 5. Search contacts autocomplete (name match)
GET {{base_url}}/api/contacts/search?q=Alice&limit=5
Authorization: Bearer {{token}}

HTTP 200
[Asserts]
jsonpath "$" isCollection


# 6. Search contacts autocomplete (email domain match)
GET {{base_url}}/api/contacts/search?q=example.com&limit=10
Authorization: Bearer {{token}}

HTTP 200
[Asserts]
jsonpath "$" isCollection


# 7. Search contacts autocomplete (empty query prefix / non-matching)
GET {{base_url}}/api/contacts/search?q=zzznonexistentcontact
Authorization: Bearer {{token}}

HTTP 200
[Asserts]
jsonpath "$" isCollection
jsonpath "$" count == 0


# 8. Trigger manual sync cycle
POST {{base_url}}/api/v1/sync/trigger
Authorization: Bearer {{token}}
Content-Type: application/json
{}

HTTP 200
[Asserts]
jsonpath "$.status" == "triggered"


# 9. Ingest Google PubSub Webhook notification
POST {{base_url}}/api/webhooks/google
Content-Type: application/json
{
  "message": {
    "data": "eyJlbWFpbEFkZHJlc3MiOiJkZW1vQGtlc3RyZWwubG9jYWwiLCJoaXN0b3J5SWQiOiIxMjM0NSJ9",
    "messageId": "pubsub-msg-001"
  }
}

HTTP 200


# 10. Ingest Microsoft Graph Webhook Handshake Validation
POST {{base_url}}/api/webhooks/microsoft?validationToken=HandshakeValidationToken12345
Content-Type: text/plain
validationToken=HandshakeValidationToken12345

HTTP 200
