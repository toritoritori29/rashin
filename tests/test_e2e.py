import subprocess
import requests
import time
import pytest

import concurrent.futures

@pytest.fixture(scope="session", autouse=True)
def setup_server():
    process = subprocess.Popen("cargo run", shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    time.sleep(3)
    yield
    process.kill()


def test_204():
    def test_func(url):
        try: 
            resp = requests.get(url)
            return resp.status_code == 204
        except:
            return False

    with concurrent.futures.ThreadPoolExecutor(max_workers=100) as executor:
        args = ["http://localhost:8080/"] * 100
        results = list(executor.map(test_func, args))
        ratio = sum(results) / len(results)
        assert ratio == 1.0
