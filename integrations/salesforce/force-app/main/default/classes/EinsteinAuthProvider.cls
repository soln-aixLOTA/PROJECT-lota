public class EinsteinAuthProvider implements Auth.AuthProviderPluginClass {
    private static final String EINSTEIN_SETTINGS = 'Default';
    
    public Auth.AuthProviderTokenResponse handleCallback(Map<String, String> callbackParams, Auth.AuthProviderCallbackState state) {
        // Not used for password auth
        return null;
    }
    
    public Auth.UserData getUserInfo(Map<String, String> authProviderConfiguration, Auth.AuthProviderTokenResponse response) {
        // Not used for password auth
        return null;
    }
    
    public String getCustomMetadataType() {
        return 'Einstein_Settings__mdt';
    }
    
    public Auth.OAuthRefreshResult refresh(Map<String, String> authProviderConfiguration, String refreshToken) {
        try {
            // Get settings from custom metadata
            Einstein_Settings__mdt settings = getEinsteinSettings();
            
            // Generate JWT token
            String jwtToken = generateJWT(settings.API_Key__c);
            
            // Exchange JWT for access token
            String accessToken = exchangeJWTForToken(jwtToken);
            
            return new Auth.OAuthRefreshResult(accessToken, refreshToken);
        } catch (Exception e) {
            handleError('Auth Refresh Error', e);
            return null;
        }
    }
    
    private static Einstein_Settings__mdt getEinsteinSettings() {
        return [SELECT API_Key__c, Is_Sandbox__c, Error_Email_Notifications__c 
                FROM Einstein_Settings__mdt 
                WHERE DeveloperName = :EINSTEIN_SETTINGS 
                LIMIT 1];
    }
    
    private static String generateJWT(String privateKey) {
        // Create JWT header
        Map<String, Object> header = new Map<String, Object>{
            'alg' => 'RS256',
            'typ' => 'JWT'
        };
        
        // Create JWT claims
        Map<String, Object> claims = new Map<String, Object>{
            'sub' => UserInfo.getUserEmail(),
            'aud' => 'https://api.einstein.ai/v2/oauth2/token',
            'exp' => DateTime.now().addMinutes(30).getTime() / 1000,
            'iat' => DateTime.now().getTime() / 1000
        };
        
        // Encode JWT parts
        String encodedHeader = base64URLencode(JSON.serialize(header));
        String encodedClaims = base64URLencode(JSON.serialize(claims));
        
        // Create signature
        String signatureInput = encodedHeader + '.' + encodedClaims;
        Blob signature = Crypto.sign('RSA-SHA256', Blob.valueOf(signatureInput), EncodingUtil.base64Decode(privateKey));
        String encodedSignature = base64URLencode(signature);
        
        // Return complete JWT
        return encodedHeader + '.' + encodedClaims + '.' + encodedSignature;
    }
    
    private static String exchangeJWTForToken(String jwt) {
        Http http = new Http();
        HttpRequest request = new HttpRequest();
        request.setEndpoint('https://api.einstein.ai/v2/oauth2/token');
        request.setMethod('POST');
        request.setHeader('Content-Type', 'application/x-www-form-urlencoded');
        request.setBody('grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer&assertion=' + jwt);
        
        HttpResponse response = http.send(request);
        
        if (response.getStatusCode() == 200) {
            Map<String, Object> result = (Map<String, Object>)JSON.deserializeUntyped(response.getBody());
            return (String)result.get('access_token');
        } else {
            throw new Auth.AuthProviderPluginException('Failed to exchange JWT: ' + response.getBody());
        }
    }
    
    private static String base64URLencode(Blob input) {
        String output = EncodingUtil.base64Encode(input);
        output = output.replace('+', '-');
        output = output.replace('/', '_');
        while (output.endsWith('=')) {
            output = output.substring(0, output.length() - 1);
        }
        return output;
    }
    
    private static void handleError(String context, Exception e) {
        // Log error
        System.debug(LoggingLevel.ERROR, context + ': ' + e.getMessage());
        
        // Create error log record
        ErrorLog__c log = new ErrorLog__c(
            Context__c = context,
            ErrorMessage__c = e.getMessage(),
            StackTrace__c = e.getStackTraceString(),
            Timestamp__c = Datetime.now(),
            Severity__c = 'Critical',
            Source__c = 'Einstein AI'
        );
        insert log;
        
        // Send email notification if configured
        Einstein_Settings__mdt settings = getEinsteinSettings();
        if (String.isNotBlank(settings.Error_Email_Notifications__c)) {
            sendErrorNotification(settings.Error_Email_Notifications__c, log);
        }
    }
    
    private static void sendErrorNotification(String recipients, ErrorLog__c errorLog) {
        Messaging.SingleEmailMessage email = new Messaging.SingleEmailMessage();
        email.setToAddresses(recipients.split(','));
        email.setSubject('Einstein AI Critical Error: ' + errorLog.Context__c);
        email.setPlainTextBody(
            'Error Details:\n\n' +
            'Context: ' + errorLog.Context__c + '\n' +
            'Message: ' + errorLog.ErrorMessage__c + '\n' +
            'Stack Trace: ' + errorLog.StackTrace__c + '\n' +
            'Timestamp: ' + errorLog.Timestamp__c
        );
        
        try {
            Messaging.sendEmail(new Messaging.SingleEmailMessage[]{email});
        } catch (Exception e) {
            System.debug(LoggingLevel.ERROR, 'Failed to send error notification: ' + e.getMessage());
        }
    }
} 