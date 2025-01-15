from opentelemetry import trace, metrics
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import ConsoleSpanExporter, BatchSpanProcessor
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import ConsoleMetricExporter, PeriodicExportingMetricReader
from opentelemetry.semconv.ai import SpanAttributes, GenAiOperationNameValues
from opentelemetry.trace import Status, StatusCode
from google import genai
from google.genai import types
import time
import asyncio
from typing import Optional, List, Dict, Any, AsyncIterator
import logging
from datetime import datetime

# Initialize OpenTelemetry tracing
tracer_provider = TracerProvider()
trace.set_tracer_provider(tracer_provider)
tracer_provider.add_span_processor(BatchSpanProcessor(ConsoleSpanExporter()))
tracer = trace.get_tracer("genai_instrumentation")

# Initialize OpenTelemetry metrics
metric_reader = PeriodicExportingMetricReader(ConsoleMetricExporter())
meter_provider = MeterProvider(metric_readers=[metric_reader])
metrics.set_meter_provider(meter_provider)
meter = metrics.get_meter("genai_instrumentation")

# Create metric instruments
operation_duration = meter.create_histogram(
    name="gen_ai.client.operation.duration",
    description="Duration of GenAI operations",
    unit="s"
)

token_usage = meter.create_histogram(
    name="gen_ai.client.token.usage",
    description="Number of tokens used in GenAI operations",
    unit="{token}"
)

request_counter = meter.create_counter(
    name="gen_ai.client.requests",
    description="Number of GenAI API requests",
    unit="1"
)

error_counter = meter.create_counter(
    name="gen_ai.client.errors",
    description="Number of GenAI API errors",
    unit="1"
)

latency_histogram = meter.create_histogram(
    name="gen_ai.client.latency",
    description="Latency of GenAI API requests",
    unit="ms"
)

class InstrumentedGenAIClient:
    def __init__(self, api_key: str):
        self.client = genai.Client(api_key=api_key)

    def _record_operation_metrics(self, operation_name: str, model: str, duration: float,
                                token_count: Optional[int] = None, error: Optional[str] = None):
        """Record common metrics for GenAI operations"""
        # Record operation duration
        operation_duration.record(
            duration,
            {
                "gen_ai.operation.name": operation_name,
                "gen_ai.model": model
            }
        )

        # Record latency in milliseconds
        latency_histogram.record(
            duration * 1000,
            {
                "gen_ai.operation.name": operation_name,
                "gen_ai.model": model
            }
        )

        # Increment request counter
        request_counter.add(
            1,
            {
                "gen_ai.operation.name": operation_name,
                "gen_ai.model": model
            }
        )

        # Record token usage if available
        if token_count is not None:
            token_usage.record(
                token_count,
                {
                    "gen_ai.token.type": "total",
                    "gen_ai.model": model
                }
            )

        # Record error if present
        if error:
            error_counter.add(1, labels={
                "gen_ai.error": error,
                "timestamp": str(datetime.utcnow())  # Add timestamp for context
            })
            logging.error(f"GenAI error recorded: {error}")  # Log the error

    def generate_content(self, prompt: str, model: str = "gemini-2.0-flash-exp",
                        config: Optional[types.GenerateContentConfig] = None) -> Any:
        with tracer.start_as_current_span("gen_ai.generate_content") as span:
            # Set common attributes
            span.set_attribute(SpanAttributes.GEN_AI_OPERATION_NAME, GenAiOperationNameValues.GENERATE)
            span.set_attribute(SpanAttributes.GEN_AI_MODEL, model)
            span.set_attribute(SpanAttributes.GEN_AI_PROMPT, prompt)
            if config:
                span.set_attribute("gen_ai.config", str(config))

            start_time = time.time()
            error_msg = None
            try:
                # Make the API call
                response = self.client.models.generate_content(
                    model=model,
                    contents=prompt,
                    config=config
                )

                # Record response attributes
                span.set_attribute(SpanAttributes.GEN_AI_RESPONSE, response.text)

                # Get token counts
                token_count = self.client.models.count_tokens(
                    model=model,
                    contents=prompt
                )
                span.set_attribute(SpanAttributes.GEN_AI_TOKEN_COUNT, token_count.total_tokens)

                span.set_status(Status(StatusCode.OK))
                return response

            except Exception as e:
                error_msg = str(e)
                span.set_attribute(SpanAttributes.GEN_AI_ERROR, error_msg)
                span.record_exception(e)
                span.set_status(Status(StatusCode.ERROR, error_msg))
                raise
            finally:
                duration = time.time() - start_time
                self._record_operation_metrics(
                    operation_name="generate_content",
                    model=model,
                    duration=duration,
                    token_count=token_count.total_tokens if 'token_count' in locals() else None,
                    error=error_msg
                )

    async def generate_content_stream(self, prompt: str, model: str = "gemini-2.0-flash-exp",
                                    config: Optional[types.GenerateContentConfig] = None) -> AsyncIterator[Any]:
        with tracer.start_as_current_span("gen_ai.generate_content_stream") as span:
            span.set_attribute(SpanAttributes.GEN_AI_OPERATION_NAME, GenAiOperationNameValues.GENERATE_STREAM)
            span.set_attribute(SpanAttributes.GEN_AI_MODEL, model)
            span.set_attribute(SpanAttributes.GEN_AI_PROMPT, prompt)
            if config:
                span.set_attribute("gen_ai.config", str(config))

            start_time = time.time()
            error_msg = None
            chunks = []

            try:
                async for chunk in self.client.aio.models.generate_content_stream(
                    model=model,
                    contents=prompt,
                    config=config
                ):
                    chunks.append(chunk.text)
                    yield chunk

                # Record final response
                full_response = "".join(chunks)
                span.set_attribute(SpanAttributes.GEN_AI_RESPONSE, full_response)

                # Get token counts for the full response
                token_count = self.client.models.count_tokens(
                    model=model,
                    contents=prompt + full_response
                )
                span.set_attribute(SpanAttributes.GEN_AI_TOKEN_COUNT, token_count.total_tokens)

                span.set_status(Status(StatusCode.OK))

            except Exception as e:
                error_msg = str(e)
                span.set_attribute(SpanAttributes.GEN_AI_ERROR, error_msg)
                span.record_exception(e)
                span.set_status(Status(StatusCode.ERROR, error_msg))
                raise
            finally:
                duration = time.time() - start_time
                self._record_operation_metrics(
                    operation_name="generate_content_stream",
                    model=model,
                    duration=duration,
                    token_count=token_count.total_tokens if 'token_count' in locals() else None,
                    error=error_msg
                )

    def embed_content(self, content: str, model: str = "text-embedding-004",
                     config: Optional[types.EmbedContentConfig] = None) -> Any:
        with tracer.start_as_current_span("gen_ai.embed_content") as span:
            span.set_attribute(SpanAttributes.GEN_AI_OPERATION_NAME, GenAiOperationNameValues.EMBED)
            span.set_attribute(SpanAttributes.GEN_AI_MODEL, model)
            span.set_attribute("gen_ai.content", content)
            if config:
                span.set_attribute("gen_ai.config", str(config))

            start_time = time.time()
            error_msg = None

            try:
                # Make the API call
                response = self.client.models.embed_content(
                    model=model,
                    contents=content,
                    config=config
                )

                # Record embedding dimensions
                span.set_attribute("gen_ai.embedding.dimensions", len(response.embedding))

                # Get token counts
                token_count = self.client.models.count_tokens(
                    model=model,
                    contents=content
                )
                span.set_attribute(SpanAttributes.GEN_AI_TOKEN_COUNT, token_count.total_tokens)

                span.set_status(Status(StatusCode.OK))
                return response

            except Exception as e:
                error_msg = str(e)
                span.set_attribute(SpanAttributes.GEN_AI_ERROR, error_msg)
                span.record_exception(e)
                span.set_status(Status(StatusCode.ERROR, error_msg))
                raise
            finally:
                duration = time.time() - start_time
                self._record_operation_metrics(
                    operation_name="embed_content",
                    model=model,
                    duration=duration,
                    token_count=token_count.total_tokens if 'token_count' in locals() else None,
                    error=error_msg
                )

async def main():
    # Initialize the instrumented client
    client = InstrumentedGenAIClient(api_key="YOUR_API_KEY")

    try:
        # Example 1: Generate content
        response = client.generate_content(
            prompt="Explain quantum computing in simple terms",
            model="gemini-2.0-flash-exp"
        )
        print(f"Generated response: {response.text}\n")

        # Example 2: Generate content with streaming
        print("Streaming response:")
        async for chunk in client.generate_content_stream(
            prompt="Write a short story about AI",
            model="gemini-2.0-flash-exp"
        ):
            print(chunk.text, end="")
        print("\n")

        # Example 3: Generate embeddings
        embedding_response = client.embed_content(
            content="This is a test sentence for embedding",
            model="text-embedding-004"
        )
        print(f"Embedding dimensions: {len(embedding_response.embedding)}\n")

    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    asyncio.run(main())
