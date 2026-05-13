import { expect, test } from '@playwright/test'

test('generates a crypto drawdown quote and renders receipt args', async ({ page }) => {
  await page.goto('/')

  await page.getByRole('button', { name: /ETH drawdown protection/i }).click()
  await page.getByRole('button', { name: /Generate quote/i }).click()

  await expect(page.getByText('CRYPTO_DRAWDOWN').first()).toBeVisible()
  await expect(page.locator('.market-table')).toContainText('ETH trades below key support')
  await expect(page.getByRole('heading', { name: 'On-chain receipt' })).toBeVisible()
  await expect(page.locator('pre')).toContainText('quoteHash')

  await page.getByRole('button', { name: 'Positions' }).click()
  await expect(page.getByRole('heading', { name: 'Exposure inventory' })).toBeVisible()

  await page.getByRole('button', { name: 'Receipts' }).click()
  await expect(page.getByRole('heading', { name: 'Quote receipt registry' })).toBeVisible()

  await page.getByRole('button', { name: 'Markets' }).click()
  await expect(page.getByRole('heading', { name: 'Prediction-market watchlist' })).toBeVisible()

  await page.getByRole('button', { name: 'Alerts' }).click()
  await expect(page.getByRole('heading', { name: 'Risk monitoring rules' })).toBeVisible()

  await page.getByRole('button', { name: 'Settings' }).click()
  await expect(page.getByRole('heading', { name: 'Quote desk controls' })).toBeVisible()
})
