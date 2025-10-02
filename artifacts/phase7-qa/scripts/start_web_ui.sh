#!/bin/bash
export PORT="${PORT:-3001}"
cd ../web
exec node index.js
