"""Read the complete CA bundle through certifi.contents() and certifi.where(); compare contents and PEM boundaries."""
import certifi
import os
pem = certifi.contents()
assert pem.count("-----BEGIN CERTIFICATE-----") > 50
assert pem.count("-----BEGIN CERTIFICATE-----") == pem.count("-----END CERTIFICATE-----")
path = certifi.where()
assert os.path.isfile(path)
with open(path) as bundle:
    assert bundle.read() == pem
assert certifi.where() == path
print("verified certifi")
