AWSインスタンスを構築するところまで省略

```
sudo useradd -m proxyuser
echo "proxyuser ALL=(ALL)       ALL" | sudo tee /etc/sudoers.d/proxyuser
sudo chmod 0440 /etc/sudoers.d/proxyuser
sudo passwd proxyuser
```
パスワードを入れる
```
sudo su - proxyuser 
```

```
sudo -u proxyuser mkdir -p /home/proxyuser/.ssh
sudo -u proxyuser chmod 700 /home/proxyuser/.ssh
echo "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIICw4ZzLPjsKazxZUhnk81ODO4WrYelXacg5717HDQJZ managementuser@management-server" | sudo tee -a /home/proxyuser/.ssh/authorized_keys
sudo chmod 600 /home/proxyuser/.ssh/authorized_keys
sudo chown -R proxyuser:proxyuser /home/proxyuser/.ssh
sudo dnf update -y
```

```
sudo dnf install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
sudo dnf -y install dnf-plugins-core
sudo dnf config-manager --add-repo https://download.docker.com/linux/rhel/docker-ce.repo
sudo systemctl enable --now docker
sudo groupadd docker
sudo usermod -aG docker $USER
sudo dnf install curl -y
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
sudo dnf install -y gcc openssl-devel pkgconfig
```

以下を実行し正しくインストールできていることを確認
```
sudo docker run hello-world
```