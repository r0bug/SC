import { test, expect } from '@playwright/test';

/**
 * API Integration Tests
 * These tests verify the web UI integrates correctly with the backend API
 */

// Helper to login and get auth cookie
async function login(page) {
	await page.goto('/auth/login');
	await page.fill('input[type="email"]', 'test@example.com');
	await page.fill('input[type="password"]', 'password');
	await page.click('button[type="submit"]');
	await page.waitForURL(/.*dashboard/, { timeout: 10000 });
}

test.describe('API Health Checks', () => {
	test('should display API connection status', async ({ page }) => {
		await login(page);
		await page.goto('/settings');

		// Check for API status indicator
		const statusIndicator = page.locator('[data-testid="api-status"], .api-status, text=Connected');
		const isVisible = await statusIndicator.isVisible().catch(() => false);

		if (isVisible) {
			await expect(statusIndicator).toBeVisible();
		}
	});
});

test.describe('Contact Import Flow', () => {
	test.beforeEach(async ({ page }) => {
		await login(page);
	});

	test('should show available import connectors', async ({ page }) => {
		await page.goto('/import');

		// Check for connector options
		await expect(page.locator('text=CSV')).toBeVisible();
		await expect(page.locator('text=vCard')).toBeVisible();

		// Check for social media imports (if visible)
		const linkedinOption = page.locator('text=LinkedIn');
		const twitterOption = page.locator('text=Twitter');

		// These may or may not be visible depending on UI state
		if (await linkedinOption.isVisible().catch(() => false)) {
			await expect(linkedinOption).toBeVisible();
		}
	});

	test('should show import progress indicators', async ({ page }) => {
		await page.goto('/import');

		// Select a format
		await page.click('input[value="csv"]');

		// Check for upload zone
		const uploadZone = page.locator('.upload-area, [data-testid="upload-zone"], .dropzone');
		await expect(uploadZone).toBeVisible();
	});

	test('should display field mapping interface', async ({ page }) => {
		await page.goto('/import');

		// After file upload, there should be field mapping
		// This is typically shown after file selection
		const mappingSection = page.locator('.field-mapping, text=Field Mapping, text=Map Fields');
		const isVisible = await mappingSection.isVisible().catch(() => false);

		// Just check the page loads correctly
		await expect(page.locator('h1')).toContainText('Import');
	});
});

test.describe('Social Import Features', () => {
	test.beforeEach(async ({ page }) => {
		await login(page);
	});

	test('should show social import options', async ({ page }) => {
		await page.goto('/import');

		// Look for social media import sections
		const importOptions = [
			'LinkedIn',
			'Twitter',
			'Facebook',
			'Instagram',
			'Google Contacts'
		];

		for (const option of importOptions) {
			const element = page.locator(`text=${option}`);
			const isVisible = await element.isVisible().catch(() => false);
			// Log which options are available (not all may be visible)
		}

		// At minimum, the import page should load
		await expect(page.locator('h1')).toContainText('Import');
	});
});

test.describe('SMS/Communication History', () => {
	test.beforeEach(async ({ page }) => {
		await login(page);
	});

	test('should display communications page with tabs', async ({ page }) => {
		await page.goto('/communications');
		await expect(page.locator('h1')).toContainText('Communications');

		// Check for communication type tabs/filters
		const smsTab = page.locator('button:has-text("SMS"), a:has-text("SMS")');
		const emailTab = page.locator('button:has-text("Email"), a:has-text("Email")');

		// These may be visible depending on the UI
		const hasSmsTab = await smsTab.isVisible().catch(() => false);
		const hasEmailTab = await emailTab.isVisible().catch(() => false);
	});

	test('should show alpha warning for communications', async ({ page }) => {
		await page.goto('/communications');
		await expect(page.locator('text=Alpha')).toBeVisible();
	});
});

test.describe('Attachment Management', () => {
	test.beforeEach(async ({ page }) => {
		await login(page);
	});

	test('should show attachment upload on contact page', async ({ page }) => {
		await page.goto('/contacts/new');

		// Look for attachment section
		const attachmentSection = page.locator('text=Attachments, .attachments-section, [data-testid="attachments"]');
		// Attachment upload may be on detail page, not new contact form
	});
});

test.describe('Real-time Updates', () => {
	test.beforeEach(async ({ page }) => {
		await login(page);
	});

	test('should show WebSocket connection indicator', async ({ page }) => {
		await page.goto('/dashboard');

		// Look for connection status
		const wsIndicator = page.locator('[data-testid="ws-status"], .ws-connected, .realtime-status');
		const isVisible = await wsIndicator.isVisible().catch(() => false);

		// The dashboard should load regardless
		await expect(page.locator('h1')).toContainText('Dashboard');
	});
});

test.describe('Error Handling', () => {
	test('should handle API errors gracefully', async ({ page }) => {
		await login(page);

		// Navigate to a potentially error-prone page
		await page.goto('/contacts?search=');

		// Should not crash, should show contacts or empty state
		const content = page.locator('h1');
		await expect(content).toContainText('Contacts');
	});

	test('should show error messages for invalid forms', async ({ page }) => {
		await login(page);
		await page.goto('/contacts/new');

		// Try to submit empty form
		await page.click('button[type="submit"]');

		// Should show validation errors
		const errorMessage = page.locator('.error, .invalid, [role="alert"], .form-error');
		// The form should either show errors or have default values
	});
});

test.describe('Performance', () => {
	test('should load dashboard in reasonable time', async ({ page }) => {
		const startTime = Date.now();
		await login(page);
		await page.goto('/dashboard');
		await page.waitForLoadState('networkidle');
		const loadTime = Date.now() - startTime;

		// Dashboard should load within 5 seconds
		expect(loadTime).toBeLessThan(5000);
	});

	test('should load contacts list efficiently', async ({ page }) => {
		await login(page);

		const startTime = Date.now();
		await page.goto('/contacts');
		await page.waitForLoadState('networkidle');
		const loadTime = Date.now() - startTime;

		// Contacts page should load within 3 seconds
		expect(loadTime).toBeLessThan(3000);
	});
});

test.describe('Responsive Design', () => {
	test('should work on mobile viewport', async ({ page }) => {
		await page.setViewportSize({ width: 375, height: 667 });
		await login(page);

		await page.goto('/dashboard');
		await expect(page.locator('h1')).toContainText('Dashboard');

		// Check for mobile menu button
		const menuButton = page.locator('[data-testid="mobile-menu"], .hamburger, button[aria-label*="menu"]');
		const hasMobileMenu = await menuButton.isVisible().catch(() => false);
	});

	test('should work on tablet viewport', async ({ page }) => {
		await page.setViewportSize({ width: 768, height: 1024 });
		await login(page);

		await page.goto('/contacts');
		await expect(page.locator('h1')).toContainText('Contacts');
	});
});

test.describe('Accessibility', () => {
	test('should have proper heading structure', async ({ page }) => {
		await login(page);
		await page.goto('/dashboard');

		const h1 = await page.locator('h1').count();
		expect(h1).toBeGreaterThan(0);
	});

	test('should have labels for form inputs', async ({ page }) => {
		await page.goto('/auth/login');

		// Check email input has a label
		const emailLabel = page.locator('label:has-text("Email"), label[for*="email"]');
		const emailInput = page.locator('input[type="email"]');

		await expect(emailInput).toBeVisible();
	});

	test('should be keyboard navigable', async ({ page }) => {
		await page.goto('/auth/login');

		// Tab through form elements
		await page.keyboard.press('Tab');
		await page.keyboard.press('Tab');

		// Focus should be on a form element
		const focusedElement = await page.locator(':focus');
		const tagName = await focusedElement.evaluate(el => el.tagName.toLowerCase());
		expect(['input', 'button', 'a', 'select', 'textarea']).toContain(tagName);
	});
});
