
# Day1. 最小限のHTTPサーバーの作成
* 常に`204 No Contents`を返す最低限のHTTPサーバーを作成する.


## Socket通信とは


## システムコールとlibc
要調査

## Day1に作成するサーバーの全体像
Day1では非常にシンプルな機能のみを持つサーバーを構築する。
新しいsocketを作成した後、以下の順番で必要なシステムコールを呼ぶ.

* listen - 接続待ちsocketとして登録する.
* bind - socketに対してIPアドレスとポートを紐付ける
* accept - 接続があるまで待機.
* read(recv) - データの読み込み
* write(send) - データの書き込み
* shutdown - 


## 試してみる
このプロジェクトではpythonを使ってe2eテストを実装する。
以下のコマンドで依存関係をインストールしてテストを実行する。
テストの実態はtests/以下に存在している。

``` python
poetry install
poetry run pytest
```


## 参考文献