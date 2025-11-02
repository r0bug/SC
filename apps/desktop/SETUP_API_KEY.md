# Setting Up Your Segmind API Key

## Quick Setup

### Method 1: Environment Variable (Recommended)

**On Linux/macOS:**
```bash
export SEGMIND_API_KEY="your-api-key-here"
cd apps/desktop
pnpm tauri dev
```

**On Windows (PowerShell):**
```powershell
$env:SEGMIND_API_KEY="your-api-key-here"
cd apps/desktop
pnpm tauri dev
```

**On Windows (CMD):**
```cmd
set SEGMIND_API_KEY=your-api-key-here
cd apps\desktop
pnpm tauri dev
```

### Method 2: .env File (For Development)

1. Copy the example file:
```bash
cp apps/desktop/.env.example apps/desktop/.env
```

2. Edit `apps/desktop/.env` and add your API key:
```bash
SEGMIND_API_KEY=your-actual-api-key-here
```

3. Run the app:
```bash
cd apps/desktop
pnpm tauri dev
```

### Method 3: System Environment Variable (Permanent)

**On Linux/macOS:**
Add to your `~/.bashrc` or `~/.zshrc`:
```bash
export SEGMIND_API_KEY="your-api-key-here"
```

Then reload:
```bash
source ~/.bashrc  # or ~/.zshrc
```

**On Windows:**
1. Search for "Environment Variables" in Windows settings
2. Click "Environment Variables"
3. Under "User variables", click "New"
4. Variable name: `SEGMIND_API_KEY`
5. Variable value: `your-api-key-here`
6. Click OK

## Getting a Segmind API Key

1. Visit https://www.segmind.com/
2. Sign up for an account
3. Navigate to API settings
4. Generate a new API key
5. Copy the key

## Testing Your Setup

Run the desktop app and check the logs:

**With API Key:**
```
INFO  Segmind AI enabled with API key
```

**Without API Key (Mock Mode):**
```
WARN  Segmind AI running in MOCK MODE (no API key found)
WARN  Set SEGMIND_API_KEY environment variable to enable real AI
```

## Mock Mode

If you don't set an API key, the app runs in **mock mode**:
- ✓ All features work
- ✓ AI responses are simulated
- ✓ No API calls made
- ✓ ~200ms simulated delay
- ✓ Perfect for testing and development

## Security Notes

⚠️ **NEVER commit your API key to git!**

The `.env` file is already in `.gitignore`. Always use:
- Environment variables for production
- `.env` file for local development only

## Troubleshooting

**App still says "Mock Mode" after setting key:**
1. Restart the desktop app completely
2. Check the environment variable is set: `echo $SEGMIND_API_KEY`
3. Make sure there are no quotes or spaces around the key

**Import not using AI:**
1. Check logs for "Segmind AI enabled" message
2. Try analyzing a test file from `sample_data/`
3. Check network connectivity if API calls fail
