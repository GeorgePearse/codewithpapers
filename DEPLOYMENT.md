# Deployment Guide

This guide explains how to deploy the CodeWithPapers website to GitHub Pages.

## Automated Deployment (Recommended)

The site is configured to automatically deploy to GitHub Pages whenever you push to the `main` branch.

### One-Time Setup

1. **Enable GitHub Pages in Repository Settings**
   - Go to your repository: https://github.com/GeorgePearse/codewithpapers
   - Navigate to Settings → Pages
   - Under "Build and deployment":
     - Source: Select "GitHub Actions"
   - Save the changes

2. **Push to Main Branch**
   ```bash
   git add .
   git commit -m "Set up GitHub Pages deployment"
   git push origin main
   ```

3. **Wait for Deployment**
   - Go to the "Actions" tab in your repository
   - Watch the "Deploy to GitHub Pages" workflow run
   - Once complete, your site will be live at: https://georgepearse.github.io/codewithpapers/

### Automatic Updates

After the initial setup, any push to `main` will automatically:
1. Build the site using Vite
2. Deploy to GitHub Pages
3. Update the live site

You can monitor deployments in the "Actions" tab.

## Manual Deployment

If you prefer to deploy manually using the command line:

```bash
# Install dependencies (first time only)
npm install

# Build and deploy
npm run deploy
```

This will:
1. Build the production site
2. Push the built files to the `gh-pages` branch
3. GitHub Pages will serve from that branch

**Note**: For manual deployment to work, you need to configure GitHub Pages to serve from the `gh-pages` branch in Settings → Pages.

## Local Testing

Before deploying, you can test the production build locally:

```bash
# Build the site
npm run build

# Preview the production build
npm run preview
```

The preview will be available at http://localhost:4173

## Configuration

The deployment is configured in:
- **vite.config.js**: Sets the base URL to `/codewithpapers/`
- **.github/workflows/deploy.yml**: GitHub Actions workflow
- **package.json**: Deploy scripts

## Troubleshooting

### Site not loading after deployment
- Check that the base URL in `vite.config.js` matches your repository name
- Verify GitHub Pages is enabled and set to use GitHub Actions
- Check the Actions tab for any build errors

### Assets not loading (404 errors)
- Ensure `base: '/codewithpapers/'` is set correctly in `vite.config.js`
- The base should match your repository name

### Build failures
- Check the Actions workflow logs for specific errors
- Verify all dependencies are in `package.json`
- Test the build locally with `npm run build`

## Live Site

Once deployed, your site will be available at:
**https://georgepearse.github.io/codewithpapers/**

## Custom Domain (Optional)

To use a custom domain:
1. Add a CNAME file to the `public/` folder with your domain
2. Configure DNS settings with your domain provider
3. Update GitHub Pages settings with your custom domain
4. Update the `base` in `vite.config.js` to `'/'`
