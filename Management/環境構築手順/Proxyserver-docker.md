AWSインスタンスを構築するところまで省略

```
sudo dnf -y install dnf-plugins-core
sudo dnf config-manager --add-repo https://download.docker.com/linux/rhel/docker-ce.repo
sudo systemctl enable --now docker
```

以下を実行し正しくインストールできていることを確認
```
sudo docker run hello-world
```
sudo useradd -m proxyuser
echo "proxyuser ALL=(ALL)       ALL" | sudo tee /etc/sudoers.d/proxyuser
sudo chmod 0440 /etc/sudoers.d/proxyuser
sudo passwd proxyuser

パスワードを入力

sudo groupadd docker
sudo usermod -aG docker $USER

sudo dnf update -y

sudo dnf install -y haproxy
sudo dnf install curl -y
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo dnf install -y gcc openssl-devel pkgconfig