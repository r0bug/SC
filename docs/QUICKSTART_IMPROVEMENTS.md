# Quick Start: UI & Performance Improvements

This guide helps you quickly get started with the new UI improvements and benchmarking tools.

## Table of Contents
1. [UI Components Quick Start](#ui-components-quick-start)
2. [Running Benchmarks](#running-benchmarks)
3. [Testing the Improvements](#testing-the-improvements)
4. [Common Tasks](#common-tasks)

---

## UI Components Quick Start

### Using Toast Notifications (Easiest!)

Toast notifications are already integrated globally. Just import and use:

```svelte
<script>
  import { toasts } from '$lib/stores/toast';

  function handleSuccess() {
    toasts.success('Operation completed successfully!');
  }

  function handleError() {
    toasts.error('Something went wrong', 'Error', {
      action: {
        label: 'Retry',
        callback: () => console.log('Retrying...')
      }
    });
  }
</script>

<button on:click={handleSuccess}>Test Success</button>
<button on:click={handleError}>Test Error</button>
```

### Using Loading Spinner

```svelte
<script>
  import LoadingSpinner from '$lib/components/ui/LoadingSpinner.svelte';
  let loading = false;
</script>

{#if loading}
  <LoadingSpinner size="md" message="Loading data..." />
{:else}
  <!-- Your content here -->
{/if}
```

### Using Progress Bar

```svelte
<script>
  import ProgressBar from '$lib/components/ui/ProgressBar.svelte';
  let progress = 0;
  let timeRemaining = '2 min';
</script>

<ProgressBar
  progress={progress}
  label="Processing files"
  estimatedTimeRemaining={timeRemaining}
/>
```

---

## Running Benchmarks

### Shell Script Benchmark (HTTP Endpoints)

```bash
# 1. Start the sync service
cd /home/robug/Projects/sagenscontact/alpha
cargo run --bin sync_service

# 2. In another terminal, run the benchmark
./scripts/benchmark.sh

# 3. View the results
cat benchmark-results/benchmark_*.md
```

**Custom configuration:**
```bash
# Test with more users and longer duration
CONCURRENT_USERS=50 DURATION=60 ./scripts/benchmark.sh

# Test different API endpoint
API_URL=https://api.example.com ./scripts/benchmark.sh
```

### Rust Criterion Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench api_bench

# View HTML report
open target/criterion/report/index.html
# Or on Linux:
xdg-open target/criterion/report/index.html
```

---

## Testing the Improvements

### 1. Test Import Progress Indicators

1. Navigate to http://localhost:5173/import (or your dev server URL)
2. Select CSV format
3. Choose a file (any CSV file)
4. Watch the progress bar during import
5. Check toast notifications for success/error

### 2. Test File Upload Improvements

1. Navigate to any page with file upload (e.g., contacts)
2. Try uploading:
   - A normal file (should work with progress)
   - A file > 50MB (should show size error)
   - An unsupported file type (should show type error)
3. Watch for progress bar and toast notifications

### 3. Test AI Suggestions Loading

1. Navigate to any page with AI suggestions
2. Observe:
   - Loading spinner during fetch
   - Error state with retry button (if API fails)
   - Empty state message (if no suggestions)
   - Apply button with loading state

### 4. Test Error Handling

1. Disconnect internet
2. Try any API operation
3. Should see: "Unable to connect to the server. Please check your internet connection."
4. Reconnect and click Retry (if shown)

---

## Common Tasks

### Add Toast Notification to Existing Component

```svelte
<script>
  import { api, ApiError } from '$lib/api/api';
  import { toasts } from '$lib/stores/toast';

  async function saveData() {
    try {
      await api.save(data);
      toasts.success('Data saved successfully');
    } catch (error) {
      if (error instanceof ApiError) {
        toasts.error(error.getUserMessage(), 'Save Failed');
      } else {
        toasts.error('An unexpected error occurred');
      }
    }
  }
</script>
```

### Add Progress Tracking to Long Operation

```svelte
<script>
  import ProgressBar from '$lib/components/ui/ProgressBar.svelte';

  let processing = false;
  let progress = 0;
  let currentItem = 0;
  let totalItems = 0;

  async function processItems(items) {
    processing = true;
    progress = 0;
    totalItems = items.length;

    for (let i = 0; i < items.length; i++) {
      await processItem(items[i]);
      currentItem = i + 1;
      progress = ((i + 1) / items.length) * 100;
    }

    processing = false;
  }
</script>

{#if processing}
  <ProgressBar
    {progress}
    label={`Processing item ${currentItem} of ${totalItems}`}
  />
{/if}
```

### Add Loading Spinner to API Call

```svelte
<script>
  import { api } from '$lib/api/api';
  import LoadingSpinner from '$lib/components/ui/LoadingSpinner.svelte';

  let loading = true;
  let data = null;

  onMount(async () => {
    loading = true;
    try {
      data = await api.getData();
    } finally {
      loading = false;
    }
  });
</script>

{#if loading}
  <LoadingSpinner message="Loading data..." />
{:else if data}
  <!-- Display data -->
{/if}
```

---

## Troubleshooting

### Toast Notifications Not Showing

**Problem:** Toast notifications don't appear

**Solution:**
1. Check that Toast component is in root layout:
   ```svelte
   <!-- apps/web/src/routes/+layout.svelte -->
   <script>
     import Toast from '$lib/components/ui/Toast.svelte';
   </script>

   <Toast />
   <!-- rest of layout -->
   ```

2. Verify import:
   ```typescript
   import { toasts } from '$lib/stores/toast';
   ```

### Benchmark Script Not Running

**Problem:** `Permission denied` when running `./scripts/benchmark.sh`

**Solution:**
```bash
chmod +x scripts/benchmark.sh
./scripts/benchmark.sh
```

**Problem:** `curl: command not found`

**Solution:**
```bash
# Ubuntu/Debian
sudo apt-get install curl

# macOS
brew install curl
```

### Criterion Benchmarks Failing

**Problem:** Compilation errors in benchmarks

**Solution:**
```bash
# Update dependencies
cargo update

# Check Cargo.toml has correct configuration
cat crates/sync_service/Cargo.toml | grep -A 5 "dev-dependencies"
```

---

## Next Steps

1. **Read Full Documentation:**
   - [UI Improvements](./UI_IMPROVEMENTS.md) - Detailed component docs
   - [Performance Benchmarking](./PERFORMANCE_BENCHMARKING.md) - Complete benchmark guide
   - [Improvements Summary](./IMPROVEMENTS_SUMMARY.md) - Overview of all changes

2. **Integrate Into Your Workflow:**
   - Add toast notifications to existing error handling
   - Replace generic loading states with LoadingSpinner
   - Add progress bars to long-running operations
   - Run benchmarks weekly

3. **Contribute:**
   - Report issues or suggestions
   - Add new reusable components
   - Improve documentation
   - Add more benchmarks

---

## Quick Reference

### Import Statements

```typescript
// Toast notifications
import { toasts } from '$lib/stores/toast';

// UI Components
import LoadingSpinner from '$lib/components/ui/LoadingSpinner.svelte';
import ProgressBar from '$lib/components/ui/ProgressBar.svelte';

// API Error handling
import { api, ApiError } from '$lib/api/api';
```

### Toast Methods

```typescript
toasts.success(message, title?, options?)
toasts.error(message, title?, options?)
toasts.warning(message, title?, options?)
toasts.info(message, title?, options?)
toasts.remove(id)
toasts.clear()
```

### Benchmark Commands

```bash
# Shell benchmarks
./scripts/benchmark.sh

# Rust benchmarks
cargo bench
cargo bench --bench api_bench
cargo bench -- --baseline main
```

---

## Examples Repository

Check the following files for real-world examples:

- **Import with Progress:** `apps/web/src/routes/import/+page.svelte`
- **Upload with Progress:** `apps/web/src/lib/components/AttachmentUpload.svelte`
- **AI Loading States:** `apps/web/src/lib/components/AiSuggestions.svelte`
- **Enhanced API Client:** `apps/web/src/lib/api/api.ts`

---

**Need Help?** Check the main documentation files or create an issue in the repository.
