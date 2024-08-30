FROM node:21 as frontend-builder

WORKDIR /frontend
ENV NODE_ENV production
COPY package.json package-lock.json ./
RUN npm ci && npm cache clean --force
COPY . .
RUN npm run build -- --outDir /frontend-build --emptyOutDir