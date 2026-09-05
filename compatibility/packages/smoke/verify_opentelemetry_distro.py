"""Apply the distribution configuration in an isolated process; preserve explicit trace-exporter settings and supply missing metrics/log/protocol defaults."""
import os
from opentelemetry.distro import OpenTelemetryDistro
keys = ["OTEL_TRACES_EXPORTER", "OTEL_METRICS_EXPORTER", "OTEL_LOGS_EXPORTER", "OTEL_EXPORTER_OTLP_PROTOCOL"]
for key in keys:
    if key in os.environ:
        del os.environ[key]
os.environ["OTEL_TRACES_EXPORTER"] = "console"
OpenTelemetryDistro()._configure()
assert os.environ["OTEL_TRACES_EXPORTER"] == "console"
assert os.environ["OTEL_METRICS_EXPORTER"] == "otlp"
assert os.environ["OTEL_LOGS_EXPORTER"] == "otlp"
assert os.environ["OTEL_EXPORTER_OTLP_PROTOCOL"] == "grpc"
print("verified opentelemetry-distro")
