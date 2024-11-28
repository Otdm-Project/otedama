# DBServerの環境構築手順
1. 以下の項目を指定してAWSでEC2インスタンスを構築する
    * 名前：DBServer
    * AMI：ami-0d9da98839203a9d1
    * インスタンスタイプ：t3.medium
    * キーペア：自身の使用するもの
    * ネットワーク設定：
    ![alt text](image.png)
    * ストレージ設定：8GB

2. ElasticIP:54.65.115.124を関連付け
