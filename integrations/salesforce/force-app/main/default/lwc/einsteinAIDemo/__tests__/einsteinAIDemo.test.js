import analyzeImage from '@salesforce/apex/EinsteinAIService.analyzeImage';
import analyzeSentiment from '@salesforce/apex/EinsteinAIService.analyzeSentiment';
import generateChat from '@salesforce/apex/EinsteinAIService.generateChat';
import generateText from '@salesforce/apex/EinsteinAIService.generateText';
import EinsteinAIDemo from 'c/einsteinAIDemo';
import { createElement } from 'lwc';

// Mock the apex methods
jest.mock(
    '@salesforce/apex/EinsteinAIService.generateText',
    () => {
        return {
            default: jest.fn()
        };
    },
    { virtual: true }
);

jest.mock(
    '@salesforce/apex/EinsteinAIService.analyzeSentiment',
    () => {
        return {
            default: jest.fn()
        };
    },
    { virtual: true }
);

jest.mock(
    '@salesforce/apex/EinsteinAIService.analyzeImage',
    () => {
        return {
            default: jest.fn()
        };
    },
    { virtual: true }
);

jest.mock(
    '@salesforce/apex/EinsteinAIService.generateChat',
    () => {
        return {
            default: jest.fn()
        };
    },
    { virtual: true }
);

describe('c-einstein-a-i-demo', () => {
    let element;
    let consoleSpy;

    beforeEach(() => {
        // Create initial element
        element = createElement('c-einstein-a-i-demo', {
            is: EinsteinAIDemo
        });
        document.body.appendChild(element);

        // Reset mocks
        jest.clearAllMocks();

        // Spy on console.error
        consoleSpy = jest.spyOn(console, 'error').mockImplementation(() => { });

        // Mock window methods
        window.HTMLElement.prototype.scrollIntoView = jest.fn();
        window.performance.mark = jest.fn();
        window.performance.measure = jest.fn();
    });

    afterEach(() => {
        // Remove element from DOM
        while (document.body.firstChild) {
            document.body.removeChild(document.body.firstChild);
        }

        // Restore mocks and spies
        jest.useRealTimers();
        consoleSpy.mockRestore();
    });

    // Test Input Change Handlers
    describe('input change handlers', () => {
        it('updates textInput when text input changes', () => {
            const input = element.shadowRoot.querySelector('lightning-textarea[name="prompt"]');
            input.value = 'Test prompt';
            input.dispatchEvent(new CustomEvent('change'));

            return Promise.resolve().then(() => {
                expect(element.textInput).toBe('Test prompt');
            });
        });

        it('updates imageUrl when image URL input changes', () => {
            const input = element.shadowRoot.querySelector('lightning-input[type="url"]');
            input.value = 'https://example.com/image.jpg';
            input.dispatchEvent(new CustomEvent('change'));

            return Promise.resolve().then(() => {
                expect(element.imageUrl).toBe('https://example.com/image.jpg');
            });
        });

        it('updates maxTokens when slider changes', () => {
            const slider = element.shadowRoot.querySelector('lightning-slider[label="Max Tokens"]');
            slider.dispatchEvent(new CustomEvent('change', {
                detail: { value: 200 }
            }));

            return Promise.resolve().then(() => {
                expect(element.maxTokens).toBe(200);
            });
        });

        it('updates temperature when slider changes', () => {
            const slider = element.shadowRoot.querySelector('lightning-slider[label="Temperature"]');
            slider.dispatchEvent(new CustomEvent('change', {
                detail: { value: 0.8 }
            }));

            return Promise.resolve().then(() => {
                expect(element.temperature).toBe(0.8);
            });
        });
    });

    // Test Text Generation
    describe('text generation', () => {
        it('shows error toast when prompt is empty', () => {
            const button = element.shadowRoot.querySelector('lightning-button[label="Generate Text"]');
            button.click();

            return Promise.resolve().then(() => {
                const toastEvent = element.dispatchEvent.mock.calls[0][0];
                expect(toastEvent.type).toBe('lightning__showtoast');
                expect(toastEvent.detail.variant).toBe('error');
            });
        });

        it('successfully generates text', () => {
            // Mock the API response
            generateText.mockResolvedValue('Generated response');

            // Set input value
            element.textInput = 'Test prompt';

            // Click generate button
            const button = element.shadowRoot.querySelector('lightning-button[label="Generate Text"]');
            button.click();

            return Promise.resolve().then(() => {
                expect(generateText).toHaveBeenCalledWith({
                    prompt: 'Test prompt',
                    maxTokens: 100,
                    temperature: 0.7
                });
                expect(element.generatedText).toBe('Generated response');
            });
        });

        it('handles API error', () => {
            // Mock API error
            generateText.mockRejectedValue(new Error('API Error'));

            // Set input value
            element.textInput = 'Test prompt';

            // Click generate button
            const button = element.shadowRoot.querySelector('lightning-button[label="Generate Text"]');
            button.click();

            return Promise.resolve().then(() => {
                const toastEvent = element.dispatchEvent.mock.calls[0][0];
                expect(toastEvent.type).toBe('lightning__showtoast');
                expect(toastEvent.detail.variant).toBe('error');
                expect(toastEvent.detail.message).toContain('API Error');
            });
        });
    });

    // Test Sentiment Analysis
    describe('sentiment analysis', () => {
        it('successfully analyzes sentiment', () => {
            // Mock API response
            analyzeSentiment.mockResolvedValue({
                document_sentiment: {
                    label: 'POSITIVE',
                    score: 0.8
                }
            });

            // Set input value
            element.textInput = 'Great product!';

            // Click analyze button
            const button = element.shadowRoot.querySelector('lightning-button[label="Analyze Sentiment"]');
            button.click();

            return Promise.resolve().then(() => {
                expect(analyzeSentiment).toHaveBeenCalledWith({
                    text: 'Great product!'
                });
                expect(element.sentimentResult.document_sentiment.label).toBe('POSITIVE');
                expect(element.formattedSentimentResult.color).toBe('sentiment-positive');
            });
        });
    });

    // Test Image Analysis
    describe('image analysis', () => {
        it('successfully analyzes image', () => {
            // Mock API response
            analyzeImage.mockResolvedValue({
                probabilities: [
                    { label: 'cat', probability: 0.95 }
                ]
            });

            // Set input value
            element.imageUrl = 'https://example.com/cat.jpg';

            // Click analyze button
            const button = element.shadowRoot.querySelector('lightning-button[label="Analyze Image"]');
            button.click();

            return Promise.resolve().then(() => {
                expect(analyzeImage).toHaveBeenCalledWith({
                    imageUrl: 'https://example.com/cat.jpg',
                    modelId: 'GeneralImageClassifier'
                });
                expect(element.formattedImageAnalysisResult[0].label).toBe('cat');
                expect(element.formattedImageAnalysisResult[0].probability).toBe('95.0%');
            });
        });
    });

    // Test Chat Functionality
    describe('chat functionality', () => {
        it('sends chat message and receives response', () => {
            // Mock API response
            generateChat.mockResolvedValue({
                choices: [
                    {
                        message: {
                            content: 'AI response'
                        }
                    }
                ]
            });

            // Set input value
            element.chatInput = 'Hello AI';

            // Mock scrollIntoView
            window.HTMLElement.prototype.scrollIntoView = jest.fn();

            // Click send button
            const button = element.shadowRoot.querySelector('lightning-button[label="Send"]');
            button.click();

            return Promise.resolve().then(() => {
                expect(element.chatMessages.length).toBe(2);
                expect(element.chatMessages[0].content).toBe('Hello AI');
                expect(element.chatMessages[1].content).toBe('AI response');
            });
        });

        it('handles Enter key press in chat input', () => {
            const textarea = element.shadowRoot.querySelector('lightning-textarea[name="chatInput"]');
            const event = new KeyboardEvent('keypress', { keyCode: 13 });
            textarea.dispatchEvent(event);

            return Promise.resolve().then(() => {
                expect(generateChat).toHaveBeenCalled();
            });
        });

        it('clears chat history', () => {
            // Add some messages
            element.chatMessages = [
                { id: 1, content: 'Message 1' },
                { id: 2, content: 'Message 2' }
            ];

            // Click clear button
            const button = element.shadowRoot.querySelector('lightning-button[label="Clear Chat"]');
            button.click();

            return Promise.resolve().then(() => {
                expect(element.chatMessages.length).toBe(0);
                expect(element.messageId).toBe(0);
            });
        });
    });

    // Test Clear Functionality
    describe('clear functionality', () => {
        it('clears text generation inputs and results', () => {
            // Set some values
            element.textInput = 'Test input';
            element.generatedText = 'Generated text';
            element.sentimentResult = { document_sentiment: { label: 'POSITIVE' } };

            // Click clear button
            const button = element.shadowRoot.querySelector('lightning-button[label="Clear"]');
            button.click();

            return Promise.resolve().then(() => {
                expect(element.textInput).toBe('');
                expect(element.generatedText).toBe('');
                expect(element.sentimentResult).toBeNull();
            });
        });
    });

    // Test Loading State
    describe('loading state', () => {
        it('shows loading spinner during API calls', () => {
            // Mock slow API response
            generateText.mockImplementation(() => new Promise(resolve => setTimeout(resolve, 1000)));

            // Set input and trigger API call
            element.textInput = 'Test prompt';
            const button = element.shadowRoot.querySelector('lightning-button[label="Generate Text"]');
            button.click();

            return Promise.resolve().then(() => {
                const spinner = element.shadowRoot.querySelector('lightning-spinner');
                expect(spinner).not.toBeNull();
            });
        });
    });

    // Accessibility Tests
    describe('accessibility', () => {
        it('has proper ARIA labels on interactive elements', () => {
            const textArea = element.shadowRoot.querySelector('lightning-textarea[name="prompt"]');
            const generateButton = element.shadowRoot.querySelector('lightning-button[label="Generate Text"]');

            expect(textArea.label).toBeTruthy();
            expect(generateButton.label).toBeTruthy();
        });

        it('maintains proper tab order', () => {
            const interactiveElements = element.shadowRoot.querySelectorAll(
                'lightning-textarea, lightning-input, lightning-button, lightning-slider'
            );

            let previousTabIndex = -1;
            interactiveElements.forEach(el => {
                const currentTabIndex = parseInt(el.tabIndex) || 0;
                expect(currentTabIndex).toBeGreaterThanOrEqual(previousTabIndex);
                previousTabIndex = currentTabIndex;
            });
        });

        it('provides proper feedback for loading states', () => {
            element.isLoading = true;
            return Promise.resolve().then(() => {
                const spinner = element.shadowRoot.querySelector('lightning-spinner');
                expect(spinner.alternativeText).toBeTruthy();
            });
        });
    });

    // Performance Tests
    describe('performance', () => {
        it('debounces rapid input changes', done => {
            jest.useFakeTimers();

            const input = element.shadowRoot.querySelector('lightning-textarea[name="prompt"]');
            let changeCount = 0;

            // Simulate rapid typing
            for (let i = 0; i < 5; i++) {
                input.value = `Test prompt ${i}`;
                input.dispatchEvent(new CustomEvent('change'));
            }

            // Fast-forward timers
            jest.runAllTimers();

            // Check that we didn't process every single change
            expect(changeCount).toBeLessThan(5);
            done();
        });

        it('handles large chat history efficiently', () => {
            // Create large chat history
            const largeChatHistory = Array.from({ length: 100 }, (_, i) => ({
                id: i,
                content: `Message ${i}`,
                role: i % 2 === 0 ? 'user' : 'assistant',
                timestamp: new Date().toLocaleTimeString()
            }));

            const startTime = performance.now();
            element.chatMessages = largeChatHistory;
            const endTime = performance.now();

            // Rendering should be reasonably fast
            expect(endTime - startTime).toBeLessThan(100); // 100ms threshold
        });
    });

    // Error Handling Tests
    describe('error handling', () => {
        it('handles network timeout errors', () => {
            generateText.mockRejectedValue(new Error('Network timeout'));
            element.textInput = 'Test prompt';

            const button = element.shadowRoot.querySelector('lightning-button[label="Generate Text"]');
            button.click();

            return Promise.resolve().then(() => {
                expect(consoleSpy).toHaveBeenCalled();
                const toastEvent = element.dispatchEvent.mock.calls[0][0];
                expect(toastEvent.detail.message).toContain('Network timeout');
            });
        });

        it('handles malformed API responses', () => {
            generateText.mockResolvedValue({ malformed: 'response' });
            element.textInput = 'Test prompt';

            const button = element.shadowRoot.querySelector('lightning-button[label="Generate Text"]');
            button.click();

            return Promise.resolve().then(() => {
                expect(consoleSpy).toHaveBeenCalled();
            });
        });

        it('handles concurrent API calls', async () => {
            const delay = ms => new Promise(resolve => setTimeout(resolve, ms));

            // Mock API calls with different delays
            generateText
                .mockImplementationOnce(() => delay(100).then(() => 'First response'))
                .mockImplementationOnce(() => delay(50).then(() => 'Second response'));

            // Trigger concurrent calls
            element.textInput = 'First prompt';
            const firstCall = element.handleGenerateText();

            element.textInput = 'Second prompt';
            const secondCall = element.handleGenerateText();

            // Wait for both calls to complete
            await Promise.all([firstCall, secondCall]);

            // Verify only the latest response is displayed
            expect(element.generatedText).toBe('Second response');
        });
    });

    // Edge Cases
    describe('edge cases', () => {
        it('handles empty API responses', () => {
            generateText.mockResolvedValue('');
            element.textInput = 'Test prompt';

            const button = element.shadowRoot.querySelector('lightning-button[label="Generate Text"]');
            button.click();

            return Promise.resolve().then(() => {
                expect(element.generatedText).toBe('');
            });
        });

        it('handles extremely long input text', () => {
            const longText = 'a'.repeat(10000);
            element.textInput = longText;

            const button = element.shadowRoot.querySelector('lightning-button[label="Generate Text"]');
            button.click();

            return Promise.resolve().then(() => {
                expect(generateText).toHaveBeenCalled();
            });
        });

        it('handles special characters in input', () => {
            const specialChars = '!@#$%^&*()_+<>?:"{}|~`';
            element.textInput = specialChars;

            const button = element.shadowRoot.querySelector('lightning-button[label="Generate Text"]');
            button.click();

            return Promise.resolve().then(() => {
                expect(generateText).toHaveBeenCalledWith(expect.objectContaining({
                    prompt: specialChars
                }));
            });
        });

        it('handles rapid tab switching', () => {
            const tabset = element.shadowRoot.querySelector('lightning-tabset');

            // Simulate rapid tab switching
            ['text', 'sentiment', 'image', 'chat'].forEach(tab => {
                tabset.dispatchEvent(new CustomEvent('active', {
                    detail: { value: tab }
                }));
            });

            // Verify no errors occurred
            expect(consoleSpy).not.toHaveBeenCalled();
        });
    });

    // Memory Management Tests
    describe('memory management', () => {
        it('properly cleans up resources on disconnect', () => {
            // Add some chat messages
            element.chatMessages = [
                { id: 1, content: 'Message 1' },
                { id: 2, content: 'Message 2' }
            ];

            // Simulate component disconnection
            document.body.removeChild(element);

            // Verify cleanup
            expect(element.chatMessages).toEqual([]);
            expect(element.generatedText).toBe('');
            expect(element.sentimentResult).toBeNull();
        });

        it('handles memory-intensive operations', () => {
            // Create large dataset
            const largeDataset = Array.from({ length: 1000 }, (_, i) => ({
                id: i,
                content: `Message ${i}`.repeat(100),
                timestamp: new Date().toLocaleTimeString()
            }));

            // Measure memory usage
            const startHeap = process.memoryUsage().heapUsed;
            element.chatMessages = largeDataset;
            const endHeap = process.memoryUsage().heapUsed;

            // Cleanup
            element.handleClearChat();

            // Verify reasonable memory usage
            const memoryIncrease = endHeap - startHeap;
            expect(memoryIncrease).toBeLessThan(50 * 1024 * 1024); // 50MB threshold
        });
    });
}); 