FROM rust:1.89.0 AS builder

# 切换工作目录，类似于 `cd app`
# docker会帮我们创建 app 文件夹，防止它不存在
WORKDIR /app
# 下载必要的系统依赖，为了 linking configuration
RUN apt update && apt install lld clang -y
# 复制所有文件到 docker image 中
COPY . .
# 构建可执行文件
RUN cargo build --release

FROM debian:bullseye-slim AS runtime
WORKDIR /app

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/zero2prod zero2prod
# 在运行时我们需要配置文件
COPY configuration configuration
ENV APP_ENVIRONMENT=production
ENTRYPOINT ["./zero2prod"]

