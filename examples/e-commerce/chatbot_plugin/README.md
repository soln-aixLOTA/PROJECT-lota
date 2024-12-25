# E-commerce Chatbot Plugin Example

This example demonstrates how to use the LotaBots plugin system to create a customer service chatbot for e-commerce applications. The chatbot uses a pre-trained language model and product catalog data to provide helpful responses to customer inquiries.

## Features

- Natural language understanding and generation
- Product catalog integration
- Context-aware responses
- Conversation history management
- GPU acceleration support
- Configurable model parameters

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
- `max_length`: Maximum length of generated responses
- `temperature`: Sampling temperature for response generation
- `product_catalog_path`: Path to the product catalog data

## Usage

1. Make sure you have a product catalog file (`sample_catalog.json` is provided)

2. Run the test script:
   ```bash
   python test_plugin.py
   ```

3. Interact with the chatbot by typing messages. Example queries:
   - "What laptops do you have?"
   - "Tell me about your smartphones"
   - "Do you have any noise-canceling headphones?"
   - "What's the battery life of the FitWatch X?"

4. Type 'quit' to exit the chat

## Example Conversation

```
User: What laptops do you have?
Assistant: We have the ProBook X1, a high-performance laptop with impressive specifications:
- Intel Core i7 processor
- 16GB DDR4 RAM
- 512GB NVMe SSD
- NVIDIA RTX 3060 GPU
- 15.6-inch 4K display
- 8 hours battery life

It's currently in stock and priced at $999.99.

User: Does it come with a warranty?
Assistant: Yes, all our laptops, including the ProBook X1, come with a standard manufacturer's warranty. Would you like me to provide more details about the warranty coverage?
```

## Customization

You can customize the plugin by:

1. Modifying the product catalog data
2. Adjusting the model parameters in the configuration
3. Implementing custom preprocessing logic
4. Adding new features to the plugin implementation

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