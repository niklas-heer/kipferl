"""Build a versioned resource URL with encoded query parameters and decode a synthetic JSON response."""
from python_http_client import Client
from python_http_client.client import Response
client = Client("https://example.com", version=3)
endpoint = client.users._("42")
assert endpoint._build_url({"q": "hello world"}) == "https://example.com/v3/users/42?q=hello+world"
class Incoming:
    def getcode(self):
        return 200
    def read(self):
        return b'{"answer": 42}'
    def info(self):
        return {"Content-Type": "application/json"}
response = Response(Incoming())
assert response.status_code == 200
assert response.to_dict == {"answer": 42}
assert response.headers["Content-Type"] == "application/json"
print("verified python-http-client")
