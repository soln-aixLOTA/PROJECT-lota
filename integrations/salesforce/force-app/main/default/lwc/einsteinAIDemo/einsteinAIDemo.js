import analyzeImage from '@salesforce/apex/EinsteinAIService.analyzeImage';
import analyzeSentiment from '@salesforce/apex/EinsteinAIService.analyzeSentiment';
import generateChat from '@salesforce/apex/EinsteinAIService.generateChat';
import generateText from '@salesforce/apex/EinsteinAIService.generateText';
import { ShowToastEvent } from 'lightning/platformShowToastEvent';
import { api, LightningElement, track } from 'lwc';

export default class EinsteinAIDemo extends LightningElement {
    // Configuration properties
    @api maxTokensDefault = 100;
    @api temperatureDefault = 0.7;
    @api modelId = 'GeneralImageClassifier';

    // Input values
    @track textInput = '';
    @track imageUrl = '';
    @track chatInput = '';
    @track maxTokens = this.maxTokensDefault;
    @track temperature = this.temperatureDefault;

    // Results
    @track generatedText = '';
    @track sentimentResult = null;
    @track imageAnalysisResult = null;
    @track chatMessages = [];

    // UI state
    @track isLoading = false;
    @track activeTab = 'text';

    // Unique identifier for chat messages
    messageId = 0;

    // Handle text input change
    handleTextInputChange(event) {
        this.textInput = event.target.value;
    }

    // Handle image URL input change
    handleImageUrlChange(event) {
        this.imageUrl = event.target.value;
    }

    // Handle chat input change
    handleChatInputChange(event) {
        this.chatInput = event.target.value;
    }

    // Handle max tokens slider change
    handleMaxTokensChange(event) {
        this.maxTokens = event.detail.value;
    }

    // Handle temperature slider change
    handleTemperatureChange(event) {
        this.temperature = event.detail.value;
    }

    // Handle chat input key press (Enter to send)
    handleChatKeyPress(event) {
        if (event.keyCode === 13 && !event.shiftKey) {
            event.preventDefault();
            this.handleSendChat();
        }
    }

    // Generate text using Einstein AI
    async handleGenerateText() {
        if (!this.textInput) {
            this.showToast('Error', 'Please enter a prompt', 'error');
            return;
        }

        this.isLoading = true;
        try {
            this.generatedText = await generateText({
                prompt: this.textInput,
                maxTokens: this.maxTokens,
                temperature: this.temperature
            });
            if (this.generatedText) {
                this.showToast('Success', 'Text generated successfully', 'success');
            }
        } catch (error) {
            this.handleError('Text Generation Error', error);
        } finally {
            this.isLoading = false;
        }
    }

    // Analyze sentiment using Einstein AI
    async handleAnalyzeSentiment() {
        if (!this.textInput) {
            this.showToast('Error', 'Please enter text to analyze', 'error');
            return;
        }

        this.isLoading = true;
        try {
            this.sentimentResult = await analyzeSentiment({
                text: this.textInput
            });
            if (this.sentimentResult) {
                this.showToast('Success', 'Sentiment analyzed successfully', 'success');
            }
        } catch (error) {
            this.handleError('Sentiment Analysis Error', error);
        } finally {
            this.isLoading = false;
        }
    }

    // Analyze image using Einstein Vision
    async handleAnalyzeImage() {
        if (!this.imageUrl) {
            this.showToast('Error', 'Please enter an image URL', 'error');
            return;
        }

        this.isLoading = true;
        try {
            this.imageAnalysisResult = await analyzeImage({
                imageUrl: this.imageUrl,
                modelId: this.modelId
            });
            if (this.imageAnalysisResult) {
                this.showToast('Success', 'Image analyzed successfully', 'success');
            }
        } catch (error) {
            this.handleError('Image Analysis Error', error);
        } finally {
            this.isLoading = false;
        }
    }

    // Send chat message
    async handleSendChat() {
        if (!this.chatInput.trim()) {
            return;
        }

        // Add user message
        this.addChatMessage('user', this.chatInput);
        const userMessage = this.chatInput;
        this.chatInput = '';

        this.isLoading = true;
        try {
            // Prepare messages array for API call
            const messages = this.chatMessages.map(msg => ({
                role: msg.role,
                content: msg.content
            }));

            // Get AI response
            const response = await generateChat({
                messages: messages,
                temperature: this.temperature
            });

            // Add assistant message
            if (response && response.choices && response.choices.length > 0) {
                const assistantMessage = response.choices[0].message.content;
                this.addChatMessage('assistant', assistantMessage);
            }
        } catch (error) {
            this.handleError('Chat Error', error);
        } finally {
            this.isLoading = false;
        }
    }

    // Add message to chat
    addChatMessage(role, content) {
        const message = {
            id: this.messageId++,
            role: role,
            content: content,
            timestamp: new Date().toLocaleTimeString(),
            containerClass: `chat-message-container ${role} fade-in`,
            messageClass: `chat-message ${role}`
        };
        this.chatMessages = [...this.chatMessages, message];

        // Scroll to bottom
        setTimeout(() => {
            const container = this.template.querySelector('.chat-container');
            if (container) {
                container.scrollTop = container.scrollHeight;
            }
        }, 0);
    }

    // Clear chat history
    handleClearChat() {
        this.chatMessages = [];
        this.messageId = 0;
    }

    // Clear current input and results
    handleClear() {
        this.textInput = '';
        this.generatedText = '';
        this.sentimentResult = null;
    }

    // Show toast notification
    showToast(title, message, variant) {
        this.dispatchEvent(
            new ShowToastEvent({
                title: title,
                message: message,
                variant: variant,
            })
        );
    }

    // Handle errors
    handleError(context, error) {
        console.error(`${context}:`, error);
        this.showToast(
            'Error',
            error.body?.message || error.message || 'An unexpected error occurred',
            'error'
        );
    }

    // Computed property for formatted sentiment result
    get formattedSentimentResult() {
        if (!this.sentimentResult) return null;
        const sentiment = this.sentimentResult.document_sentiment;
        return {
            label: sentiment.label,
            score: (sentiment.score * 100).toFixed(1) + '%',
            color: this.getSentimentColor(sentiment.label)
        };
    }

    // Computed property for formatted image analysis result
    get formattedImageAnalysisResult() {
        if (!this.imageAnalysisResult) return null;
        return this.imageAnalysisResult.probabilities.map(prob => ({
            label: prob.label,
            probability: (prob.probability * 100).toFixed(1) + '%'
        }));
    }

    // Helper method to get sentiment color class
    getSentimentColor(sentiment) {
        const colors = {
            'POSITIVE': 'sentiment-positive',
            'NEGATIVE': 'sentiment-negative',
            'NEUTRAL': 'sentiment-neutral'
        };
        return colors[sentiment] || '';
    }
} 