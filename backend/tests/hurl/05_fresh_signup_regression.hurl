# ==============================================================================
# Intent-Based Hurl Suite: 05_fresh_signup_regression.hurl
# Fresh-User Signup Path (K-1473): register -> login -> me -> empty events
# list -> calendars. Fails if a brand-new account cannot list events
# (regression: migration 010's `-- Down` section was executed by sqlx as
# part of the upgrade, dropping has_conflict; repaired by migration 016).
# Requires: --variable run_id=<unique> so reruns don't collide on username.
# ==============================================================================

# 1. Register a brand-new user
POST {{base_url}}/api/v1/auth/register
Content-Type: application/json
{
  "username": "hurl_fresh_{{run_id}}@kestrel.dev",
  "password": "SuperSecurePassword2026!"
}

HTTP 201
[Asserts]
jsonpath "$.user_id" exists
[Captures]
fresh_user_id: jsonpath "$.user_id"


# 2. Login as the new user
POST {{base_url}}/api/v1/auth/token
Content-Type: application/json
{
  "username": "hurl_fresh_{{run_id}}@kestrel.dev",
  "password": "SuperSecurePassword2026!"
}

HTTP 200
[Asserts]
jsonpath "$.token" exists
jsonpath "$.user_id" == "{{fresh_user_id}}"
[Captures]
fresh_token: jsonpath "$.token"


# 3. Identity check
GET {{base_url}}/api/v1/auth/me
Authorization: Bearer {{fresh_token}}

HTTP 200
[Asserts]
jsonpath "$.user_id" == "{{fresh_user_id}}"


# 4. Fresh user lists events: must be 200 with an empty collection,
# not a 500 (missing has_conflict column).
GET {{base_url}}/api/v1/events?start_time=1700000000&end_time=1900000000
Authorization: Bearer {{fresh_token}}

HTTP 200
[Asserts]
jsonpath "$.events" isCollection
jsonpath "$.events" count == 0


# 5. Fresh user lists calendars
GET {{base_url}}/api/v1/calendars
Authorization: Bearer {{fresh_token}}

HTTP 200
[Asserts]
jsonpath "$.calendars" isCollection


# 6. Fresh user lists messages (same has_conflict column family)
GET {{base_url}}/api/v1/messages?page=1&per_page=10
Authorization: Bearer {{fresh_token}}

HTTP 200
