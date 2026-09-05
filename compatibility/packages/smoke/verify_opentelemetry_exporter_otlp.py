"""Create and end one SDK span, export through a recording HTTP session, and require a nonempty serialized payload and success result."""
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import SpanExportResult
from opentelemetry.exporter.otlp.proto.http.trace_exporter import OTLPSpanExporter
class Reply:
    ok = True
class Session:
    def __init__(self):
        self.headers = {}
        self.calls = []
    def post(self, **kwargs):
        self.calls.append(kwargs)
        return Reply()
    def close(self):
        pass
session = Session()
exporter = OTLPSpanExporter(endpoint="https://example.invalid/v1/traces", session=session)
provider = TracerProvider()
span = provider.get_tracer("verification").start_span("work")
span.end()
assert exporter.export([span]) == SpanExportResult.SUCCESS
assert len(session.calls) == 1
assert session.calls[0]["url"] == "https://example.invalid/v1/traces"
assert len(session.calls[0]["data"]) > 0
exporter.shutdown()
provider.shutdown()
print("verified opentelemetry-exporter-otlp")
