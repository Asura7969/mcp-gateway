# base image
FROM node:18-alpine AS base
LABEL maintainer="mcp-gateway@example.com"

# Set npm registry to use Taobao mirror
RUN npm config set registry https://registry.npmmirror.com

# Install pnpm
RUN npm install -g pnpm

# install packages
FROM base AS packages

WORKDIR /app/web

COPY package.json .
COPY pnpm-lock.yaml .

RUN pnpm install --no-frozen-lockfile

# build resources
FROM base AS builder

WORKDIR /app/web

# Copy package files and install dependencies
COPY package.json .
COPY pnpm-lock.yaml .
RUN pnpm install --no-frozen-lockfile

# Copy source code
COPY . .

ENV NODE_OPTIONS="--max-old-space-size=4096"
RUN pnpm vite build

# production stage
FROM base AS production

ENV NODE_ENV=production

# set timezone
ENV TZ=UTC
RUN ln -s /usr/share/zoneinfo/${TZ} /etc/localtime \
    && echo ${TZ} > /etc/timezone

WORKDIR /app/web

# Copy built assets from builder stage
COPY --from=builder /app/web/dist ./dist

# Install serve to serve static files
RUN npm install -g serve

ARG COMMIT_SHA
ENV COMMIT_SHA=${COMMIT_SHA}

EXPOSE 3000
CMD ["serve", "-s", "dist", "-l", "3000"]