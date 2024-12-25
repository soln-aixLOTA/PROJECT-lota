import { cn } from '@/lib/utils';
import { motion } from 'framer-motion';
import React from 'react';

interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
    children: React.ReactNode;
    gradient?: boolean;
    hover?: boolean;
}

export function Card({
    children,
    className,
    gradient = false,
    hover = false,
    ...props
}: CardProps) {
    return (
        <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 20 }}
            transition={{ duration: 0.3 }}
            className={cn(
                'relative rounded-lg bg-secondary/50 backdrop-blur-xl border border-white/10',
                hover && 'transition-transform hover:-translate-y-1 hover:shadow-xl',
                gradient && 'bg-gradient-to-br from-accent/10 via-secondary/50 to-secondary/50',
                className
            )}
            {...props}
        >
            {/* Shimmer effect on hover */}
            {hover && (
                <div className="absolute inset-0 -z-10 overflow-hidden rounded-lg">
                    <div className="absolute inset-0 translate-x-[-100%] bg-gradient-to-r from-transparent via-white/5 to-transparent group-hover:animate-shimmer" />
                </div>
            )}

            {children}
        </motion.div>
    );
}

interface CardHeaderProps extends React.HTMLAttributes<HTMLDivElement> {
    children: React.ReactNode;
}

export function CardHeader({ children, className, ...props }: CardHeaderProps) {
    return (
        <div className={cn('flex flex-col space-y-1.5 p-6', className)} {...props}>
            {children}
        </div>
    );
}

interface CardTitleProps extends React.HTMLAttributes<HTMLHeadingElement> {
    children: React.ReactNode;
}

export function CardTitle({ children, className, ...props }: CardTitleProps) {
    return (
        <h3 className={cn('font-semibold leading-none tracking-tight', className)} {...props}>
            {children}
        </h3>
    );
}

interface CardDescriptionProps extends React.HTMLAttributes<HTMLParagraphElement> {
    children: React.ReactNode;
}

export function CardDescription({ children, className, ...props }: CardDescriptionProps) {
    return (
        <p className={cn('text-sm text-white/60', className)} {...props}>
            {children}
        </p>
    );
}

interface CardContentProps extends React.HTMLAttributes<HTMLDivElement> {
    children: React.ReactNode;
}

export function CardContent({ children, className, ...props }: CardContentProps) {
    return (
        <div className={cn('p-6 pt-0', className)} {...props}>
            {children}
        </div>
    );
}

interface CardFooterProps extends React.HTMLAttributes<HTMLDivElement> {
    children: React.ReactNode;
}

export function CardFooter({ children, className, ...props }: CardFooterProps) {
    return (
        <div className={cn('flex items-center p-6 pt-0', className)} {...props}>
            {children}
        </div>
    );
} 