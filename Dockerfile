FROM rust:latest

WORKDIR /app

COPY . .

RUN cargo install --path .

EXPOSE 8000
ENTRYPOINT ["alertrs"]