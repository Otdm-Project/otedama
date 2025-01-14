FROM rust:latest

# PGファイルを作成用のDIRに移動
WORKDIR /app
COPY . .

# すでにビルドは完了しているはずなのでバイナリを実行
CMD ["./target/release/proxyserver"]
