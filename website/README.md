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
├── CNAME           # Custom domain for GitHub Pages
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

Pushing to `main` with changes in `website/` triggers automatic deployment via GitHub Actions.

Workflow: `.github/workflows/static.yml`

**No manual deployment needed** — GitHub Pages handles everything.

## Infrastructure

- **Hosting:** GitHub Pages
- **Domain:** flaglite.dev (Cloudflare DNS → GitHub Pages)
- **SSL:** GitHub Pages (automatic via Let's Encrypt)

## llms.txt

The `/llms.txt` endpoint serves LLM-friendly documentation for AI assistants. Content mirrors the API's `/llms.txt` handler in `api/src/handlers/llms.rs`.
