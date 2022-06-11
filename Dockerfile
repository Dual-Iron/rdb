FROM rust:1.61

ARG PORT

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=$PORT
ENV ROCKET_KEEP_ALIVE=0

WORKDIR /app
COPY . .

RUN cargo build --release

CMD ROCKET_PORT=$PORT ./target/release/rdb
