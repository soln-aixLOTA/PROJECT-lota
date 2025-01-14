# Usage Tracking at LotaBots

LotaBots tracks API usage to support billing and the tiered pricing model. We track the following metrics:

-   **API Requests:** The number of API requests made by each user.
-   **Message Volume:** The total volume of messages processed by the platform.
-   **Resource Usage:** The amount of CPU, memory, and GPU resources used by each user.

Usage data is stored in a dedicated database and exposed through a monitoring system. We provide an API for integrating with billing systems.

## Tiered Pricing Model

LotaBots offers a tiered pricing model:

-   **Free:** Limited access to the platform with basic features and limited resources.
-   **Professional:** Access to more features and resources, suitable for small to medium-sized businesses.
-   **Enterprise:** Full access to the platform with all features and resources, suitable for large enterprises.

Usage is tracked and billed based on the selected tier. 