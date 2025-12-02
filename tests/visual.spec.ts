import { test, expect } from '@playwright/test';

test('visual check of homepage and sota page', async ({ page }) => {
  // Go to homepage
  await page.goto('/');

  // Wait for loading to finish (either papers list appears or empty state or error)
  // We explicitly want to verify success for a "functional" check, so we expect papers-list
  await page.waitForSelector('.papers-list', { timeout: 10000 });

  // Take screenshot of homepage
  await page.screenshot({ path: 'tests/homepage.png', fullPage: true });

  // Click on "Browse State-of-the-Art"
  await page.getByText('Browse State-of-the-Art').click();

  // Wait for SOTA view
  await page.waitForSelector('.sota-view');

  // Take screenshot of SOTA page
  await page.screenshot({ path: 'tests/sota.png', fullPage: true });
});
