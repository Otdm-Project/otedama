# APIServer構築手順
## Rustのインストール
```
sudo dnf update -y
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
sudo dnf groupinstall "Development Tools" -y
sudo dnf install cmake openssl-devel pkgconfig -y
```

## プログラムのビルド
```
cargo build --release
```
## プログラムをビルドして実行
```
cargo run --release
```
