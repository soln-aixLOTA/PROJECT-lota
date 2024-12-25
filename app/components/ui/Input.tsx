import { cn } from '@/lib/utils';
import React from 'react';

export interface InputProps
    extends React.InputHTMLAttributes<HTMLInputElement> {
    error?: boolean;
}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
    ({ className, error, ...props }, ref) => {
        return (
            <input
                className={cn(
                    'w-full px-3 py-2 rounded-md bg-white/5 border border-white/10',
                    'focus:border-accent focus:ring-1 focus:ring-accent transition-colors',
                    'placeholder:text-white/40',
                    error && 'border-red-400 focus:border-red-400 focus:ring-red-400',
                    className
                )}
                ref={ref}
                {...props}
            />
        );
    }
);

Input.displayName = 'Input';

export { Input };
