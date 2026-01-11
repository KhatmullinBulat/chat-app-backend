# Этап сборки
FROM rust:1.78-slim-bookworm AS builder
WORKDIR /usr/src/app

# Копируем файлы проекта
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Собираем в релизном режиме
RUN cargo build --release

# Финальный этап запуска
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# Копируем бинарник из этапа сборки
COPY --from=builder /usr/src/app/target/release/backend /usr/local/bin/app

# Сообщаем, что приложение использует порт 3000 (для информации)
EXPOSE 3000

# Команда запуска
CMD ["app"]