import subprocess
import requests
import time
import pytest
import fcntl
import os

import concurrent.futures

@pytest.fixture(scope="session", autouse=True)
def setup_server():
    process = subprocess.Popen("RUST_LOG=main=debug cargo run", shell=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    time.sleep(10)
    yield
    # stdoutをノンブロッキングモードにする
    flag = fcntl.fcntl(process.stdout.fileno(), fcntl.F_GETFL)
    fcntl.fcntl(process.stdout.fileno(), fcntl.F_SETFL, flag | os.O_NONBLOCK)

    bytes = process.stdout.read()
    with open("dump.txt", "w") as f:
        f.write(bytes.decode("utf-8"))
    lines = bytes.decode("utf-8").split("\n")
    for line in lines:
        print(line)
    process.kill()


def test_204():
    def test_func(idx):
        url = f"http://localhost:8080/{idx}"
        try: 
            resp = requests.get(url, timeout=1)
            return resp.status_code == 204
        except Exception as e:
            print(url)
            print(e)
            return False

    with concurrent.futures.ThreadPoolExecutor(max_workers=100) as executor:
        args = list(range(100))
        results = list(executor.map(test_func, args))
        ratio = sum(results) / len(results)
        assert ratio == 1.0
