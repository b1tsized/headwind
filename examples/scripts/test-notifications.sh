#!/bin/bash
set -e

echo "🧪 Headwind Notification Testing Guide"
echo "========================================"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Please run this script from the headwind directory"
    exit 1
fi

echo "This script will help you test Headwind notifications."
echo "You'll need two terminal windows."
echo ""

echo "📋 Step 1: Choose your notification backend"
echo ""
echo "Option A - Local Webhook Server (recommended for testing):"
echo "  Terminal 1: python3 test-webhook-server.py"
echo "  Terminal 2: export WEBHOOK_ENABLED=true"
echo "              export WEBHOOK_URL=http://localhost:8080"
echo "              export WEBHOOK_SECRET=test-secret-key"
echo ""

echo "Option B - webhook.site (easy, no setup):"
echo "  1. Visit https://webhook.site"
echo "  2. Copy your unique URL"
echo "  3. export WEBHOOK_ENABLED=true"
echo "     export WEBHOOK_URL=https://webhook.site/your-unique-id"
echo ""

echo "Option C - Slack:"
echo "  1. Create a Slack Incoming Webhook: https://api.slack.com/messaging/webhooks"
echo "  2. export SLACK_ENABLED=true"
echo "     export SLACK_WEBHOOK_URL=https://hooks.slack.com/services/..."
echo "     export SLACK_CHANNEL=#deployments"
echo ""

echo "Option D - Microsoft Teams:"
echo "  1. Create an Incoming Webhook connector in Teams"
echo "  2. export TEAMS_ENABLED=true"
echo "     export TEAMS_WEBHOOK_URL=https://outlook.office.com/webhook/..."
echo ""

echo "📋 Step 2: Build and run Headwind"
echo "  cargo build --release"
echo "  RUST_LOG=headwind=debug ./target/release/headwind"
echo ""

echo "📋 Step 3: Trigger notifications"
echo ""
echo "A) Test UpdateDetected + UpdateRequestCreated:"
echo "   Send a webhook notification about a new image:"
echo "   curl -X POST http://localhost:8000/webhook \\"
echo "     -H 'Content-Type: application/json' \\"
echo "     -d '{"
echo "       \"name\": \"nginx\","
echo "       \"tag\": \"1.25.3\","
echo "       \"digest\": \"sha256:abc123...\","
echo "       \"repository\": \"nginx\""
echo "     }'"
echo ""

echo "B) Test UpdateApproved + UpdateCompleted/Failed:"
echo "   1. List pending updates:"
echo "      kubectl get updaterequests -A"
echo "   2. Approve an update:"
echo "      curl -X POST http://localhost:3030/api/v1/approvals/default/nginx-123/approve \\"
echo "        -H 'Content-Type: application/json' \\"
echo "        -d '{\"approver\": \"admin\"}'"
echo ""

echo "C) Test UpdateRejected:"
echo "   curl -X POST http://localhost:3030/api/v1/approvals/default/nginx-123/reject \\"
echo "     -H 'Content-Type: application/json' \\"
echo "     -d '{\"approver\": \"admin\", \"reason\": \"Not yet ready\"}'"
echo ""

echo "D) Test Rollback notifications:"
echo "   1. Enable auto-rollback in your deployment:"
echo "      kubectl annotate deployment nginx \\"
echo "        headwind.sh/auto-rollback-enabled=true \\"
echo "        headwind.sh/auto-rollback-health-threshold=50"
echo "   2. Deploy a broken image and watch auto-rollback trigger"
echo ""

echo "📋 Step 4: Check notification metrics"
echo "  curl http://localhost:9090/metrics | grep notification"
echo ""

echo "Expected metrics:"
echo "  - headwind_notifications_sent_total"
echo "  - headwind_notifications_failed_total"
echo "  - headwind_notifications_slack_sent_total"
echo "  - headwind_notifications_teams_sent_total"
echo "  - headwind_notifications_webhook_sent_total"
echo ""

echo "💡 Tips:"
echo "  - Check Headwind logs for notification send attempts"
echo "  - webhook.site shows payload and headers in real-time"
echo "  - Local webhook server verifies HMAC signatures"
echo "  - Use RUST_LOG=headwind=debug for detailed logging"
echo ""

echo "🎯 Quick test with local webhook server:"
echo ""
read -p "Start local webhook server now? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Starting webhook server on http://localhost:8080..."
    echo "Press Ctrl+C when done testing"
    python3 test-webhook-server.py
fi
