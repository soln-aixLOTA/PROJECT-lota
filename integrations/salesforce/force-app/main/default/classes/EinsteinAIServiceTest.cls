@isTest
private class EinsteinAIServiceTest {
    // Mock HTTP callout class
    private class MockHttpResponse implements HttpCalloutMock {
        private Integer statusCode;
        private String body;
        private Integer callCount = 0;
        private List<String> responses;
        
        public MockHttpResponse(Integer statusCode, String body) {
            this.statusCode = statusCode;
            this.body = body;
        }
        
        public MockHttpResponse(List<String> responses) {
            this.responses = responses;
            this.statusCode = 200;
        }
        
        public HTTPResponse respond(HTTPRequest request) {
            HttpResponse response = new HttpResponse();
            
            if (responses != null && callCount < responses.size()) {
                response.setBody(responses[callCount++]);
                response.setStatusCode(200);
            } else {
                response.setStatusCode(statusCode);
                response.setBody(body);
            }
            
            return response;
        }
    }
    
    @TestSetup
    static void setupTestData() {
        // Create test error log
        ErrorLog__c log = new ErrorLog__c(
            Context__c = 'Test Error',
            ErrorMessage__c = 'Test error message',
            StackTrace__c = 'Test stack trace',
            Timestamp__c = Datetime.now(),
            Severity__c = 'Error',
            Source__c = 'Einstein AI'
        );
        insert log;
    }
    
    @isTest
    static void testGenerateText() {
        // Setup test data with trust metrics
        String mockResponse = JSON.serialize(new Map<String, Object>{
            'choices' => new List<Map<String, Object>>{
                new Map<String, Object>{
                    'message' => new Map<String, Object>{
                        'content' => 'This is a test response'
                    }
                }
            },
            'trustMetrics' => new Map<String, Object>{
                'containsToxicContent' => false,
                'sensitiveDataDetected' => false
            }
        });
        Test.setMock(HttpCalloutMock.class, new MockHttpResponse(200, mockResponse));
        
        Test.startTest();
        
        // Test successful text generation
        String result = EinsteinAIService.generateText(
            'Test prompt',
            100,
            0.7
        );
        
        // Verify results
        System.assertEquals('This is a test response', result, 'Generated text should match mock response');
        
        Test.stopTest();
    }
    
    @isTest
    static void testGenerateTextWithToxicContent() {
        // Setup test data with toxic content
        String mockResponse = JSON.serialize(new Map<String, Object>{
            'choices' => new List<Map<String, Object>>{
                new Map<String, Object>{
                    'message' => new Map<String, Object>{
                        'content' => 'This content would be toxic'
                    }
                }
            },
            'trustMetrics' => new Map<String, Object>{
                'containsToxicContent' => true,
                'sensitiveDataDetected' => false
            }
        });
        Test.setMock(HttpCalloutMock.class, new MockHttpResponse(200, mockResponse));
        
        Test.startTest();
        
        try {
            // Test toxic content handling
            String result = EinsteinAIService.generateText('Test prompt', 100, 0.7);
            System.assert(false, 'Should have thrown an exception for toxic content');
        } catch (EinsteinAIService.EinsteinException e) {
            // Verify error handling
            System.assert(e.getMessage().contains('Content Safety Error'), 
                'Exception should indicate content safety violation');
            
            // Verify error logging
            List<ErrorLog__c> logs = [SELECT Context__c, Severity__c 
                                    FROM ErrorLog__c 
                                    WHERE Context__c = 'Content Safety Error'];
            System.assertEquals(1, logs.size(), 'Should have created one error log');
            System.assertEquals('Critical', logs[0].Severity__c, 
                'Content safety violations should be logged as Critical');
        }
        
        Test.stopTest();
    }
    
    @isTest
    static void testGenerateChat() {
        // Setup test data
        String mockResponse = JSON.serialize(new Map<String, Object>{
            'choices' => new List<Map<String, Object>>{
                new Map<String, Object>{
                    'message' => new Map<String, Object>{
                        'content' => 'Chat response',
                        'role' => 'assistant'
                    }
                }
            },
            'trustMetrics' => new Map<String, Object>{
                'containsToxicContent' => false,
                'sensitiveDataDetected' => false
            }
        });
        Test.setMock(HttpCalloutMock.class, new MockHttpResponse(200, mockResponse));
        
        Test.startTest();
        
        // Test chat generation
        List<Map<String, String>> messages = new List<Map<String, String>>{
            new Map<String, String>{
                'role' => 'user',
                'content' => 'Hello'
            }
        };
        
        Map<String, Object> result = EinsteinAIService.generateChat(messages, 0.7);
        
        // Verify results
        System.assertNotEquals(null, result, 'Should have received a response');
        List<Object> choices = (List<Object>)result.get('choices');
        System.assertNotEquals(null, choices, 'Response should contain choices');
        System.assertEquals(1, choices.size(), 'Should have one choice');
        
        Test.stopTest();
    }
    
    @isTest
    static void testGenerateEmbeddings() {
        // Setup test data
        String mockResponse = JSON.serialize(new Map<String, Object>{
            'embeddings' => new List<List<Double>>{
                new List<Double>{0.1, 0.2, 0.3}
            }
        });
        Test.setMock(HttpCalloutMock.class, new MockHttpResponse(200, mockResponse));
        
        Test.startTest();
        
        // Test embedding generation
        List<String> texts = new List<String>{'Test text'};
        Map<String, Object> result = EinsteinAIService.generateEmbeddings(texts);
        
        // Verify results
        System.assertNotEquals(null, result, 'Should have received embeddings');
        List<Object> embeddings = (List<Object>)result.get('embeddings');
        System.assertNotEquals(null, embeddings, 'Response should contain embeddings');
        System.assertEquals(1, embeddings.size(), 'Should have one embedding vector');
        
        Test.stopTest();
    }
    
    @isTest
    static void testAnalyzeSentiment() {
        // Setup test data
        String mockResponse = '{"document_sentiment": {"label": "POSITIVE", "score": 0.8}}';
        Test.setMock(HttpCalloutMock.class, new MockHttpResponse(200, mockResponse));
        
        Test.startTest();
        
        // Test sentiment analysis
        Map<String, Object> result = EinsteinAIService.analyzeSentiment('Great product!');
        
        // Verify results
        Map<String, Object> sentiment = (Map<String, Object>)result.get('document_sentiment');
        System.assertEquals('POSITIVE', sentiment.get('label'), 'Sentiment label should be POSITIVE');
        System.assertEquals(0.8, sentiment.get('score'), 'Sentiment score should be 0.8');
        
        Test.stopTest();
    }
    
    @isTest
    static void testAnalyzeImage() {
        // Setup test data
        String mockResponse = '{"probabilities": [{"label": "cat", "probability": 0.95}]}';
        Test.setMock(HttpCalloutMock.class, new MockHttpResponse(200, mockResponse));
        
        Test.startTest();
        
        // Test image analysis
        Map<String, Object> result = EinsteinAIService.analyzeImage(
            'https://example.com/image.jpg',
            'GeneralImageClassifier'
        );
        
        // Verify results
        List<Object> probabilities = (List<Object>)result.get('probabilities');
        Map<String, Object> prediction = (Map<String, Object>)probabilities[0];
        System.assertEquals('cat', prediction.get('label'), 'Image label should be cat');
        System.assertEquals(0.95, prediction.get('probability'), 'Probability should be 0.95');
        
        Test.stopTest();
    }
    
    @isTest
    static void testErrorHandling() {
        // Setup test data
        Test.setMock(HttpCalloutMock.class, new MockHttpResponse(400, 'Invalid request'));
        
        Test.startTest();
        
        try {
            // Test error handling
            EinsteinAIService.generateText('Test prompt', 100, 0.7);
            System.assert(false, 'Should have thrown an exception');
        } catch (EinsteinAIService.EinsteinException e) {
            // Verify error handling
            System.assert(e.getMessage().contains('Text Generation Error'), 
                'Exception message should contain error context');
            
            // Verify error logging
            List<ErrorLog__c> logs = [SELECT Context__c, ErrorMessage__c, Severity__c, Source__c 
                                    FROM ErrorLog__c 
                                    WHERE Context__c = 'Text Generation Error'];
            System.assertEquals(1, logs.size(), 'Should have created one error log');
            System.assert(logs[0].ErrorMessage__c.contains('Invalid request'), 
                'Error log should contain error message');
            System.assertEquals('Error', logs[0].Severity__c, 'Error severity should be Error');
            System.assertEquals('Einstein AI', logs[0].Source__c, 'Error source should be Einstein AI');
        }
        
        Test.stopTest();
    }
    
    @isTest
    static void testTokenRefreshAndRetry() {
        // Setup test data with multiple responses
        List<String> responses = new List<String>{
            '{"access_token": "new_token", "expires_in": 3600}',  // Initial token response
            '{"error": "Token expired"}',                         // First API call fails
            '{"access_token": "newer_token", "expires_in": 3600}', // Token refresh response
            '{"generatedText": "Test response"}'                  // Retry API call succeeds
        };
        
        // Create and set mock that returns different responses
        Test.setMock(HttpCalloutMock.class, new MockHttpResponse(responses));
        
        Test.startTest();
        
        // Test API call with token refresh and retry
        String result = EinsteinAIService.generateText('Test prompt', 100, 0.7);
        
        // Verify results
        System.assertEquals('Test response', result, 'Should have received response after token refresh and retry');
        
        Test.stopTest();
    }
    
    @isTest
    static void testErrorNotifications() {
        // Enable email notifications in settings
        Test.setMock(HttpCalloutMock.class, new MockHttpResponse(500, 'Internal server error'));
        
        Test.startTest();
        
        try {
            EinsteinAIService.generateText('Test prompt', 100, 0.7);
            System.assert(false, 'Should have thrown an exception');
        } catch (EinsteinAIService.EinsteinException e) {
            // Verify error notification
            Integer emailInvocations = Limits.getEmailInvocations();
            System.assert(emailInvocations > 0, 'Should have sent error notification email');
            
            // Verify error log
            List<ErrorLog__c> logs = [SELECT Context__c, Severity__c, Source__c 
                                    FROM ErrorLog__c 
                                    WHERE Context__c = 'Text Generation Error' 
                                    AND Severity__c = 'Error'];
            System.assertEquals(1, logs.size(), 'Should have created error log with correct severity');
        }
        
        Test.stopTest();
    }
    
    @isTest
    static void testDefaultSettings() {
        // Setup test data
        String mockResponse = '{"generatedText": "Test response"}';
        Test.setMock(HttpCalloutMock.class, new MockHttpResponse(200, mockResponse));
        
        Test.startTest();
        
        // Test API call without specifying optional parameters
        String result = EinsteinAIService.generateText('Test prompt', null, null);
        
        // Verify results
        System.assertEquals('Test response', result, 'Should have used default settings for missing parameters');
        
        Test.stopTest();
    }
    
    @isTest
    static void testBatchProcessing() {
        // Setup test data
        List<String> prompts = new List<String>{
            'Prompt 1',
            'Prompt 2',
            'Prompt 3'
        };
        
        String mockResponse = '{"generatedText": "Response"}';
        Test.setMock(HttpCalloutMock.class, new MockHttpResponse(200, mockResponse));
        
        Test.startTest();
        
        // Test batch processing of multiple prompts
        List<String> results = new List<String>();
        for (String prompt : prompts) {
            results.add(EinsteinAIService.generateText(prompt, 100, 0.7));
        }
        
        // Verify results
        System.assertEquals(prompts.size(), results.size(), 
            'Should have processed all prompts');
        for (String result : results) {
            System.assertEquals('Response', result, 
                'Each result should match mock response');
        }
        
        Test.stopTest();
    }
    
    @isTest
    static void testErrorSeverityLevels() {
        Test.setMock(HttpCalloutMock.class, new MockHttpResponse(500, 'Internal server error'));
        
        Test.startTest();
        
        // Test different error scenarios
        try {
            EinsteinAIService.generateText('Test prompt', 100, 0.7);
            System.assert(false, 'Should have thrown an exception');
        } catch (EinsteinAIService.EinsteinException e) {
            // Verify error severity for regular errors
            List<ErrorLog__c> regularLogs = [SELECT Severity__c 
                                           FROM ErrorLog__c 
                                           WHERE Context__c = 'Text Generation Error'];
            System.assertEquals('Error', regularLogs[0].Severity__c, 
                'Regular errors should have Error severity');
        }
        
        // Test token refresh error (should be Critical)
        try {
            EinsteinAIService.refreshAuthToken();
            System.assert(false, 'Should have thrown an exception');
        } catch (EinsteinAIService.EinsteinException e) {
            // Verify error severity for token errors
            List<ErrorLog__c> tokenLogs = [SELECT Severity__c 
                                         FROM ErrorLog__c 
                                         WHERE Context__c = 'Token Refresh Error'];
            System.assertEquals('Critical', tokenLogs[0].Severity__c, 
                'Token errors should have Critical severity');
        }
        
        Test.stopTest();
    }
} 