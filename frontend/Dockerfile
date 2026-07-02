# Build context is frontend/ (see docker-compose.yml).

# --- Build stage -------------------------------------------------------
FROM oven/bun:1 AS builder
WORKDIR /app

COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

COPY . .

# Vite inlines VITE_* variables into the bundle at build time, so the API
# URL the frontend will call has to be known now, not at container start.
ARG VITE_API_URL=http://localhost:8080/api
ENV VITE_API_URL=$VITE_API_URL
RUN bun run build

# --- Runtime stage -------------------------------------------------------
FROM nginx:alpine AS runtime
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
