# --- build ---
FROM rust:latest AS builder

WORKDIR /app

ENV PATH="/root/.cargo/bin:${PATH}"

# install dx through cargo-binstall, as cargo-binstall is quite a lot smaller
RUN cargo install cargo-binstall 
RUN cargo binstall dioxus-cli

COPY . .

# build
RUN dx bundle --web --release


# --- runtime ---
FROM debian:stable-slim

WORKDIR /app

# backend binary
COPY --from=builder /app/target/dx/mng-schuelerziitig/release/web/mng-schuelerziitig /app/mng-schuelerziitig
# static files
COPY --from=builder /app/target/dx/mng-schuelerziitig/release/web/public /app/public

RUN mkdir svgs

EXPOSE 8080
CMD ["./mng-schuelerziitig"]
