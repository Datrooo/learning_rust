# -- Build stage --
FROM rust:latest AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
# Создаём пустой src для кеширования зависимостей
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 
# Копируем код и собираем
COPY src ./src
RUN touch src/main.rs && cargo build --release

# -- Runtime stage --
# Используем debian:trixie-slim чтобы GLIBC совпадал с rust:latest (основан на Trixie)
FROM debian:trixie-slim
RUN apt-get update && apt-get install -y ca-certificates ffmpeg && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/audio_upload_service /usr/local/bin/app
EXPOSE 8080
CMD ["app"]
