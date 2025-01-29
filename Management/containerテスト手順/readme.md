# コンテナテスト手順

## vmを用意

2v8Gib以上を推奨
ubuntu

## 環境セットアップ

- docker インストール

``` bash
# Add Docker's official GPG key:
sudo apt-get update
sudo apt-get install ca-certificates curl
sudo install -m 0755 -d /etc/apt/keyrings
sudo curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
sudo chmod a+r /etc/apt/keyrings/docker.asc

# Add the repository to Apt sources:
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu \
  $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
  sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
sudo apt-get update
sudo apt-get install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
sudo usermod -aG docker $USER
newgrp docker
```

- git インストール

```bash
  sudo apt update && sudo apt upgrade -y
  sudo apt install -y git
```

- サーバリポジトリを持ってくる

```bash
git clone https://github.com/Otdm-Project/otedama.git
cd otedama
git fetch origin OrganizeDockerfile
git switch OrganizeDockerfile
```

## コンテナのbuild
```bash
cd docker-compose/
docker compose build --no-cache
```