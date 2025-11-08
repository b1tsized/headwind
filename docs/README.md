# Headwind Documentation Site

This directory contains the Docusaurus-based documentation site for Headwind.

## Local Development

```bash
cd docs
npm install
npm start
```

This starts a local development server at http://localhost:3000 with hot reload.

## Build

```bash
npm run build
```

This generates static content into the `build` directory.

## Deployment

The site is automatically deployed to GitHub Pages via GitHub Actions when changes are pushed to the `main` branch.

### Custom Domain Setup

The site is configured to use the custom domain `headwind.sh`. To complete the setup:

1. **DNS Configuration** - Add these DNS records to your domain:
   ```
   Type: A
   Name: @
   Value: 185.199.108.153

   Type: A
   Name: @
   Value: 185.199.109.153

   Type: A
   Name: @
   Value: 185.199.110.153

   Type: A
   Name: @
   Value: 185.199.111.153

   Type: CNAME
   Name: www
   Value: headwind.sh.github.io
   ```

2. **GitHub Pages Settings** - In the repository settings:
   - Go to Settings → Pages
   - Set Source to "GitHub Actions"
   - Add custom domain: `headwind.sh`
   - Enable "Enforce HTTPS" (after DNS propagates)

3. **Verify** - After DNS propagates (can take up to 24 hours):
   - Visit https://headwind.sh
   - Verify SSL certificate is valid
   - Check that www.headwind.sh redirects properly

## Documentation Structure

- `docs/docs/` - Main documentation markdown files
- `docs/src/` - Custom React components and pages
- `docs/static/` - Static assets (images, CNAME, etc.)
- `docusaurus.config.ts` - Site configuration
- `sidebars.ts` - Sidebar navigation structure
