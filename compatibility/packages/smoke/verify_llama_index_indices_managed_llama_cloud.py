"""Require an error when no index identifier is supplied and when two competing identifiers are supplied."""
from llama_index.indices.managed.llama_cloud import LlamaCloudIndex
for options in [{}, {"name": "one", "pipeline_id": "two"}]:
    rejected = False
    try:
        LlamaCloudIndex(**options)
    except ValueError as error:
        assert "Exactly one" in str(error)
        rejected = True
    assert rejected
print("verified llama-index-indices-managed-llama-cloud")
