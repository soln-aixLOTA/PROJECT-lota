public with sharing class EinsteinAIService {
    // Einstein AI API endpoints using Named Credential
    private static final String NAMED_CREDENTIAL = 'callout:EinsteinAI';
    private static final String MODELS_API_URL = NAMED_CREDENTIAL + '/v1/models';
    private static final String EINSTEIN_VISION_URL = NAMED_CREDENTIAL + '/v2/vision';
    private static final String EINSTEIN_LANGUAGE_URL = NAMED_CREDENTIAL + '/v2/language';
    
    // Cache for authentication token
    private static String authToken;
    private static Datetime tokenExpiry;
    
    /**
     * Generates AI-powered responses using Einstein Models API
     * @param prompt The input prompt for text generation
     * @param maxTokens Maximum number of tokens to generate
     * @param temperature Controls randomness (0.0-1.0)
     * @return Generated text response
     */
    public static String generateText(String prompt, Integer maxTokens, Decimal temperature) {
        try {
            // Get settings from custom metadata
            Einstein_Settings__mdt settings = getEinsteinSettings();
            
            // Prepare request body with Trust Layer settings
            Map<String, Object> requestBody = new Map<String, Object>{
                'messages' => new List<Map<String, String>>{
                    new Map<String, String>{
                        'role' => 'system',
                        'content' => 'You are a helpful AI assistant.'
                    },
                    new Map<String, String>{
                        'role' => 'user',
                        'content' => prompt
                    }
                },
                'temperature' => temperature != null ? temperature : settings.Temperature__c,
                'maxTokens' => maxTokens != null ? maxTokens : settings.Max_Tokens__c,
                'model' => settings.Model_ID__c,
                'options' => new Map<String, Object>{
                    'maskSensitiveData' => true,
                    'detectToxicity' => true
                }
            };
            
            // Make API call
            HttpResponse response = makeEinsteinAPICall(
                MODELS_API_URL + '/chat/completions',
                'POST',
                JSON.serialize(requestBody)
            );
            
            // Parse response and check for toxicity
            Map<String, Object> responseData = (Map<String, Object>)JSON.deserializeUntyped(response.getBody());
            Map<String, Object> trustMetrics = (Map<String, Object>)responseData.get('trustMetrics');
            
            if (trustMetrics != null && (Boolean)trustMetrics.get('containsToxicContent')) {
                handleError('Content Safety Error', 
                    new EinsteinException('Generated content contains potentially toxic content'));
                return null;
            }
            
            // Extract generated text from response
            List<Object> choices = (List<Object>)responseData.get('choices');
            if (choices != null && !choices.isEmpty()) {
                Map<String, Object> choice = (Map<String, Object>)choices[0];
                Map<String, Object> message = (Map<String, Object>)choice.get('message');
                return (String)message.get('content');
            }
            
            return null;
        } catch (Exception e) {
            handleError('Text Generation Error', e);
            return null;
        }
    }
    
    /**
     * Generates chat responses using Einstein Models API
     * @param messages List of chat messages with roles and content
     * @param temperature Controls randomness (0.0-1.0)
     * @return Chat completion response
     */
    public static Map<String, Object> generateChat(List<Map<String, String>> messages, Decimal temperature) {
        try {
            // Get settings from custom metadata
            Einstein_Settings__mdt settings = getEinsteinSettings();
            
            // Prepare request body with Trust Layer settings
            Map<String, Object> requestBody = new Map<String, Object>{
                'messages' => messages,
                'temperature' => temperature != null ? temperature : settings.Temperature__c,
                'model' => settings.Model_ID__c,
                'options' => new Map<String, Object>{
                    'maskSensitiveData' => true,
                    'detectToxicity' => true
                }
            };
            
            // Make API call
            HttpResponse response = makeEinsteinAPICall(
                MODELS_API_URL + '/chat/completions',
                'POST',
                JSON.serialize(requestBody)
            );
            
            // Return full response for chat applications
            return (Map<String, Object>)JSON.deserializeUntyped(response.getBody());
        } catch (Exception e) {
            handleError('Chat Generation Error', e);
            return null;
        }
    }
    
    /**
     * Generates embeddings using Einstein Models API
     * @param texts List of texts to generate embeddings for
     * @return Embedding vectors
     */
    public static Map<String, Object> generateEmbeddings(List<String> texts) {
        try {
            // Get settings from custom metadata
            Einstein_Settings__mdt settings = getEinsteinSettings();
            
            // Prepare request body
            Map<String, Object> requestBody = new Map<String, Object>{
                'texts' => texts,
                'model' => settings.Model_ID__c
            };
            
            // Make API call
            HttpResponse response = makeEinsteinAPICall(
                MODELS_API_URL + '/embeddings',
                'POST',
                JSON.serialize(requestBody)
            );
            
            // Return embedding vectors
            return (Map<String, Object>)JSON.deserializeUntyped(response.getBody());
        } catch (Exception e) {
            handleError('Embedding Generation Error', e);
            return null;
        }
    }
    
    /**
     * Analyzes sentiment of text using Einstein Language
     * @param text Text to analyze
     * @return Sentiment analysis result
     */
    public static Map<String, Object> analyzeSentiment(String text) {
        try {
            // Get settings from custom metadata
            Einstein_Settings__mdt settings = getEinsteinSettings();
            
            // Prepare request body
            Map<String, String> requestBody = new Map<String, String>{
                'document' => text,
                'modelId' => settings.Model_ID__c
            };
            
            // Make API call
            HttpResponse response = makeEinsteinAPICall(
                EINSTEIN_LANGUAGE_URL + '/sentiment',
                'POST',
                JSON.serialize(requestBody)
            );
            
            // Return sentiment analysis
            return (Map<String, Object>)JSON.deserializeUntyped(response.getBody());
        } catch (Exception e) {
            handleError('Sentiment Analysis Error', e);
            return null;
        }
    }
    
    /**
     * Processes images using Einstein Vision
     * @param imageUrl URL of the image to process
     * @param modelId Einstein Vision model ID
     * @return Vision analysis result
     */
    public static Map<String, Object> analyzeImage(String imageUrl, String modelId) {
        try {
            // Get settings from custom metadata
            Einstein_Settings__mdt settings = getEinsteinSettings();
            
            // Prepare request body
            Map<String, String> requestBody = new Map<String, String>{
                'sampleLocation' => imageUrl,
                'modelId' => modelId != null ? modelId : settings.Model_ID__c
            };
            
            // Make API call
            HttpResponse response = makeEinsteinAPICall(
                EINSTEIN_VISION_URL + '/predict',
                'POST',
                JSON.serialize(requestBody)
            );
            
            // Return vision analysis
            return (Map<String, Object>)JSON.deserializeUntyped(response.getBody());
        } catch (Exception e) {
            handleError('Image Analysis Error', e);
            return null;
        }
    }
    
    /**
     * Makes authenticated API calls to Einstein services
     */
    private static HttpResponse makeEinsteinAPICall(String endpoint, String method, String body) {
        // Check and refresh auth token if needed
        if (authToken == null || tokenExpiry == null || tokenExpiry < Datetime.now()) {
            refreshAuthToken();
        }
        
        // Prepare HTTP request
        Http http = new Http();
        HttpRequest request = new HttpRequest();
        request.setEndpoint(endpoint);
        request.setMethod(method);
        request.setHeader('Authorization', 'Bearer ' + authToken);
        request.setHeader('Content-Type', 'application/json');
        request.setBody(body);
        
        // Send request
        HttpResponse response = http.send(request);
        
        // Handle response
        if (response.getStatusCode() == 200) {
            return response;
        } else if (response.getStatusCode() == 401) {
            // Token expired, refresh and retry
            refreshAuthToken();
            request.setHeader('Authorization', 'Bearer ' + authToken);
            response = http.send(request);
            
            if (response.getStatusCode() == 200) {
                return response;
            }
        }
        
        throw new EinsteinException('Einstein API call failed: ' + response.getBody());
    }
    
    /**
     * Refreshes the Einstein API authentication token using Named Credentials
     */
    private static void refreshAuthToken() {
        try {
            // Use Named Credentials to get a new token
            Http http = new Http();
            HttpRequest request = new HttpRequest();
            request.setEndpoint(NAMED_CREDENTIAL + '/v2/oauth2/token');
            request.setMethod('GET');
            
            HttpResponse response = http.send(request);
            
            if (response.getStatusCode() == 200) {
                Map<String, Object> tokenData = (Map<String, Object>)JSON.deserializeUntyped(response.getBody());
                authToken = (String)tokenData.get('access_token');
                Integer expiresIn = (Integer)tokenData.get('expires_in');
                tokenExpiry = Datetime.now().addSeconds(expiresIn - 300); // Refresh 5 minutes before expiry
            } else {
                throw new EinsteinException('Failed to refresh token: ' + response.getBody());
            }
        } catch (Exception e) {
            handleError('Token Refresh Error', e);
            throw new EinsteinException('Failed to refresh token: ' + e.getMessage());
        }
    }
    
    /**
     * Gets Einstein settings from custom metadata
     */
    private static Einstein_Settings__mdt getEinsteinSettings() {
        return [SELECT API_Key__c, Model_ID__c, Max_Tokens__c, Temperature__c, 
                Is_Sandbox__c, Error_Email_Notifications__c 
                FROM Einstein_Settings__mdt 
                WHERE DeveloperName = 'Default' 
                LIMIT 1];
    }
    
    /**
     * Handles errors in Einstein API operations with enhanced logging
     */
    private static void handleError(String context, Exception e) {
        // Log error
        System.debug(LoggingLevel.ERROR, context + ': ' + e.getMessage());
        
        // Determine severity based on error type
        String severity = 'Error';
        if (context.contains('Content Safety') || 
            context.contains('Token') || 
            e instanceof Auth.AuthProviderPluginException) {
            severity = 'Critical';
        }
        
        // Create error log record
        ErrorLog__c log = new ErrorLog__c(
            Context__c = context,
            ErrorMessage__c = e.getMessage(),
            StackTrace__c = e.getStackTraceString(),
            Timestamp__c = Datetime.now(),
            Severity__c = severity,
            Source__c = 'Einstein AI'
        );
        insert log;
        
        // Send email notification if configured and critical
        if (severity == 'Critical') {
            Einstein_Settings__mdt settings = getEinsteinSettings();
            if (String.isNotBlank(settings.Error_Email_Notifications__c)) {
                sendErrorNotification(settings.Error_Email_Notifications__c, log);
            }
        }
        
        // Throw custom exception
        throw new EinsteinException(context + ': ' + e.getMessage());
    }
    
    /**
     * Sends error notification emails
     */
    private static void sendErrorNotification(String recipients, ErrorLog__c errorLog) {
        Messaging.SingleEmailMessage email = new Messaging.SingleEmailMessage();
        email.setToAddresses(recipients.split(','));
        email.setSubject('Einstein AI Error: ' + errorLog.Context__c);
        email.setPlainTextBody(
            'Error Details:\n\n' +
            'Context: ' + errorLog.Context__c + '\n' +
            'Message: ' + errorLog.ErrorMessage__c + '\n' +
            'Stack Trace: ' + errorLog.StackTrace__c + '\n' +
            'Timestamp: ' + errorLog.Timestamp__c + '\n' +
            'Severity: ' + errorLog.Severity__c
        );
        
        try {
            Messaging.sendEmail(new Messaging.SingleEmailMessage[]{email});
        } catch (Exception e) {
            System.debug(LoggingLevel.ERROR, 'Failed to send error notification: ' + e.getMessage());
        }
    }
    
    /**
     * Custom exception class for Einstein API errors
     */
    public class EinsteinException extends Exception {}
} 