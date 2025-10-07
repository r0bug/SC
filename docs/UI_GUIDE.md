# SagensContact UI Design Guide

Version: 1.0.0
Last Updated: 2025-10-06

## Overview

This guide documents the design system, component library, and UI patterns for SagensContact Alpha. The system is built to be accessible, responsive, and easy to extend.

## Design Philosophy

1. **Accessibility First**: WCAG 2.1 AA compliance, keyboard navigation, screen reader support
2. **Progressive Enhancement**: Works without JavaScript, enhanced with it
3. **Mobile-Responsive**: Desktop-first with tablet/mobile breakpoints
4. **Performance**: Minimal CSS, optimized assets, lazy loading
5. **Consistency**: Reusable components, standardized patterns

## Design Tokens

All design tokens are defined in `apps/web/src/lib/styles/design-tokens.css`.

### Color Palette

#### Primary Colors (Indigo)
```css
--color-primary-50:  #eef2ff
--color-primary-100: #e0e7ff
--color-primary-600: #4f46e5  /* Main brand color */
--color-primary-700: #4338ca  /* Hover states */
```

#### Neutral Colors (Gray)
```css
--color-neutral-0:   #ffffff
--color-neutral-50:  #f9fafb  /* Background */
--color-neutral-100: #f3f4f6  /* Subtle backgrounds */
--color-neutral-200: #e5e7eb  /* Borders */
--color-neutral-600: #4b5563  /* Secondary text */
--color-neutral-900: #111827  /* Primary text */
```

#### Semantic Colors
```css
--color-success-500: #10b981  /* Success states */
--color-warning-500: #f59e0b  /* Warning states */
--color-error-500:   #ef4444  /* Error states */
--color-info-500:    #3b82f6  /* Informational */
```

### Typography

#### Font Families
```css
--font-sans: system-ui, -apple-system, ...
--font-mono: 'SF Mono', Monaco, ...
```

#### Font Sizes
```css
--text-xs:   0.75rem   /* 12px */
--text-sm:   0.875rem  /* 14px - body, labels */
--text-base: 1rem      /* 16px - body */
--text-lg:   1.125rem  /* 18px - subheadings */
--text-2xl:  1.5rem    /* 24px - headings */
--text-3xl:  1.875rem  /* 30px - page titles */
```

#### Font Weights
```css
--font-normal:   400
--font-medium:   500  /* Labels, buttons */
--font-semibold: 600  /* Subheadings */
--font-bold:     700  /* Headings */
```

### Spacing Scale

Based on 4px grid:
```css
--space-1:  0.25rem  /*  4px */
--space-2:  0.5rem   /*  8px - tight spacing */
--space-3:  0.75rem  /* 12px */
--space-4:  1rem     /* 16px - standard spacing */
--space-6:  1.5rem   /* 24px - section spacing */
--space-8:  2rem     /* 32px - large gaps */
--space-12: 3rem     /* 48px - page sections */
```

### Border Radius
```css
--radius-sm:   0.25rem  /* 4px - tight corners */
--radius-md:   0.375rem /* 6px - buttons, inputs */
--radius-lg:   0.5rem   /* 8px - cards */
--radius-xl:   0.75rem  /* 12px */
--radius-full: 9999px   /* Pills, avatars */
```

### Shadows
```css
--shadow-sm: /* Subtle elevation */
--shadow-md: /* Cards, dropdowns */
--shadow-lg: /* Modals, popovers */
```

### Transitions
```css
--transition-fast: 150ms cubic-bezier(0.4, 0, 0.2, 1)
--transition-base: 200ms cubic-bezier(0.4, 0, 0.2, 1)
--transition-slow: 300ms cubic-bezier(0.4, 0, 0.2, 1)
```

## Component Library

All reusable component styles are in `apps/web/src/lib/styles/components.css`.

### Buttons

#### Variants
```html
<button class="btn btn-primary">Primary Action</button>
<button class="btn btn-secondary">Secondary</button>
<button class="btn btn-outline">Outline</button>
<button class="btn btn-ghost">Ghost</button>
<button class="btn btn-danger">Delete</button>
<button class="btn btn-success">Confirm</button>
```

#### Sizes
```html
<button class="btn btn-sm">Small</button>
<button class="btn">Default</button>
<button class="btn btn-lg">Large</button>
```

#### Modifiers
```html
<button class="btn btn-block">Full Width</button>
<button class="btn btn-icon">🔍</button>
<button class="btn" disabled>Disabled</button>
```

### Cards

```html
<div class="card">
  <div class="card-header">
    <h3 class="card-title">Card Title</h3>
  </div>
  <div class="card-body">
    Content goes here
  </div>
  <div class="card-footer">
    <button class="btn btn-primary">Action</button>
  </div>
</div>
```

#### Variants
```html
<div class="card card-compact">Tighter padding</div>
<div class="card card-interactive">Clickable card</div>
```

### Form Elements

#### Basic Form
```html
<div class="form-group">
  <label class="label" for="email">Email Address</label>
  <input
    type="email"
    id="email"
    class="input"
    placeholder="you@example.com"
  />
  <span class="input-hint">We'll never share your email</span>
</div>
```

#### Required Fields
```html
<label class="label label-required" for="name">Full Name</label>
```

#### Error States
```html
<input type="text" class="input input-error" />
<span class="input-error-message">This field is required</span>
```

#### Textarea
```html
<textarea class="textarea" rows="4"></textarea>
```

#### Select
```html
<select class="select">
  <option>Option 1</option>
  <option>Option 2</option>
</select>
```

### Badges

```html
<span class="badge badge-primary">New</span>
<span class="badge badge-success">Active</span>
<span class="badge badge-warning">Pending</span>
<span class="badge badge-error">Failed</span>
<span class="badge badge-neutral">Draft</span>
```

### Tables

```html
<div class="table-wrapper">
  <table class="table">
    <thead>
      <tr>
        <th>Name</th>
        <th>Email</th>
        <th>Status</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>John Doe</td>
        <td>john@example.com</td>
        <td><span class="badge badge-success">Active</span></td>
      </tr>
    </tbody>
  </table>
</div>
```

### Loading States

#### Skeleton Loaders
```html
<div class="skeleton skeleton-title"></div>
<div class="skeleton skeleton-text"></div>
<div class="skeleton skeleton-text"></div>
```

#### Spinners
```html
<div class="spinner"></div>
<div class="spinner spinner-sm"></div>
<div class="spinner spinner-lg"></div>
```

### Alerts

```html
<div class="alert alert-info">
  ℹ️ This is an informational message
</div>
<div class="alert alert-success">
  ✅ Operation completed successfully
</div>
<div class="alert alert-warning">
  ⚠️ Please review this warning
</div>
<div class="alert alert-error">
  ❌ An error occurred
</div>
```

### Empty States

```html
<div class="empty-state">
  <div class="empty-state-icon">📭</div>
  <h3 class="empty-state-title">No items found</h3>
  <p class="empty-state-description">
    Get started by creating your first item
  </p>
  <button class="btn btn-primary">Create Item</button>
</div>
```

## Svelte Components

### PageHeader

Standardized page header with breadcrumbs and actions.

```svelte
<script>
  import PageHeader from '$lib/components/ui/PageHeader.svelte';
</script>

<PageHeader
  title="Contacts"
  description="Manage your contact relationships"
  breadcrumbs={[
    { label: 'Dashboard', href: '/dashboard' },
    { label: 'Contacts' }
  ]}
>
  <svelte:fragment slot="actions">
    <button class="btn btn-primary">Add Contact</button>
  </svelte:fragment>
</PageHeader>
```

### Breadcrumb

```svelte
<script>
  import Breadcrumb from '$lib/components/ui/Breadcrumb.svelte';
</script>

<Breadcrumb
  items={[
    { label: 'Home', href: '/' },
    { label: 'Settings', href: '/settings' },
    { label: 'Profile' }
  ]}
/>
```

### Toast

Global toast notifications (already in layout):

```typescript
import { toast } from '$lib/stores/toast';

toast.success('Item created successfully');
toast.error('Failed to save changes');
toast.info('New update available');
```

### LoadingSpinner

```svelte
<script>
  import LoadingSpinner from '$lib/components/ui/LoadingSpinner.svelte';
</script>

<LoadingSpinner />
```

## Layout Patterns

### Page Structure

```svelte
<script>
  import PageHeader from '$lib/components/ui/PageHeader.svelte';
</script>

<div class="container" style="padding: var(--space-6);">
  <PageHeader
    title="Page Title"
    description="Page description"
  />

  <div class="card">
    <!-- Main content -->
  </div>
</div>
```

### Grid Layout

```html
<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: var(--space-4);">
  <div class="card">Card 1</div>
  <div class="card">Card 2</div>
  <div class="card">Card 3</div>
</div>
```

### Flex Layout

```html
<div style="display: flex; gap: var(--space-4); align-items: center; justify-content: space-between;">
  <div>Left content</div>
  <div>Right content</div>
</div>
```

## Accessibility Guidelines

### Focus States

All interactive elements have visible focus indicators:
```css
*:focus-visible {
  outline: 3px solid var(--focus-ring-color);
  outline-offset: 2px;
}
```

### Keyboard Navigation

- **Tab**: Navigate forward
- **Shift+Tab**: Navigate backward
- **Enter/Space**: Activate buttons
- **Escape**: Close modals/dropdowns
- **Arrow keys**: Navigate lists/menus

### Screen Reader Support

- Use semantic HTML (`<nav>`, `<main>`, `<article>`)
- Provide `aria-label` for icon-only buttons
- Use `aria-describedby` for form hints
- Mark current page in navigation with `aria-current="page"`

### Color Contrast

All text meets WCAG AA standards:
- Normal text: 4.5:1 minimum
- Large text (18px+): 3:1 minimum
- Interactive elements: 3:1 minimum

## Responsive Breakpoints

```css
/* Mobile-first approach */
@media (max-width: 768px) {
  /* Tablet and below */
}

@media (max-width: 640px) {
  /* Mobile */
}
```

### Common Responsive Patterns

```css
/* Stack on mobile */
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
}

/* Hide on mobile */
@media (max-width: 768px) {
  .hide-mobile {
    display: none;
  }
}
```

## Best Practices

### Do's ✅

- Use design tokens for all styling
- Implement keyboard navigation
- Provide loading states
- Show error messages clearly
- Use semantic HTML
- Test with screen readers
- Optimize images
- Lazy load heavy components

### Don'ts ❌

- Don't use pixel values directly
- Don't skip focus states
- Don't use color alone to convey information
- Don't block keyboard access
- Don't use `<div>` for buttons
- Don't forget alt text on images
- Don't hardcode colors

## Dark Mode (Future)

The design system is prepared for dark mode:

```css
:root[data-theme="dark"] {
  --color-neutral-0: #111827;
  --color-neutral-900: #f9fafb;
  /* ... */
}
```

To enable: Set `data-theme="dark"` on root element.

## File Structure

```
apps/web/src/
├── app.css                    # Main stylesheet
├── lib/
│   ├── styles/
│   │   ├── design-tokens.css  # Design system tokens
│   │   └── components.css     # Component styles
│   └── components/
│       └── ui/
│           ├── Breadcrumb.svelte
│           ├── PageHeader.svelte
│           ├── Toast.svelte
│           ├── LoadingSpinner.svelte
│           └── UpdateNotification.svelte
└── routes/
    └── +layout.svelte         # Main layout
```

## Contributing

When adding new components:

1. Use design tokens, not hardcoded values
2. Include hover/focus/active states
3. Support keyboard navigation
4. Add loading/error states
5. Test on mobile
6. Document in this guide

## Resources

- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [Svelte Accessibility](https://svelte.dev/docs/accessibility-warnings)
- [WebAIM Contrast Checker](https://webaim.org/resources/contrastchecker/)
