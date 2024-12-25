# Fraud Detection Plugin Example

This example demonstrates how to use the LotaBots plugin system to detect fraudulent transactions in financial services. The plugin uses a pre-trained model to analyze transaction data and identify potential fraud.

## Features

- Fraud detection using machine learning
- Configurable detection threshold
- GPU acceleration support

## Prerequisites

- Python 3.8 or higher
- NVIDIA GPU with CUDA support (recommended)
- LotaBots platform installed

## Installation

1. Create a virtual environment:
   ```bash
   python -m venv venv
   source venv/bin/activate  # Linux/Mac
   # or
   .\venv\Scripts\activate  # Windows
   ```

2. Install dependencies:
   ```bash
   pip install -r requirements.txt
   ```

## Configuration

The plugin can be configured through the `metadata.json` file. Key configuration options include:

- `model_name`: Name of the pre-trained model to use
- `threshold`: Threshold for classifying a transaction as fraudulent

## Usage

1. Run the test script:
   ```bash
   python test_plugin.py
   ```

2. Enter transaction data to analyze. Example inputs:
   - "Transaction of $1000 from account A to account B"
   - "Payment of $500 for invoice #12345"

3. Type 'quit' to exit the test

## Example Output

```
Transaction: Transaction of $1000 from account A to account B
Fraud Detected:
Confidence: 0.85

Transaction: Payment of $500 for invoice #12345
No Fraud Detected
Confidence: 0.15
```

## Customization

You can customize the plugin by:

1. Adjusting the detection threshold in the configuration
2. Using a different pre-trained model

## Integration

To integrate this plugin into your LotaBots application:

1. Copy the plugin directory to your plugins folder
2. Configure the plugin through your application's configuration
3. Use the plugin manager to load and initialize the plugin
4. Send requests through the plugin's execute method

## Contributing

Feel free to contribute to this example by:

1. Adding new features
2. Improving the documentation
3. Fixing bugs
4. Suggesting enhancements

## License

This example is part of the LotaBots platform and is subject to the same license terms. 