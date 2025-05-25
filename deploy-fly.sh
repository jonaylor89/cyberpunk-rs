#!/bin/bash

# Deploy to Fly.io with scale-to-zero configuration
# Even cheaper than Cloud Run for many use cases

set -e

echo "🪁 Deploying cyberpunk-rs to Fly.io"
echo ""

# Check if flyctl is installed
if ! command -v flyctl &> /dev/null; then
    echo "❌ flyctl not found. Install it:"
    echo "curl -L https://fly.io/install.sh | sh"
    exit 1
fi

# Launch the app (will create it if it doesn't exist)
echo "🚀 Launching app..."
flyctl launch --no-deploy --copy-config

# Create persistent volume for audio storage
echo "💾 Creating persistent volume..."
flyctl volumes create cyberpunk_data --region ord --size 1

# Deploy the app
echo "📦 Deploying to Fly.io..."
flyctl deploy

# Get the app URL
APP_URL=$(flyctl info --json | jq -r '.Hostname' | sed 's/^/https:\/\//')

echo ""
echo "✅ Deployment complete!"
echo "🌐 App URL: $APP_URL"
echo "💰 Cost: ~$1.94/month for 1GB storage + compute only when running"
echo ""
echo "Test your deployment:"
echo "curl $APP_URL/health"
echo ""
echo "Use with MCP:"
echo "npx @cyberpunk-rs/mcp-server --server=$APP_URL"
echo ""
echo "🎛️ Monitor and scale:"
echo "flyctl status"
echo "flyctl scale count 0  # Manual scale to zero"