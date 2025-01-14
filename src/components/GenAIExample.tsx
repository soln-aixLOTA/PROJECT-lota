import { useGenAI } from '@/lib/genai/GenAIContext';
import React, { useState } from 'react';

export function GenAIExample() {
    const { client, isInitialized, error } = useGenAI();
    const [prompt, setPrompt] = useState('');
    const [response, setResponse] = useState('');
    const [isLoading, setIsLoading] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!client || !prompt.trim()) return;

        setIsLoading(true);
        try {
            const result = await client.generateContent(prompt);
            setResponse(result.text);
        } catch (err) {
            console.error('Error generating content:', err);
            setResponse('Error generating content. Please try again.');
        } finally {
            setIsLoading(false);
        }
    };

    if (!isInitialized) {
        return <div>Initializing GenAI client...</div>;
    }

    if (error) {
        return <div>Error: {error.message}</div>;
    }

    if (!client) {
        return <div>GenAI client not available. Please check your API key.</div>;
    }

    return (
        <div className="max-w-2xl mx-auto p-4">
            <form onSubmit={handleSubmit} className="space-y-4">
                <div>
                    <label htmlFor="prompt" className="block text-sm font-medium mb-2">
                        Enter your prompt
                    </label>
                    <textarea
                        id="prompt"
                        value={prompt}
                        onChange={(e) => setPrompt(e.target.value)}
                        className="w-full p-2 border rounded-md bg-gray-800 text-white"
                        rows={4}
                        placeholder="Type your prompt here..."
                    />
                </div>
                <button
                    type="submit"
                    disabled={isLoading || !prompt.trim()}
                    className={`px-4 py-2 rounded-md ${isLoading || !prompt.trim()
                            ? 'bg-gray-500 cursor-not-allowed'
                            : 'bg-blue-500 hover:bg-blue-600'
                        }`}
                >
                    {isLoading ? 'Generating...' : 'Generate'}
                </button>
            </form>

            {response && (
                <div className="mt-8">
                    <h2 className="text-lg font-semibold mb-2">Response:</h2>
                    <div className="p-4 bg-gray-800 rounded-md whitespace-pre-wrap">
                        {response}
                    </div>
                </div>
            )}
        </div>
    );
} 