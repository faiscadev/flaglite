#!/bin/bash
set -e

# FlagLite Website Deploy Script
# Deploys static site to S3 and invalidates CloudFront cache

# Configuration (override via environment variables)
S3_BUCKET="${S3_BUCKET:-flaglite-website}"
CLOUDFRONT_DISTRIBUTION_ID="${CLOUDFRONT_DISTRIBUTION_ID:-}"
AWS_REGION="${AWS_REGION:-us-east-1}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚩 FlagLite Website Deployment${NC}"
echo "==============================="

# Check for AWS CLI
if ! command -v aws &> /dev/null; then
    echo -e "${RED}Error: AWS CLI is not installed${NC}"
    exit 1
fi

# Check AWS credentials
if ! aws sts get-caller-identity &> /dev/null; then
    echo -e "${RED}Error: AWS credentials not configured${NC}"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo -e "${YELLOW}→ Syncing to S3 bucket: s3://${S3_BUCKET}${NC}"

# Sync HTML files with no cache (for immediate updates)
aws s3 sync . "s3://${S3_BUCKET}" \
    --exclude "*.sh" \
    --exclude ".git/*" \
    --exclude "README.md" \
    --delete \
    --cache-control "max-age=0, no-cache, no-store, must-revalidate" \
    --region "$AWS_REGION"

# Set longer cache for static assets
aws s3 cp "s3://${S3_BUCKET}/css/" "s3://${S3_BUCKET}/css/" \
    --recursive \
    --cache-control "max-age=31536000, public" \
    --metadata-directive REPLACE \
    --region "$AWS_REGION" 2>/dev/null || true

aws s3 cp "s3://${S3_BUCKET}/js/" "s3://${S3_BUCKET}/js/" \
    --recursive \
    --cache-control "max-age=31536000, public" \
    --metadata-directive REPLACE \
    --region "$AWS_REGION" 2>/dev/null || true

# Set correct content types
aws s3 cp "s3://${S3_BUCKET}/llms.txt" "s3://${S3_BUCKET}/llms.txt" \
    --content-type "text/plain; charset=utf-8" \
    --cache-control "max-age=3600, public" \
    --metadata-directive REPLACE \
    --region "$AWS_REGION" 2>/dev/null || true

echo -e "${GREEN}✓ S3 sync complete${NC}"

# Invalidate CloudFront cache if distribution ID is provided
if [ -n "$CLOUDFRONT_DISTRIBUTION_ID" ]; then
    echo -e "${YELLOW}→ Invalidating CloudFront cache${NC}"
    aws cloudfront create-invalidation \
        --distribution-id "$CLOUDFRONT_DISTRIBUTION_ID" \
        --paths "/*" \
        --region "$AWS_REGION"
    echo -e "${GREEN}✓ CloudFront invalidation created${NC}"
else
    echo -e "${YELLOW}⚠ No CloudFront distribution ID set, skipping cache invalidation${NC}"
fi

echo ""
echo -e "${GREEN}✓ Deployment complete!${NC}"
echo "  Website: https://flaglite.dev"
