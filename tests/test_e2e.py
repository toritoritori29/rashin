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
