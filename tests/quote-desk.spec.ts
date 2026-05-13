import { expect, test } from '@playwright/test'

test('generates a crypto drawdown quote and renders receipt args', async ({ page }) => {
  await page.goto('/')

  await page.getByRole('button', { name: /ETH drawdown protection/i }).click()
  await page.getByRole('button', { name: /Generate hedge quote/i }).click()

  await expect(page.getByText('CRYPTO_DRAWDOWN').first()).toBeVisible()
  await expect(page.locator('.market-table')).toContainText('ETH trades below key support')
  await expect(page.getByText('Quote receipt')).toBeVisible()
  await expect(page.locator('pre')).toContainText('quoteHash')
})
