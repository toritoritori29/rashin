
# rashin

## 目指すゴール
シングルプロセス・マルチスレッドで動作するイベント駆動型のHTTP1.1サーバーを構築する

## 目次


## 開発環境のセットアップ
mac, windows上で開発しやすいように`.devcontainer`に開発環境用のコンテナの設定を記載してある.
vscodeのremote-container拡張機能を利用して当該コンテナを開けば, linux環境でなくても開発・動作検証を行うことができる.


## 基本的な操作

* 結合テストの実装
```bash
$ poetry run pytest
```

* format
```bash
$ cargo fmt
```