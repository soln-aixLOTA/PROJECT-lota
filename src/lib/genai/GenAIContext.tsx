import React, { createContext, ReactNode, useContext } from 'react';
import { InstrumentedGenAIClient } from './InstrumentedGenAIClient';

interface GenAIContextType {
    client: InstrumentedGenAIClient | null;
    isInitialized: boolean;
    error: Error | null;
}

const GenAIContext = createContext<GenAIContextType>({
    client: null,
    isInitialized: false,
    error: null,
});

interface GenAIProviderProps {
    children: ReactNode;
    apiKey?: string;
}

export function GenAIProvider({ children, apiKey }: GenAIProviderProps) {
    const [state, setState] = React.useState<GenAIContextType>({
        client: null,
        isInitialized: false,
        error: null,
    });

    React.useEffect(() => {
        if (!apiKey) {
            setState({
                client: null,
                isInitialized: true,
                error: new Error('GenAI API key not provided'),
            });
            return;
        }

        try {
            const client = new InstrumentedGenAIClient(apiKey);
            setState({
                client,
                isInitialized: true,
                error: null,
            });
        } catch (error) {
            setState({
                client: null,
                isInitialized: true,
                error: error instanceof Error ? error : new Error('Failed to initialize GenAI client'),
            });
        }
    }, [apiKey]);

    return <GenAIContext.Provider value={state}>{children}</GenAIContext.Provider>;
}

export function useGenAI() {
    const context = useContext(GenAIContext);
    if (context === undefined) {
        throw new Error('useGenAI must be used within a GenAIProvider');
    }
    return context;
} 