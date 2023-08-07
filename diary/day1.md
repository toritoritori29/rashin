
# Day1. 最小限のHTTPサーバーの作成

## 目標
* 常に`204 No Contents`を返す最低限のHTTPサーバーを作成する.


## Socket通信とは


## システムコールとlibc
ChatGpt3.5に尋ねたところ以下のような回答が得られた。
```
システムコール（System Call）とlibc（C標準ライブラリ）は、オペレーティングシステムとCプログラムの間で重要な関係があります。
以下にそれぞれの概要と、彼らの関係性を説明します。

システムコール（System Call）：
システムコールは、オペレーティングシステム (OS) が提供するサービスや機能を呼び出すためのインターフェースです。
プロセス（Cプログラムを含む）がカーネル（オペレーティングシステムのコア部分）の機能にアクセスするために使用します。
一般的なシステムコールには、ファイルの読み書き、プロセスの作成・終了、ネットワーク通信などがあります。システムコールは低レベルのインターフェースであり、直接カーネルの機能を呼び出すことができます。

libc（C標準ライブラリ）：
libcは、C言語のプログラムを開発する際に便利な機能を提供するライブラリです。
標準Cライブラリとも呼ばれます。これには、C言語での文字列処理、メモリ操作、数学関数、ファイル入出力などが含まれています。
libcは、システムコールをより使いやすくする高レベルのインターフェースを提供し、プログラムがOS固有のシステムコールに直接依存せずにポータブルで移植性のあるコードを記述できるようにします。

関係性：
libcは、システムコールを利用してOSの機能にアクセスするためのAPI（アプリケーションプログラミングインターフェース）を提供します。
つまり、libc関数は、Cプログラムがシステムコールを抽象化して、より使いやすい形でOS機能を利用できるようにします。
```

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

``` python
poetry install
poetry run pytest
```

テストはtests/以下に存在していて、今回動かしたテストは正しく204を返すかどうか検証するものだ。
``` python
import subprocess
import requests
import time
import pytest

@pytest.fixture(scope="session", autouse=True)
def setup_server():
    process = subprocess.Popen("cargo run", shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    time.sleep(1)
    yield
    process.kill()


def test_204():
    resp = requests.get("http://localhost:8080/")
    assert resp.status_code == 204
```

## 課題
SIGINTを検知するためのフラグを実装しているが、acceptが処理をブロックする（入力があるまで待機する）のでループが回らずうまく機能していない。
今後epollという仕組みを使って, acceptが処理をブロックし続けないように修正していく。

## 参考文献