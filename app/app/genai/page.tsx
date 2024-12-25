'use client';

import { GenAIExample } from '@/components/GenAIExample';

export default function GenAIPage() {
    return (
        <div className="container mx-auto py-8">
            <h1 className="text-3xl font-bold mb-8 text-center">GenAI Demo</h1>
            <GenAIExample />
        </div>
    );
} 