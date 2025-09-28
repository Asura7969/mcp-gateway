#!/bin/bash

# Test script for connection tracking API endpoints

echo "Testing connection tracking API endpoints..."

# Test getting connection count for a specific endpoint (using a random UUID)
ENDPOINT_ID="00000000-0000-0000-0000-000000000000"
echo "Testing endpoint connection count for $ENDPOINT_ID..."
curl -X GET "http://localhost:3000/api/connections/endpoint/$ENDPOINT_ID/count" \
  -H "Content-Type: application/json" \
  | jq .

# Test getting connection logs for a specific endpoint
echo "Testing endpoint connections for $ENDPOINT_ID..."
curl -X GET "http://localhost:3000/api/connections/endpoint/$ENDPOINT_ID" \
  -H "Content-Type: application/json" \
  | jq .

# Test getting time series connection counts
echo "Testing time series connection counts..."
curl -X GET "http://localhost:3000/api/connections/time-series" \
  -H "Content-Type: application/json" \
  | jq .

echo "Test completed."