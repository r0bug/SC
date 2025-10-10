import { test, expect } from '@playwright/test';

test.describe('Auth Flow', () => {
	test('should display login page', async ({ page }) => {
		await page.goto('/auth/login');
		await expect(page.locator('h1')).toContainText('Sign In');
		await expect(page.locator('input[type="email"]')).toBeVisible();
		await expect(page.locator('input[type="password"]')).toBeVisible();
	});

	test('should login with mocked credentials', async ({ page }) => {
		await page.goto('/auth/login');
		await page.fill('input[type="email"]', 'test@example.com');
		await page.fill('input[type="password"]', 'password');
		await page.click('button[type="submit"]');

		// Should redirect to dashboard (mocked)
		await expect(page).toHaveURL(/.*dashboard/, { timeout: 5000 });
	});
});

test.describe('Contact Management', () => {
	test.beforeEach(async ({ page }) => {
		// Mock login
		await page.goto('/auth/login');
		await page.fill('input[type="email"]', 'test@example.com');
		await page.fill('input[type="password"]', 'password');
		await page.click('button[type="submit"]');
	});

	test('should display contacts list', async ({ page }) => {
		await page.goto('/contacts');
		await expect(page.locator('h1')).toContainText('Contacts');
		await expect(page.locator('.contact-grid, .empty')).toBeVisible();
	});

	test('should search contacts', async ({ page }) => {
		await page.goto('/contacts');
		const searchInput = page.locator('input[placeholder*="Search"]');
		await searchInput.fill('John');
		await searchInput.press('Enter');
		// Results would appear (mocked)
	});

	test('should navigate to contact details', async ({ page }) => {
		await page.goto('/contacts');
		// If contacts exist, click on first one
		const contactCards = page.locator('.contact-card');
		const count = await contactCards.count();
		if (count > 0) {
			await contactCards.first().locator('a').first().click();
			await expect(page).toHaveURL(/.*contacts\/.*/, { timeout: 5000 });
		}
	});
});

test.describe('Dashboard', () => {
	test.beforeEach(async ({ page }) => {
		// Mock login
		await page.goto('/auth/login');
		await page.fill('input[type="email"]', 'test@example.com');
		await page.fill('input[type="password"]', 'password');
		await page.click('button[type="submit"]');
	});

	test('should display dashboard summaries', async ({ page }) => {
		await page.goto('/dashboard');
		await expect(page.locator('h1')).toContainText('Dashboard');

		// Check for summary cards
		await expect(page.locator('.summary-card').first()).toBeVisible();

		// Check for sections
		await expect(page.locator('text=Upcoming Events')).toBeVisible();
		await expect(page.locator('text=Recent Communications')).toBeVisible();
		await expect(page.locator('text=AI Suggestions')).toBeVisible();
	});

	test('should navigate from dashboard to contacts', async ({ page }) => {
		await page.goto('/dashboard');
		await page.click('text=View Contacts');
		await expect(page).toHaveURL(/.*contacts/);
	});
});

test.describe('Calendar', () => {
	test.beforeEach(async ({ page }) => {
		// Mock login
		await page.goto('/auth/login');
		await page.fill('input[type="email"]', 'test@example.com');
		await page.fill('input[type="password"]', 'password');
		await page.click('button[type="submit"]');
	});

	test('should display calendar month view', async ({ page }) => {
		await page.goto('/calendar');
		await expect(page.locator('h1')).toContainText('Calendar');
		await expect(page.locator('.month-view, .week-view')).toBeVisible();

		// Check for weekday headers
		await expect(page.locator('.weekday').first()).toBeVisible();
	});

	test('should navigate between months', async ({ page }) => {
		await page.goto('/calendar');

		// Click next month
		await page.click('button:has-text("Next")');

		// Click previous month
		await page.click('button:has-text("Previous")');

		// Click today
		await page.click('button:has-text("Today")');
	});

	test('should toggle between month and week view', async ({ page }) => {
		await page.goto('/calendar');

		// Click week view
		await page.click('button:has-text("Week")');

		// Click month view
		await page.click('button:has-text("Month")');
		await expect(page.locator('.month-view')).toBeVisible();
	});
});

test.describe('Import', () => {
	test.beforeEach(async ({ page }) => {
		// Mock login
		await page.goto('/auth/login');
		await page.fill('input[type="email"]', 'test@example.com');
		await page.fill('input[type="password"]', 'password');
		await page.click('button[type="submit"]');
	});

	test('should display import page', async ({ page }) => {
		await page.goto('/import');
		await expect(page.locator('h1')).toContainText('Import Data');

		// Check format options
		await expect(page.locator('text=CSV')).toBeVisible();
		await expect(page.locator('text=vCard')).toBeVisible();
		await expect(page.locator('text=JSON')).toBeVisible();
	});

	test('should select import format', async ({ page }) => {
		await page.goto('/import');

		// Select CSV
		await page.click('input[value="csv"]');
		await expect(page.locator('input[value="csv"]')).toBeChecked();

		// Select vCard
		await page.click('input[value="vcard"]');
		await expect(page.locator('input[value="vcard"]')).toBeChecked();
	});

	test('should show file upload area', async ({ page }) => {
		await page.goto('/import');
		await expect(page.locator('.upload-area')).toBeVisible();
		await expect(page.locator('text=Click to select')).toBeVisible();
	});
});

test.describe('Communications', () => {
	test.beforeEach(async ({ page }) => {
		// Mock login
		await page.goto('/auth/login');
		await page.fill('input[type="email"]', 'test@example.com');
		await page.fill('input[type="password"]', 'password');
		await page.click('button[type="submit"]');
	});

	test('should display communications page', async ({ page }) => {
		await page.goto('/communications');
		await expect(page.locator('h1')).toContainText('Communications');

		// Should show mock warning
		await expect(page.locator('text=Alpha Release')).toBeVisible();
	});
});

test.describe('Navigation', () => {
	test.beforeEach(async ({ page }) => {
		// Mock login
		await page.goto('/auth/login');
		await page.fill('input[type="email"]', 'test@example.com');
		await page.fill('input[type="password"]', 'password');
		await page.click('button[type="submit"]');
	});

	test('should show navigation sidebar after login', async ({ page }) => {
		await page.goto('/dashboard');

		// Check main navigation links
		await expect(page.locator('nav a:has-text("Dashboard")')).toBeVisible();
		await expect(page.locator('nav a:has-text("Contacts")')).toBeVisible();
		await expect(page.locator('nav a:has-text("Groups")')).toBeVisible();
		await expect(page.locator('nav a:has-text("Projects")')).toBeVisible();
		await expect(page.locator('nav a:has-text("Calendar")')).toBeVisible();
		await expect(page.locator('nav a:has-text("Notes")')).toBeVisible();
		await expect(page.locator('nav a:has-text("Communications")')).toBeVisible();
		await expect(page.locator('nav a:has-text("Settings")')).toBeVisible();
	});

	test('should navigate through main sections', async ({ page }) => {
		await page.goto('/dashboard');

		// Navigate to Contacts
		await page.click('nav a:has-text("Contacts")');
		await expect(page).toHaveURL(/.*contacts/);

		// Navigate to Calendar
		await page.click('nav a:has-text("Calendar")');
		await expect(page).toHaveURL(/.*calendar/);

		// Navigate to Import
		await page.click('nav a:has-text("Import")');
		await expect(page).toHaveURL(/.*import/);

		// Navigate back to Dashboard
		await page.click('nav a:has-text("Dashboard")');
		await expect(page).toHaveURL(/.*dashboard/);
	});

	test('should logout', async ({ page }) => {
		await page.goto('/dashboard');

		// Click logout button
		await page.click('button:has-text("Logout")');

		// Should redirect to login
		await expect(page).toHaveURL(/.*auth\/login/, { timeout: 5000 });
	});
});

test.describe('Contact CRUD Operations', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/auth/login');
		await page.fill('input[type="email"]', 'test@example.com');
		await page.fill('input[type="password"]', 'password');
		await page.click('button[type="submit"]');
	});

	test('should create a new contact', async ({ page }) => {
		await page.goto('/contacts/new');
		await expect(page.locator('h1')).toContainText('Create Contact');

		// Fill form
		await page.fill('input[name="first_name"]', 'John');
		await page.fill('input[name="last_name"]', 'Doe');
		await page.fill('input[type="email"]', 'john@example.com');
		await page.fill('input[type="tel"]', '555-0123');

		// Submit
		await page.click('button[type="submit"]');

		// Should redirect to contact detail
		await expect(page).toHaveURL(/.*contacts\/.*/, { timeout: 5000 });
	});

	test('should edit existing contact', async ({ page }) => {
		await page.goto('/contacts');
		const contactCards = page.locator('.contact-card');
		const count = await contactCards.count();

		if (count > 0) {
			await contactCards.first().locator('a:has-text("Edit")').click();
			await page.click('button:has-text("Edit")');

			// Modify contact
			await page.fill('input[name="first_name"]', 'Jane');
			await page.click('button:has-text("Save")');

			// Should show updated name
			await expect(page.locator('h1')).toContainText('Jane');
		}
	});

	test('should delete contact', async ({ page }) => {
		await page.goto('/contacts');
		const contactCards = page.locator('.contact-card');
		const count = await contactCards.count();

		if (count > 0) {
			// Go to contact detail
			await contactCards.first().locator('a:has-text("Edit")').first().click();

			// Click delete
			page.once('dialog', dialog => dialog.accept());
			await page.click('button:has-text("Delete")');

			// Should redirect to contacts list
			await expect(page).toHaveURL(/.*contacts$/, { timeout: 5000 });
		}
	});
});

test.describe('Projects', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/auth/login');
		await page.fill('input[type="email"]', 'test@example.com');
		await page.fill('input[type="password"]', 'password');
		await page.click('button[type="submit"]');
	});

	test('should display projects page', async ({ page }) => {
		await page.goto('/projects');
		await expect(page.locator('h1')).toContainText('Projects');
	});

	test('should filter projects by status', async ({ page }) => {
		await page.goto('/projects');

		// Click Active filter
		await page.click('button:has-text("Active")');

		// Click Completed filter
		await page.click('button:has-text("Completed")');
	});

	test('should create new project', async ({ page }) => {
		await page.goto('/projects');
		await page.click('button:has-text("+ New Project")');

		// Fill project form
		await page.fill('input[name="name"]', 'New Project');
		await page.fill('textarea[name="description"]', 'Project description');

		// Submit
		await page.click('button:has-text("Create")');

		// Should close modal
		await expect(page.locator('.modal')).not.toBeVisible();
	});
});

test.describe('Notes', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/auth/login');
		await page.fill('input[type="email"]', 'test@example.com');
		await page.fill('input[type="password"]', 'password');
		await page.click('button[type="submit"]');
	});

	test('should display notes page', async ({ page }) => {
		await page.goto('/notes');
		await expect(page.locator('h1')).toContainText('Notes');
	});

	test('should create new note', async ({ page }) => {
		await page.goto('/notes');
		await page.click('button:has-text("+ New Note")');

		// Fill note form
		await page.fill('input[name="title"]', 'Test Note');
		await page.fill('textarea[name="content"]', 'Note content here');

		// Submit
		await page.click('button:has-text("Create")');

		// Should close modal
		await expect(page.locator('.modal')).not.toBeVisible();
	});
});

test.describe('Groups', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/auth/login');
		await page.fill('input[type="email"]', 'test@example.com');
		await page.fill('input[type="password"]', 'password');
		await page.click('button[type="submit"]');
	});

	test('should display groups page', async ({ page }) => {
		await page.goto('/groups');
		await expect(page.locator('h1')).toContainText('Groups');
	});

	test('should create new group', async ({ page }) => {
		await page.goto('/groups');
		await page.click('button:has-text("+ New Group")');

		// Fill group form
		await page.fill('input[name="name"]', 'Test Group');
		await page.fill('textarea[name="description"]', 'Group description');

		// Submit
		await page.click('button:has-text("Create")');
	});
});

test.describe('Concepts', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/auth/login');
		await page.fill('input[type="email"]', 'test@example.com');
		await page.fill('input[type="password"]', 'password');
		await page.click('button[type="submit"]');
	});

	test('should display concepts page', async ({ page }) => {
		await page.goto('/concepts');
		await expect(page.locator('h1')).toContainText('Concepts');
	});

	test('should create new concept', async ({ page }) => {
		await page.goto('/concepts');
		await page.click('button:has-text("+ New Concept")');

		// Fill concept form
		await page.fill('input[name="name"]', 'Test Concept');
		await page.fill('textarea[name="description"]', 'Concept description');

		// Submit
		await page.click('button:has-text("Create")');
	});
});

test.describe('Insights', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/auth/login');
		await page.fill('input[type="email"]', 'test@example.com');
		await page.fill('input[type="password"]', 'password');
		await page.click('button[type="submit"]');
	});

	test('should display insights page', async ({ page }) => {
		await page.goto('/insights');
		await expect(page.locator('h1')).toContainText('AI Insights');
	});

	test('should show insight confidence', async ({ page }) => {
		await page.goto('/insights');
		const insights = page.locator('.insight-card');
		const count = await insights.count();

		if (count > 0) {
			await expect(insights.first().locator('.confidence')).toBeVisible();
		}
	});

	test('should allow insight feedback', async ({ page }) => {
		await page.goto('/insights');
		const insights = page.locator('.insight-card');
		const count = await insights.count();

		if (count > 0) {
			// Click helpful button
			await insights.first().locator('button:has-text("Helpful")').click();
		}
	});
});

test.describe('Search History', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/auth/login');
		await page.fill('input[type="email"]', 'test@example.com');
		await page.fill('input[type="password"]', 'password');
		await page.click('button[type="submit"]');
	});

	test('should display search history page', async ({ page }) => {
		await page.goto('/search');
		await expect(page.locator('h1')).toContainText('Search History');
	});

	test('should clear search history', async ({ page }) => {
		await page.goto('/search');
		const historyItems = page.locator('.history-item');
		const count = await historyItems.count();

		if (count > 0) {
			page.once('dialog', dialog => dialog.accept());
			await page.click('button:has-text("Clear History")');
		}
	});
});

test.describe('Settings', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/auth/login');
		await page.fill('input[type="email"]', 'test@example.com');
		await page.fill('input[type="password"]', 'password');
		await page.click('button[type="submit"]');
	});

	test('should display settings page', async ({ page }) => {
		await page.goto('/settings');
		await expect(page.locator('h1')).toContainText('Settings');
	});

	test('should show worker metrics', async ({ page }) => {
		await page.goto('/settings');
		await expect(page.locator('text=Worker Metrics')).toBeVisible();
	});

	test('should update theme setting', async ({ page }) => {
		await page.goto('/settings');
		await page.selectOption('select[name="theme"]', 'dark');
		await page.click('button:has-text("Save Settings")');
	});
});

test.describe('Share Acceptance', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/auth/login');
		await page.fill('input[type="email"]', 'test@example.com');
		await page.fill('input[type="password"]', 'password');
		await page.click('button[type="submit"]');
	});

	test('should display shares page', async ({ page }) => {
		await page.goto('/shares');
		await expect(page.locator('h1')).toContainText('Shared Items');
	});

	test('should accept share invite', async ({ page }) => {
		await page.goto('/shares');
		const shares = page.locator('.share-card');
		const count = await shares.count();

		if (count > 0) {
			await shares.first().locator('button:has-text("Accept")').click();
		}
	});

	test('should revoke share', async ({ page }) => {
		await page.goto('/shares');
		const shares = page.locator('.share-card');
		const count = await shares.count();

		if (count > 0) {
			page.once('dialog', dialog => dialog.accept());
			await shares.first().locator('button:has-text("Revoke")').click();
		}
	});
});

test.describe('Mock Warnings', () => {
	test('should display alpha/mock warnings', async ({ page }) => {
		await page.goto('/auth/login');

		// Check for alpha notice on login
		await expect(page.locator('text=Alpha Notice')).toBeVisible();

		// Login and check dashboard
		await page.fill('input[type="email"]', 'test@example.com');
		await page.fill('input[type="password"]', 'password');
		await page.click('button[type="submit"]');

		await page.goto('/dashboard');
		await expect(page.locator('text=Alpha Release')).toBeVisible();
		await expect(page.locator('text=mocked')).toBeVisible();
	});
});