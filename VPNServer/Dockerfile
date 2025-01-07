# AlmaLinux 9をベースイメージとする
FROM almalinux:9

# 1. dnf-plugins-core で CRBリポジトリを有効化し、必要パッケージをインストール
RUN dnf install -y epel-release dnf-plugins-core && \
    dnf config-manager --set-enabled crb && \
    dnf upgrade -y && \
    dnf install -y wireguard-tools \
                   gcc clang openssl-devel libstdc++-static make \
                   python3-pip \
    && dnf clean all

# 2. Rust のインストール
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
# cargo, rustc などをパスに通す
ENV PATH="/root/.cargo/bin:${PATH}"

# 3. cqlsh のインストール
RUN python3 -m pip install --no-cache-dir cqlsh

# 4. 作業ディレクトリを作成・移動
WORKDIR /app

# 5. ソースコードをコンテナへコピー
COPY . .

# 6. Rust プロジェクトをビルド (リリースビルド)
RUN cargo build --release

# 7. ポート開放 (WebSocket用など)
EXPOSE 8090

# 8. コンテナ起動時に実行するコマンド
CMD ["./target/release/vpnserver"]
