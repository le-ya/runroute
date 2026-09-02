FROM node:22.14.0-bookworm-slim AS build
WORKDIR /src
COPY .prettierrc .prettierignore ./
COPY gpx/package.json gpx/package-lock.json gpx/tsconfig.json gpx/
COPY gpx/src/ gpx/src/
RUN npm ci --prefix gpx
COPY website/package.json website/package-lock.json website/.npmrc website/svelte.config.js website/tailwind.config.js website/tsconfig.json website/vite.config.ts website/components.json website/
COPY website/src/ website/src/
COPY website/static/ website/static/
ARG PUBLIC_MAPTILER_KEY=""
ENV PUBLIC_MAPTILER_KEY=${PUBLIC_MAPTILER_KEY}
RUN npm ci --prefix website && npm run build --prefix website

FROM nginxinc/nginx-unprivileged:1.27.2-alpine
COPY deploy/website/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /src/website/build/ /usr/share/nginx/html/
EXPOSE 8080
