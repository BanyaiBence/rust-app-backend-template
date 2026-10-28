# Walk-through of the API, runnable as a smoke test against any running server:
#   make smoke                          (http://localhost:3000)
#   make smoke BASE_URL=https://...     (another deployment)
# Variables: base_url, username (must be unused; `make smoke` makes a unique one).
# Hurl: https://hurl.dev — each entry is a request followed by assertions on its response.

# Liveness and readiness
GET {{base_url}}/health
HTTP 200
[Asserts]
jsonpath "$.status" == "ok"
header "x-request-id" exists

GET {{base_url}}/ready
HTTP 200

# Register, then log in to get a bearer token
POST {{base_url}}/auth/register
{
  "username": "{{username}}",
  "password": "password123"
}
HTTP 201
[Asserts]
jsonpath "$.username" == "{{username}}"
jsonpath "$.password_hash" not exists

POST {{base_url}}/auth/login
{
  "username": "{{username}}",
  "password": "password123"
}
HTTP 200
[Captures]
token: jsonpath "$.token"

GET {{base_url}}/auth/me
Authorization: Bearer {{token}}
HTTP 200
[Asserts]
jsonpath "$.username" == "{{username}}"

# Errors carry a stable code
GET {{base_url}}/notes
HTTP 401
[Asserts]
jsonpath "$.code" == "unauthorized"

# Notes: create, list, update, delete
POST {{base_url}}/notes
Authorization: Bearer {{token}}
{
  "title": "Groceries",
  "body": "milk, eggs"
}
HTTP 201
[Captures]
note_id: jsonpath "$.id"

GET {{base_url}}/notes?limit=10
Authorization: Bearer {{token}}
HTTP 200
[Asserts]
jsonpath "$.items" count == 1
jsonpath "$.limit" == 10

PATCH {{base_url}}/notes/{{note_id}}
Authorization: Bearer {{token}}
{
  "title": "Groceries for Sunday"
}
HTTP 200
[Asserts]
jsonpath "$.title" == "Groceries for Sunday"
jsonpath "$.body" == "milk, eggs"

POST {{base_url}}/notes
Authorization: Bearer {{token}}
{
  "title": "   "
}
HTTP 422
[Asserts]
jsonpath "$.code" == "validation_failed"

DELETE {{base_url}}/notes/{{note_id}}
Authorization: Bearer {{token}}
HTTP 204

GET {{base_url}}/notes/{{note_id}}
Authorization: Bearer {{token}}
HTTP 404
[Asserts]
jsonpath "$.code" == "not_found"
