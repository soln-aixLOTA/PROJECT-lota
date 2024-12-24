# GenAI Web Application

A Next.js web application with Google's Generative AI integration and OpenTelemetry instrumentation.

## Features

- Google GenAI integration with the Gemini model
- OpenTelemetry instrumentation for monitoring and tracing
- React components for AI interaction
- TypeScript support
- Tailwind CSS styling

## Prerequisites

- Node.js 16.x or later
- npm 7.x or later
- A Google GenAI API key

## Setup

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd <repository-name>
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Create a `.env.local` file in the root directory:
   ```bash
   NEXT_PUBLIC_GENAI_API_KEY=your_api_key_here
   ```

4. Start the development server:
   ```bash
   npm run dev
   ```

5. Open [http://localhost:3000/genai](http://localhost:3000/genai) in your browser.

## OpenTelemetry Integration

The application includes OpenTelemetry instrumentation for:

- Tracing GenAI API calls
- Monitoring token usage
- Tracking latency and errors
- Recording operation durations

Telemetry data is currently exported to the console. In production, you should configure appropriate exporters (e.g., Jaeger, Zipkin).

## Project Structure

```
├── app/                    # Next.js app directory
│   └── genai/             # GenAI demo page
├── src/
│   ├── components/        # React components
│   └── lib/
│       ├── genai/        # GenAI client and context
│       └── telemetry/    # OpenTelemetry configuration
├── .env.local             # Environment variables
└── tsconfig.json          # TypeScript configuration
```

## Usage

1. Navigate to `/genai` in your browser
2. Enter a prompt in the text area
3. Click "Generate" to get a response
4. View telemetry data in the browser console

## Development

- Run tests: `npm test`
- Build for production: `npm run build`
- Start production server: `npm start`

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details. # PROJECT-lota
