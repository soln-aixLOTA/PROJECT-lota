import { Button, ButtonProps } from '@/components/ui/Button';
import React from 'react';

type CustomDropdownButtonProps = ButtonProps & {
    children?: React.ReactNode;
};

export function CustomDropdownButton({ children, ...props }: CustomDropdownButtonProps) {
    return (
        <Button {...props}>
            {children}
        </Button>
    );
} 