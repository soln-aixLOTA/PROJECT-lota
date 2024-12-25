'use client';

import { motion } from 'framer-motion';
import { useEffect, useRef } from 'react';

interface Point {
    x: number;
    y: number;
}

interface MetricsChartProps {
    data: number[];
    height?: number;
    gradient?: boolean;
    animated?: boolean;
}

export default function MetricsChart({
    data,
    height = 100,
    gradient = true,
    animated = true,
}: MetricsChartProps) {
    const svgRef = useRef<SVGSVGElement>(null);
    const maxValue = Math.max(...data);
    const minValue = Math.min(...data);

    // Convert data points to SVG coordinates
    const points: Point[] = data.map((value, index) => ({
        x: (index / (data.length - 1)) * 100,
        y: 100 - ((value - minValue) / (maxValue - minValue)) * 100,
    }));

    // Create SVG path
    const createPath = (points: Point[]): string => {
        const line = points
            .map((point, index) => `${index === 0 ? 'M' : 'L'} ${point.x} ${point.y}`)
            .join(' ');

        // Add the area fill
        return `${line} L 100 100 L 0 100 Z`;
    };

    const pathLength = useRef<number>(0);

    useEffect(() => {
        if (svgRef.current) {
            const path = svgRef.current.querySelector('path');
            if (path) {
                pathLength.current = path.getTotalLength();
            }
        }
    }, [data]);

    return (
        <div className="w-full" style={{ height }}>
            <svg
                ref={svgRef}
                viewBox="0 0 100 100"
                preserveAspectRatio="none"
                className="w-full h-full"
            >
                <defs>
                    <linearGradient id="gradient" x1="0" x2="0" y1="0" y2="1">
                        <stop offset="0%" stopColor="rgb(0, 240, 255)" stopOpacity="0.3" />
                        <stop offset="100%" stopColor="rgb(0, 240, 255)" stopOpacity="0" />
                    </linearGradient>
                </defs>
                <motion.path
                    d={createPath(points)}
                    fill={gradient ? 'url(#gradient)' : 'none'}
                    stroke="rgb(0, 240, 255)"
                    strokeWidth="0.5"
                    vectorEffect="non-scaling-stroke"
                    initial={
                        animated
                            ? {
                                pathLength: 0,
                                opacity: 0,
                            }
                            : undefined
                    }
                    animate={{
                        pathLength: 1,
                        opacity: 1,
                    }}
                    transition={{
                        duration: 2,
                        ease: 'easeInOut',
                    }}
                />
            </svg>
        </div>
    );
} 