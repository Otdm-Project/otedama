# VPNServer構築手順
以下の項目を指定してAWSでEC2インスタンスを構築する
* 名前：VPN
* AMI：ami-0d9da98839203a9d1
* インスタンスタイプ：t3.small
* キーペア：自身の使用するもの
* ネットワーク設定：

* ストレージ設定：8GB
ElasticIP:54.178.75.68を関連付け
SSH接続
```
ssh ec2-user@35.73.31.183
```
インストールと設定を入れる
```
sudo useradd -m vpnuser
echo "vpnuser ALL=(ALL)       ALL" | sudo tee /etc/sudoers.d/vpnuser
sudo chmod 0440 /etc/sudoers.d/vpnuser
sudo passwd vpnuser
```
パスワードを入力
```
sudo su - apiuser 
```
sudo -u apiuser mkdir -p /home/apiuser/.ssh
sudo -u apiuser chmod 700 /home/apiuser/.ssh
echo "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIICw4ZzLPjsKazxZUhnk81ODO4WrYelXacg5717HDQJZ managementuser@management-server" | sudo tee -a /home/apiuser/.ssh/authorized_keys
sudo chmod 600 /home/apiuser/.ssh/authorized_keys
sudo chown -R apiuser:apiuser /home/apiuser/.ssh
sudo dnf update -y 
sudo -u vpnserver mkdir -p /home/vpnserver/.ssh
sudo -u vpnserver chmod 700 /home/vpnserver/.ssh
echo "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIICw4ZzLPjsKazxZUhnk81ODO4WrYelXacg5717HDQJZ managementuser@management-server" | sudo tee -a /home/vpnserver/.ssh/authorized_keys
sudo chmod 600 /home/vpnserver/.ssh/authorized_keys
sudo chown -R vpnserver:vpnserver /home/vpnserver/.ssh
