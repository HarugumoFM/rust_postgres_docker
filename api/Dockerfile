FROM rust:1.85 as builder

WORKDIR /usr/src/app

# キャッシュ効率化のためにまずCargo.tomlとCargo.lockをコピー
COPY Cargo.toml Cargo.lock* ./

# 実際のソースコードをコピーしてビルド
COPY /src ./src
RUN cargo build --release


CMD ["./target/release/api"]
