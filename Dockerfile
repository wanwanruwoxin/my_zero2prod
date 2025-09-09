FROM rust:1.89.0

# 切换工作目录，类似于 `cd app`
# docker会帮我们创建 app 文件夹，防止它不存在
WORKDIR /app
# 下载必要的系统依赖，为了 linking configuration
RUN apt update && apt install lld clang -y
# 复制所有文件到 docker image 中
COPY . .
# 构建可执行文件
RUN cargo build --release
ENV APP_ENVIRONMENT=production
ENTRYPOINT ["./target/release/zero2prod"]

