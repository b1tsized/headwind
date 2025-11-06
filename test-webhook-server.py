#!/usr/bin/env python3
"""
Simple webhook server for testing Headwind notifications.
Usage: python3 test-webhook-server.py
"""
import json
import hmac
import hashlib
from http.server import HTTPServer, BaseHTTPRequestHandler
from datetime import datetime

# Optional: Set this to test signature verification
SECRET = "test-secret-key"

class WebhookHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        content_length = int(self.headers['Content-Length'])
        body = self.rfile.read(content_length)

        print(f"\n{'='*80}")
        print(f"[{datetime.now().isoformat()}] Received webhook notification")
        print(f"{'='*80}")

        # Check signature if present
        signature = self.headers.get('X-Headwind-Signature')
        if signature:
            print(f"Signature: {signature}")
            if SECRET:
                # Verify signature
                expected = hmac.new(
                    SECRET.encode(),
                    body,
                    hashlib.sha256
                ).hexdigest()
                expected_sig = f"sha256={expected}"
                if signature == expected_sig:
                    print("✓ Signature verified!")
                else:
                    print(f"✗ Signature mismatch! Expected: {expected_sig}")

        # Parse and pretty-print the JSON payload
        try:
            payload = json.loads(body)
            print("\nPayload:")
            print(json.dumps(payload, indent=2))

            # Highlight key fields
            print(f"\nEvent: {payload.get('event')}")
            deployment = payload.get('deployment', {})
            print(f"Deployment: {deployment.get('namespace')}/{deployment.get('name')}")
            print(f"Image: {deployment.get('currentImage')} → {deployment.get('newImage')}")

        except json.JSONDecodeError:
            print("Body (not JSON):")
            print(body.decode('utf-8'))

        # Send response
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.wfile.write(b'{"status": "ok"}')

    def log_message(self, format, *args):
        # Suppress default logging
        pass

if __name__ == '__main__':
    port = 9999
    server = HTTPServer(('0.0.0.0', port), WebhookHandler)
    print(f"🚀 Webhook test server listening on http://localhost:{port}")
    print(f"📝 Set WEBHOOK_URL=http://localhost:{port} in your environment")
    if SECRET:
        print(f"🔐 Signature verification enabled with secret: {SECRET}")
    print("\nWaiting for notifications...\n")

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n\nShutting down...")
        server.shutdown()
