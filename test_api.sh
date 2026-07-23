#!/bin/bash
set -e

cd "C:/Users/Sam/Consusson/Projects/Kestrel"

# Start server in background
./target/debug/backend.exe &
SERVER_PID=$!
echo "Server PID: $SERVER_PID"

# Wait for server to start
echo "Waiting for server to start..."
for i in $(seq 1 30); do
    if curl -s http://127.0.0.1:8080/api/v1/health > /dev/null 2>&1; then
        echo "Server is up!"
        break
    fi
    sleep 1
done

# Test health
echo ""
echo "=== HEALTH ==="
curl -s http://127.0.0.1:8080/api/v1/health

# Register user
echo ""
echo "=== REGISTER ==="
curl -s -X POST http://127.0.0.1:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"testpassword123"}'

# Get token
echo ""
echo "=== TOKEN ==="
TOKEN_RESP=$(curl -s -X POST http://127.0.0.1:8080/api/v1/auth/token \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"testpassword123"}')
echo "$TOKEN_RESP"

TOKEN=$(echo "$TOKEN_RESP" | grep -o '"token":"[^"]*"' | sed 's/"token":"//;s/"//')

if [ -z "$TOKEN" ]; then
    echo "Failed to get token, aborting tests"
    kill $SERVER_PID 2>/dev/null
    exit 1
fi

echo ""
echo "=== MESSAGES ==="
curl -s http://127.0.0.1:8080/api/v1/messages -H "Authorization: Bearer $TOKEN"

echo ""
echo "=== SEARCH ==="
curl -s "http://127.0.0.1:8080/api/v1/search?q=test" -H "Authorization: Bearer $TOKEN"

echo ""
echo "=== CALENDARS ==="
curl -s http://127.0.0.1:8080/api/v1/calendars -H "Authorization: Bearer $TOKEN"

echo ""
echo "=== EVENTS ==="
curl -s "http://127.0.0.1:8080/api/v1/events?start_time=0&end_time=9999999999" -H "Authorization: Bearer $TOKEN"

echo ""
echo "=== SYNC TRIGGER ==="
curl -s -X POST http://127.0.0.1:8080/api/v1/sync/trigger \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{}'

echo ""
echo "=== PROVIDERS ==="
curl -s http://127.0.0.1:8080/api/v1/providers

echo ""
echo "=== ALL TESTS COMPLETE ==="

# Kill server
kill $SERVER_PID 2>/dev/null
echo "Server stopped."
