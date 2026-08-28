# ==============================================================================
# Intent-Based Hurl Suite: 01_auth_and_security.hurl
# Self-Seeding Authentication, Security, Rate Limiting & Identity Verification
# ==============================================================================

# 1. Verify health check endpoint (Public)
GET {{base_url}}/api/v1/health

HTTP 200
[Asserts]
jsonpath "$.status" == "ok"
header "Content-Type" contains "application/json"


# 2. Query public providers catalog
GET {{base_url}}/api/v1/providers

HTTP 200
[Asserts]
jsonpath "$.providers" isCollection
jsonpath "$.providers" count >= 1
jsonpath "$.providers[0].id" exists
jsonpath "$.providers[0].name" exists


# 3. Negative Test: Attempt registration with short password (< 8 chars)
POST {{base_url}}/api/v1/auth/register
Content-Type: application/json
{
  "username": "short_pw_user@kestrel.dev",
  "password": "123"
}

HTTP 400
[Asserts]
jsonpath "$.error" contains "Password must be at least 8 characters"


# 4. Negative Test: Attempt registration with empty fields
POST {{base_url}}/api/v1/auth/register
Content-Type: application/json
{
  "username": "",
  "password": ""
}

HTTP 400


# 5. Self-Seed: Register a new primary test user
POST {{base_url}}/api/v1/auth/register
Content-Type: application/json
{
  "username": "hurl_poweruser@kestrel.dev",
  "password": "SuperSecurePassword2026!"
}

HTTP 201
[Asserts]
jsonpath "$.user_id" exists
[Captures]
seeded_user_id: jsonpath "$.user_id"


# 6. Negative Test: Duplicate username registration
POST {{base_url}}/api/v1/auth/register
Content-Type: application/json
{
  "username": "hurl_poweruser@kestrel.dev",
  "password": "SuperSecurePassword2026!"
}

HTTP 400


# 7. Negative Test: Login with incorrect password
POST {{base_url}}/api/v1/auth/token
Content-Type: application/json
{
  "username": "hurl_poweruser@kestrel.dev",
  "password": "WrongPasswordAttempt"
}

HTTP 401


# 8. Negative Test: Login with non-existent username
POST {{base_url}}/api/v1/auth/token
Content-Type: application/json
{
  "username": "non_existent_user_9999@kestrel.dev",
  "password": "SuperSecurePassword2026!"
}

HTTP 401


# 9. Successful Login: Obtain JWT Bearer Token
POST {{base_url}}/api/v1/auth/token
Content-Type: application/json
{
  "username": "hurl_poweruser@kestrel.dev",
  "password": "SuperSecurePassword2026!"
}

HTTP 200
[Asserts]
jsonpath "$.token" exists
jsonpath "$.user_id" == "{{seeded_user_id}}"
[Captures]
auth_token: jsonpath "$.token"


# 10. Negative Test: Access protected /me without Authorization header
GET {{base_url}}/api/v1/auth/me

HTTP 401


# 11. Negative Test: Access protected /me with forged / malformed JWT
GET {{base_url}}/api/v1/auth/me
Authorization: Bearer forged.invalid.token

HTTP 401


# 12. Identity Verification: Query /me with valid Bearer Token
GET {{base_url}}/api/v1/auth/me
Authorization: Bearer {{auth_token}}

HTTP 200
[Asserts]
jsonpath "$.user_id" == "{{seeded_user_id}}"


# 13. Alternative Token Passing: Query /me with query parameter token (for SSE/EventSource)
GET {{base_url}}/api/v1/auth/me?token={{auth_token}}

HTTP 200
[Asserts]
jsonpath "$.user_id" == "{{seeded_user_id}}"


# 14. Alternative Token Passing: Query /me with Cookie header
GET {{base_url}}/api/v1/auth/me
Cookie: kestrel_token={{auth_token}}

HTTP 200
[Asserts]
jsonpath "$.user_id" == "{{seeded_user_id}}"
