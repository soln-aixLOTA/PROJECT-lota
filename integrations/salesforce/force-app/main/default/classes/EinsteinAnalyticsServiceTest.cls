@isTest
private class EinsteinAnalyticsServiceTest {
    // Mock HTTP callout class
    private class MockHttpResponse implements HttpCalloutMock {
        private Integer statusCode;
        private String body;
        
        public MockHttpResponse(Integer statusCode, String body) {
            this.statusCode = statusCode;
            this.body = body;
        }
        
        public HTTPResponse respond(HTTPRequest request) {
            HttpResponse response = new HttpResponse();
            response.setStatusCode(statusCode);
            response.setBody(body);
            return response;
        }
    }
    
    @isTest
    static void testUpsertDataset() {
        // Create test account records
        List<Account> testAccounts = new List<Account>();
        for(Integer i = 0; i < 5; i++) {
            testAccounts.add(new Account(
                Name = 'Test Account ' + i,
                Industry = 'Technology',
                AnnualRevenue = 1000000
            ));
        }
        insert testAccounts;
        
        // Setup mock responses
        String mockDatasetResponse = '{"id": "0Fb000000000001"}';
        Test.setMock(HttpCalloutMock.class, new MockHttpResponse(200, mockDatasetResponse));
        
        Test.startTest();
        
        // Test dataset creation
        String datasetId = EinsteinAnalyticsService.upsertDataset(
            'TestDataset',
            'Test Dataset Label',
            testAccounts
        );
        
        // Verify results
        System.assertEquals('0Fb000000000001', datasetId, 'Dataset ID should match mock response');
        
        Test.stopTest();
    }
    
    @isTest
    static void testExecuteSAQLQuery() {
        // Setup mock response
        String mockQueryResponse = '{"results": {"records": [{"count": 5, "amount": 1000000}]}}';
        Test.setMock(HttpCalloutMock.class, new MockHttpResponse(200, mockQueryResponse));
        
        Test.startTest();
        
        // Test SAQL query execution
        String query = 'q = load \"TestDataset\"; q = group q by Industry; q = foreach q generate count() as count, sum(AnnualRevenue) as amount;';
        Map<String, Object> result = EinsteinAnalyticsService.executeSAQLQuery(query);
        
        // Verify results
        Map<String, Object> results = (Map<String, Object>)result.get('results');
        List<Object> records = (List<Object>)results.get('records');
        Map<String, Object> record = (Map<String, Object>)records[0];
        
        System.assertEquals(5, record.get('count'), 'Record count should match mock data');
        System.assertEquals(1000000, record.get('amount'), 'Amount should match mock data');
        
        Test.stopTest();
    }
    
    @isTest
    static void testCreateDashboard() {
        // Setup test dashboard widgets
        List<Map<String, Object>> widgets = new List<Map<String, Object>>{
            new Map<String, Object>{
                'type' => 'chart',
                'name' => 'Revenue by Industry',
                'properties' => new Map<String, Object>{
                    'chartType' => 'bar'
                }
            }
        };
        
        // Setup mock response
        String mockDashboardResponse = '{"id": "0FK000000000001"}';
        Test.setMock(HttpCalloutMock.class, new MockHttpResponse(200, mockDashboardResponse));
        
        Test.startTest();
        
        // Test dashboard creation
        String dashboardId = EinsteinAnalyticsService.createDashboard(
            'TestDashboard',
            'Test Dashboard Label',
            widgets
        );
        
        // Verify results
        System.assertEquals('0FK000000000001', dashboardId, 'Dashboard ID should match mock response');
        
        Test.stopTest();
    }
    
    @isTest
    static void testDatasetMetadata() {
        // Create test account
        Account testAccount = new Account(
            Name = 'Test Account',
            Industry = 'Technology',
            AnnualRevenue = 1000000
        );
        insert testAccount;
        
        // Setup mock response
        String mockResponse = '{"id": "0Fb000000000001"}';
        Test.setMock(HttpCalloutMock.class, new MockHttpResponse(200, mockResponse));
        
        Test.startTest();
        
        // Test metadata generation and dataset creation
        String datasetId = EinsteinAnalyticsService.upsertDataset(
            'TestMetadataDataset',
            'Test Metadata Dataset',
            new List<Account>{testAccount}
        );
        
        // Verify results
        System.assertEquals('0Fb000000000001', datasetId, 'Dataset ID should match mock response');
        
        Test.stopTest();
    }
    
    @isTest
    static void testErrorHandling() {
        // Setup error response
        Test.setMock(HttpCalloutMock.class, new MockHttpResponse(400, 'Invalid request'));
        
        Test.startTest();
        
        try {
            // Test error handling
            EinsteinAnalyticsService.executeSAQLQuery('invalid query');
            System.assert(false, 'Should have thrown an exception');
        } catch (EinsteinAnalyticsService.AnalyticsException e) {
            // Verify error handling
            System.assert(e.getMessage().contains('SAQL Query Error'), 
                'Exception message should contain error context');
            
            // Verify error logging
            List<ErrorLog__c> logs = [SELECT Context__c, ErrorMessage__c 
                                    FROM ErrorLog__c 
                                    WHERE Context__c = 'SAQL Query Error'];
            System.assertEquals(1, logs.size(), 'Should have created one error log');
            System.assert(logs[0].ErrorMessage__c.contains('Invalid request'), 
                'Error log should contain error message');
        }
        
        Test.stopTest();
    }
    
    @isTest
    static void testCSVConversion() {
        // Create test accounts with various data types
        List<Account> testAccounts = new List<Account>{
            new Account(
                Name = 'Test Account 1',
                Industry = 'Technology',
                AnnualRevenue = 1000000,
                Description = 'Test, with comma'
            ),
            new Account(
                Name = 'Test Account 2',
                Industry = 'Healthcare',
                AnnualRevenue = 2000000,
                Description = 'Test\nwith\nnewline'
            )
        };
        insert testAccounts;
        
        // Setup mock response
        String mockResponse = '{"id": "0Fb000000000001"}';
        Test.setMock(HttpCalloutMock.class, new MockHttpResponse(200, mockResponse));
        
        Test.startTest();
        
        // Test dataset creation with CSV conversion
        String datasetId = EinsteinAnalyticsService.upsertDataset(
            'TestCSVDataset',
            'Test CSV Dataset',
            testAccounts
        );
        
        // Verify results
        System.assertEquals('0Fb000000000001', datasetId, 'Dataset ID should match mock response');
        
        Test.stopTest();
    }
    
    @isTest
    static void testLargeDatasetHandling() {
        // Create large number of test records
        List<Account> testAccounts = new List<Account>();
        for(Integer i = 0; i < 100; i++) {
            testAccounts.add(new Account(
                Name = 'Test Account ' + i,
                Industry = 'Technology',
                AnnualRevenue = 1000000 * i
            ));
        }
        insert testAccounts;
        
        // Setup mock response
        String mockResponse = '{"id": "0Fb000000000001"}';
        Test.setMock(HttpCalloutMock.class, new MockHttpResponse(200, mockResponse));
        
        Test.startTest();
        
        // Test large dataset handling
        String datasetId = EinsteinAnalyticsService.upsertDataset(
            'TestLargeDataset',
            'Test Large Dataset',
            testAccounts
        );
        
        // Verify results
        System.assertEquals('0Fb000000000001', datasetId, 'Dataset ID should match mock response');
        
        Test.stopTest();
    }
} 