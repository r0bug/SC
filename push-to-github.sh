#!/bin/bash
#
# Helper script to push SagensContact to GitHub
#

echo "Setting up git configuration..."
git config user.email "robug@example.com"
git config user.name "robug"

echo "Committing changes..."
git commit -m "Initial commit: SagensContact Alpha

- Phase 7 security features (rate limiting, headers, validation)
- Turnkey installer with prerequisite detection
- Complete documentation
- 75 tests passing
- Production-ready systemd services
- Self-contained installer bundle
" || echo "Already committed"

echo "Setting branch to main..."
git branch -M main

echo "Adding remote..."
git remote add origin git@github.com:r0bug/SC.git 2>/dev/null || git remote set-url origin git@github.com:r0bug/SC.git

echo "Pushing to GitHub..."
git push -u origin main

echo ""
echo "✓ Repository pushed to: https://github.com/r0bug/SC"
echo ""
echo "Now update the installer URLs:"
echo "  sed -i 's|your-org/sagenscontact-alpha|r0bug/SC|g' sagenscontact-setup.sh"
