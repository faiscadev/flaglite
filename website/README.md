# FlagLite Website

Landing page for FlagLite — https://flaglite.dev

## Structure

```
website/
├── index.html      # Main landing page
├── css/
│   └── style.css   # Styles (dark theme)
├── js/
│   └── main.js     # Tab switching, copy code
├── llms.txt        # LLM-friendly docs
├── deploy.sh       # Manual deploy script
└── README.md
```

## Development

Just open `index.html` in a browser. No build step required.

```bash
# macOS
open website/index.html

# Linux
xdg-open website/index.html

# Or use a simple server
cd website && python3 -m http.server 8000
```

## Deployment

### Automatic (GitHub Actions)

Pushing to `main` with changes in `website/` triggers automatic deployment.

Required secrets:
- `AWS_ROLE_ARN` — IAM role with S3/CloudFront permissions
- `S3_BUCKET` — Target S3 bucket name
- `CLOUDFRONT_DISTRIBUTION_ID` — CloudFront distribution (optional)

### Manual

```bash
export S3_BUCKET="flaglite-website"
export CLOUDFRONT_DISTRIBUTION_ID="E123456789"
export AWS_REGION="us-east-1"

./deploy.sh
```

## Infrastructure

- **Hosting:** S3 static website + CloudFront CDN
- **Domain:** flaglite.dev (Route53)
- **SSL:** ACM certificate (CloudFront handles HTTPS)

### S3 Bucket Policy

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "CloudFrontAccess",
      "Effect": "Allow",
      "Principal": {
        "Service": "cloudfront.amazonaws.com"
      },
      "Action": "s3:GetObject",
      "Resource": "arn:aws:s3:::flaglite-website/*",
      "Condition": {
        "StringEquals": {
          "AWS:SourceArn": "arn:aws:cloudfront::ACCOUNT_ID:distribution/DISTRIBUTION_ID"
        }
      }
    }
  ]
}
```

## llms.txt

The `/llms.txt` endpoint serves LLM-friendly documentation for AI assistants. Content mirrors the API's `/llms.txt` handler in `api/src/handlers/llms.rs`.
