import { cn } from '@/lib/utils';
import React from 'react';

export interface TextareaProps
    extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
    error?: boolean;
}

const Textarea = React.forwardRef<HTMLTextAreaElement, TextareaProps>(
    ({ className, error, ...props }, ref) => {
        return (
            <textarea
                className={cn(
                    'w-full px-3 py-2 rounded-md bg-white/5 border border-white/10',
                    'focus:border-accent focus:ring-1 focus:ring-accent transition-colors',
                    'placeholder:text-white/40 min-h-[80px] resize-y',
                    error && 'border-red-400 focus:border-red-400 focus:ring-red-400',
                    className
                )}
                ref={ref}
                {...props}
            />
        );
    }
);

Textarea.displayName = 'Textarea';

export { Textarea };
