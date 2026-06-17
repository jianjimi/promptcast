# 多阶段：装依赖+构建 → 运行精简镜像。
FROM node:22-alpine AS build
WORKDIR /app
COPY package.json ./
RUN npm install
COPY prisma ./prisma
RUN npx prisma generate
COPY tsconfig*.json nest-cli.json ./
COPY src ./src
RUN npm run build

FROM node:22-alpine AS runtime
WORKDIR /app
ENV NODE_ENV=production
COPY package.json ./
RUN npm install --omit=dev
COPY prisma ./prisma
RUN npx prisma generate
COPY --from=build /app/dist ./dist
EXPOSE 3000
# 启动先把迁移部署到库（建表 + 序列），再起服务。
CMD ["sh", "-c", "npx prisma migrate deploy && node dist/main.js"]
