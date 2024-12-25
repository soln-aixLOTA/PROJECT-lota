public with sharing class EinsteinAnalyticsService {
    // Analytics API endpoints
    private static final String ANALYTICS_API_URL = '/services/data/v58.0/wave';
    
    /**
     * Creates or updates a dataset in Einstein Analytics
     * @param datasetName Name of the dataset
     * @param datasetLabel User-friendly label for the dataset
     * @param records List of records to upload
     * @return Dataset ID
     */
    public static String upsertDataset(String datasetName, String datasetLabel, List<SObject> records) {
        try {
            // Get dataset metadata
            Map<String, Object> metadata = getDatasetMetadata(records[0].getSObjectType());
            
            // Create or update dataset
            String datasetId = createOrUpdateDataset(datasetName, datasetLabel, metadata);
            
            // Upload data
            uploadDatasetData(datasetId, records);
            
            return datasetId;
        } catch (Exception e) {
            handleError('Dataset Operation Error', e);
            return null;
        }
    }
    
    /**
     * Executes a SAQL query against Einstein Analytics datasets
     * @param query SAQL query string
     * @return Query results
     */
    public static Map<String, Object> executeSAQLQuery(String query) {
        try {
            // Prepare request
            Map<String, String> requestBody = new Map<String, String>{
                'query' => query
            };
            
            // Execute query
            HttpResponse response = makeAnalyticsAPICall(
                ANALYTICS_API_URL + '/query',
                'POST',
                JSON.serialize(requestBody)
            );
            
            // Parse and return results
            return (Map<String, Object>)JSON.deserializeUntyped(response.getBody());
        } catch (Exception e) {
            handleError('SAQL Query Error', e);
            return null;
        }
    }
    
    /**
     * Creates a dashboard programmatically
     * @param dashboardName Name of the dashboard
     * @param dashboardLabel User-friendly label
     * @param widgets List of widget configurations
     * @return Dashboard ID
     */
    public static String createDashboard(String dashboardName, String dashboardLabel, List<Map<String, Object>> widgets) {
        try {
            // Prepare dashboard definition
            Map<String, Object> dashboardDef = new Map<String, Object>{
                'name' => dashboardName,
                'label' => dashboardLabel,
                'folderName' => 'LotaBots Analytics',
                'widgets' => widgets
            };
            
            // Create dashboard
            HttpResponse response = makeAnalyticsAPICall(
                ANALYTICS_API_URL + '/dashboards',
                'POST',
                JSON.serialize(dashboardDef)
            );
            
            // Parse and return dashboard ID
            Map<String, Object> result = (Map<String, Object>)JSON.deserializeUntyped(response.getBody());
            return (String)result.get('id');
        } catch (Exception e) {
            handleError('Dashboard Creation Error', e);
            return null;
        }
    }
    
    /**
     * Gets metadata for a given SObject type
     */
    private static Map<String, Object> getDatasetMetadata(Schema.SObjectType sObjectType) {
        Map<String, Schema.SObjectField> fields = sObjectType.getDescribe().fields.getMap();
        List<Map<String, String>> fieldMetadata = new List<Map<String, String>>();
        
        for (String fieldName : fields.keySet()) {
            Schema.DescribeFieldResult field = fields.get(fieldName).getDescribe();
            fieldMetadata.add(new Map<String, String>{
                'name' => field.getName(),
                'label' => field.getLabel(),
                'type' => String.valueOf(field.getType())
            });
        }
        
        return new Map<String, Object>{
            'objects' => new List<Object>{
                new Map<String, Object>{
                    'name' => sObjectType.getDescribe().getName(),
                    'label' => sObjectType.getDescribe().getLabel(),
                    'fields' => fieldMetadata
                }
            }
        };
    }
    
    /**
     * Creates or updates a dataset definition
     */
    private static String createOrUpdateDataset(String name, String label, Map<String, Object> metadata) {
        Map<String, Object> datasetDef = new Map<String, Object>{
            'name' => name,
            'label' => label,
            'edgemart' => name,
            'metadata' => metadata
        };
        
        HttpResponse response = makeAnalyticsAPICall(
            ANALYTICS_API_URL + '/datasets',
            'POST',
            JSON.serialize(datasetDef)
        );
        
        Map<String, Object> result = (Map<String, Object>)JSON.deserializeUntyped(response.getBody());
        return (String)result.get('id');
    }
    
    /**
     * Uploads data to a dataset
     */
    private static void uploadDatasetData(String datasetId, List<SObject> records) {
        // Convert records to CSV format
        String csv = convertRecordsToCSV(records);
        
        // Upload data
        HttpResponse response = makeAnalyticsAPICall(
            ANALYTICS_API_URL + '/datasets/' + datasetId + '/data',
            'PUT',
            csv
        );
        
        if (response.getStatusCode() != 200) {
            throw new AnalyticsException('Data upload failed: ' + response.getBody());
        }
    }
    
    /**
     * Converts SObject records to CSV format
     */
    private static String convertRecordsToCSV(List<SObject> records) {
        if (records.isEmpty()) return '';
        
        // Get field names
        Set<String> fieldNames = records[0].getPopulatedFieldsAsMap().keySet();
        
        // Build CSV header
        String csv = String.join(new List<String>(fieldNames), ',') + '\n';
        
        // Add record data
        for (SObject record : records) {
            List<String> values = new List<String>();
            for (String field : fieldNames) {
                String value = String.valueOf(record.get(field));
                values.add(value == null ? '' : value.escapeCsv());
            }
            csv += String.join(values, ',') + '\n';
        }
        
        return csv;
    }
    
    /**
     * Makes authenticated API calls to Analytics services
     */
    private static HttpResponse makeAnalyticsAPICall(String endpoint, String method, String body) {
        Http http = new Http();
        HttpRequest request = new HttpRequest();
        request.setEndpoint(URL.getSalesforceBaseUrl().toExternalForm() + endpoint);
        request.setMethod(method);
        request.setHeader('Authorization', 'Bearer ' + UserInfo.getSessionId());
        
        if (method != 'GET') {
            request.setHeader('Content-Type', method == 'PUT' ? 'text/csv' : 'application/json');
            request.setBody(body);
        }
        
        HttpResponse response = http.send(request);
        
        if (response.getStatusCode() == 200) {
            return response;
        } else {
            throw new AnalyticsException('Analytics API call failed: ' + response.getBody());
        }
    }
    
    /**
     * Handles errors in Analytics operations
     */
    private static void handleError(String context, Exception e) {
        // Log error
        System.debug(LoggingLevel.ERROR, context + ': ' + e.getMessage());
        
        // Create error log record
        ErrorLog__c log = new ErrorLog__c(
            Context__c = context,
            ErrorMessage__c = e.getMessage(),
            StackTrace__c = e.getStackTraceString(),
            Timestamp__c = Datetime.now()
        );
        insert log;
        
        // Throw custom exception
        throw new AnalyticsException(context + ': ' + e.getMessage());
    }
    
    /**
     * Custom exception class for Analytics API errors
     */
    public class AnalyticsException extends Exception {}
} 