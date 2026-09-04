"""Fetch JSON from an HTTP or HTTPS API."""
import argparse
import json
import sys

http = __import__("http.client")
# CPython returns the parent package; Kipferl returns the module directly.
http = getattr(http, "client", http)

parser = argparse.ArgumentParser(description='Fetch JSON from an HTTP or HTTPS API.')
parser.add_argument("host", help="Hostname without a URL scheme")
parser.add_argument("--path", default="/", help="Request path, including any query")
parser.add_argument("--port", type=int, default=0)
parser.add_argument("--https", action="store_true")
args = parser.parse_args()

connection_type = http.HTTPSConnection if args.https else http.HTTPConnection
port = args.port or (443 if args.https else 80)
try:
    connection = connection_type(args.host, port, timeout=10)
    connection.request("GET", args.path, headers={"Accept": "application/json"})
    response = connection.getresponse()
    if response.status < 200 or response.status >= 300:
        sys.stderr.write("API returned HTTP " + str(response.status) + "\n")
        sys.exit(1)
    print(json.dumps(json.loads(response.read().decode()), sort_keys=True))
except Exception as error:
    sys.stderr.write("Could not fetch JSON: " + str(error) + "\n")
    sys.exit(1)
