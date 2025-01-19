import React from 'react';
import { ErrorBoundary as ReactErrorBoundary } from 'react-error-boundary';

interface FallbackProps {
    error: Error;
    resetErrorBoundary: () => void;
}

const ErrorFallback = ({ error, resetErrorBoundary }: FallbackProps) => {
    return (
        <div className="flex flex-col items-center justify-center min-h-screen bg-gray-50">
            <div className="p-8 bg-white rounded-lg shadow-md max-w-lg w-full">
                <h2 className="text-2xl font-bold text-red-600 mb-4">Something went wrong</h2>
                <p className="text-gray-600 mb-4">{error.message}</p>
                <button
                    onClick={resetErrorBoundary}
                    className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
                >
                    Try again
                </button>
            </div>
        </div>
    );
};

export const ErrorBoundary = ({ children }: { children: React.ReactNode }) => {
    return (
        <ReactErrorBoundary
            FallbackComponent={ErrorFallback}
            onReset={() => {
                // Reset the state of your app here
                window.location.reload();
            }}
        >
            {children}
        </ReactErrorBoundary>
    );
};
