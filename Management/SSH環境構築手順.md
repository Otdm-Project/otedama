
## 各SSH先サーバでの設定
### ホスト名を変更
```
sudo hostnamectl set-hostname XXX-server
```
### 各サーバ用のユーザを追加
```
sudo useradd -m xxxuser
```
ユーザをsudoersに追加
```
sudo visudo
```
以下を追加
```
apiuser ALL=(ALL)       ALL
```
パスワードを設定
```
sudo passwd xxxuser
```
ユーザ変更できることを確認
```
su - xxxuser
```
SSH用の設定を追加
```
mkdir .ssh 
cd .ssh 
```
```
sudo vi authorized_keys
```
を実行し、先程catした公開鍵を貼り付け

再起動して設定適用
```
sudo systemctl restart sshd 
```


### デフォルトユーザを削除
**この作業を行う前に必ずAMI化を行いバックアップから復旧ができるようにしてください**
```
userdel -r ec2-user
```
sudo hostnamectl set-hostname db-server
sed -e '$s|$|-tail|g' in > out

ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIICw4ZzLPjsKazxZUhnk81ODO4WrYelXacg5717HDQJZ managementuser@management-server