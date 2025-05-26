FROM node:22.16.0 AS frontend-builder

WORKDIR /frontend
ENV NODE_ENV=production
COPY . .
RUN npm ci
RUN npm run build

FROM alpine AS copier
COPY --from=frontend-builder /frontend/dist /frontend-build
