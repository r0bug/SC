FROM node:20-alpine

WORKDIR /app

# Install pnpm
RUN npm install -g pnpm

# Copy package files
COPY package.json pnpm-lock.yaml ./

# Install dependencies
RUN pnpm install --frozen-lockfile

# Copy source
COPY . .

# Expose port
EXPOSE 3001

# Start dev server
CMD ["pnpm", "dev", "--host", "0.0.0.0"]
