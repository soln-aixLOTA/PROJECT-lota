import { GenAI } from '@google/generative-ai';
import { metrics, SpanKind, SpanStatusCode, trace } from '@opentelemetry/api';

interface TokenCount {
    totalTokens: number;
    promptTokens: number;
    completionTokens: number;
}

interface GenerateContentConfig {
    temperature?: number;
    topP?: number;
    topK?: number;
    maxOutputTokens?: number;
    stopSequences?: string[];
}

interface EmbedContentConfig {
    outputDimensionality?: number;
}

interface GenerateContentResponse {
    text: string;
    safetyRatings?: any[];
}

export class InstrumentedGenAIClient {
    private tracer = trace.getTracer('genai-instrumentation');
    private meter = metrics.getMeter('genai-instrumentation');
    private client: GenAI;

    // Metric instruments
    private operationDuration;
    private tokenUsage;
    private requestCounter;
    private errorCounter;
    private latencyHistogram;

    constructor(apiKey: string) {
        this.client = new GenAI(apiKey);

        // Initialize metrics
        this.operationDuration = this.meter.createHistogram('gen_ai.client.operation.duration', {
            description: 'Duration of GenAI operations',
            unit: 's',
        });

        this.tokenUsage = this.meter.createHistogram('gen_ai.client.token.usage', {
            description: 'Number of tokens used in GenAI operations',
            unit: '{token}',
        });

        this.requestCounter = this.meter.createCounter('gen_ai.client.requests', {
            description: 'Number of GenAI API requests',
        });

        this.errorCounter = this.meter.createCounter('gen_ai.client.errors', {
            description: 'Number of GenAI API errors',
        });

        this.latencyHistogram = this.meter.createHistogram('gen_ai.client.latency', {
            description: 'Latency of GenAI API requests',
            unit: 'ms',
        });
    }

    private recordOperationMetrics(
        operationName: string,
        model: string,
        duration: number,
        tokenCount?: number,
        error?: string
    ): void {
        const attributes = {
            'gen_ai.operation.name': operationName,
            'gen_ai.model': model,
        };

        // Record operation duration
        this.operationDuration.record(duration, attributes);

        // Record latency in milliseconds
        this.latencyHistogram.record(duration * 1000, attributes);

        // Increment request counter
        this.requestCounter.add(1, attributes);

        // Record token usage if available
        if (tokenCount !== undefined) {
            this.tokenUsage.record(tokenCount, {
                ...attributes,
                'gen_ai.token.type': 'total',
            });
        }

        // Record error if present
        if (error) {
            this.errorCounter.add(1, {
                ...attributes,
                'gen_ai.error': error,
            });
        }
    }

    async generateContent(
        prompt: string,
        model: string = 'gemini-2.0-flash-exp',
        config?: GenerateContentConfig
    ): Promise<GenerateContentResponse> {
        const startTime = Date.now();
        let tokenCount: TokenCount | undefined;
        let errorMsg: string | undefined;

        const span = this.tracer.startSpan('gen_ai.generate_content', {
            kind: SpanKind.CLIENT,
            attributes: {
                'gen_ai.operation.name': 'generate',
                'gen_ai.model': model,
                'gen_ai.prompt': prompt,
                ...(config && { 'gen_ai.config': JSON.stringify(config) }),
            },
        });

        try {
            const response = await this.client.generateContent({
                model,
                prompt,
                ...config,
            });

            // Get token count
            tokenCount = await this.client.countTokens(model, prompt);
            span.setAttribute('gen_ai.token_count', tokenCount.totalTokens);
            span.setAttribute('gen_ai.response', response.text);
            span.setStatus({ code: SpanStatusCode.OK });

            return response;
        } catch (error) {
            errorMsg = error instanceof Error ? error.message : 'Unknown error';
            span.setAttribute('gen_ai.error', errorMsg);
            span.setStatus({
                code: SpanStatusCode.ERROR,
                message: errorMsg,
            });
            throw error;
        } finally {
            const duration = (Date.now() - startTime) / 1000; // Convert to seconds
            this.recordOperationMetrics(
                'generate_content',
                model,
                duration,
                tokenCount?.totalTokens,
                errorMsg
            );
            span.end();
        }
    }

    async embedContent(
        content: string,
        model: string = 'text-embedding-004',
        config?: EmbedContentConfig
    ): Promise<number[]> {
        const startTime = Date.now();
        let tokenCount: TokenCount | undefined;
        let errorMsg: string | undefined;

        const span = this.tracer.startSpan('gen_ai.embed_content', {
            kind: SpanKind.CLIENT,
            attributes: {
                'gen_ai.operation.name': 'embed',
                'gen_ai.model': model,
                'gen_ai.content': content,
                ...(config && { 'gen_ai.config': JSON.stringify(config) }),
            },
        });

        try {
            const response = await this.client.embedContent({
                model,
                content,
                ...config,
            });

            // Get token count
            tokenCount = await this.client.countTokens(model, content);
            span.setAttribute('gen_ai.token_count', tokenCount.totalTokens);
            span.setAttribute('gen_ai.embedding.dimensions', response.length);
            span.setStatus({ code: SpanStatusCode.OK });

            return response;
        } catch (error) {
            errorMsg = error instanceof Error ? error.message : 'Unknown error';
            span.setAttribute('gen_ai.error', errorMsg);
            span.setStatus({
                code: SpanStatusCode.ERROR,
                message: errorMsg,
            });
            throw error;
        } finally {
            const duration = (Date.now() - startTime) / 1000;
            this.recordOperationMetrics(
                'embed_content',
                model,
                duration,
                tokenCount?.totalTokens,
                errorMsg
            );
            span.end();
        }
    }
} 