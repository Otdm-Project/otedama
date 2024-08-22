# Rustをインストール
```
sudo dnf update -y
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
sudo dnf groupinstall "Development Tools" -y
sudo dnf install cmake openssl-devel pkgconfig -y
```
## Rustプロジェクトの作成
```
cargo new apiclient
```
## プログラムのビルド
```
cargo build --release
```
## プログラムのビルド+実行
```
cargo run --release
```
