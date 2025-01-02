import { motion, useSpring } from 'framer-motion';
import { useEffect, useRef, useState } from 'react';

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

interface AnimationState {
    isAnimating: boolean;
    values: {
        pathLength: number;
        opacity: number;
    };
}

function ErrorFallback({ error }: { error: Error }) {
    return (
        <div className="w-full h-full flex items-center justify-center bg-red-500/10 rounded-lg">
            <div className="text-red-500 text-sm">
                Failed to render chart: {error.message}
            </div>
        </div>
    );
}

// Async data processing function
async function processChartData(data: number[], width: number, height: number) {
    // Simulate heavy computation
    await new Promise(resolve => setTimeout(resolve, 0));

    const padding = 20;
    const graphWidth = width - padding * 2;
    const graphHeight = height - padding * 2;

    const min = Math.min(...data);
    const max = Math.max(...data);
    const range = max - min || 1;

    return data.map((value, index) => ({
        x: padding + (index * graphWidth) / (data.length - 1),
        y: padding + graphHeight - ((value - min) * graphHeight) / range,
    }));
}

function ChartContent({
    points,
    gradient,
    animated,
    pathLength,
}: {
    points: Point[];
    gradient: boolean;
    animated: boolean;
    pathLength: { current: number };
}) {
    const [animationState, setAnimationState] = useState<AnimationState>({
        isAnimating: false,
        values: {
            pathLength: 0,
            opacity: 0,
        }
    });

    useEffect(() => {
        if (animated) {
            setAnimationState({
                isAnimating: true,
                values: {
                    pathLength: 1,
                    opacity: 1,
                }
            });
        }
    }, [animated]);

    // Create SVG path
    const createPath = (points: Point[]): string => {
        const line = points
            .map((point, index) => `${index === 0 ? 'M' : 'L'} ${point.x} ${point.y}`)
            .join(' ');
        return `${line} L 100 100 L 0 100 Z`;
    };

    const springConfig = {
        type: "spring" as const,
        stiffness: 100,
        damping: 30,
        restDelta: 0.001
    };

    const pathSpring = useSpring(0, springConfig);
    const opacitySpring = useSpring(0, springConfig);

    useEffect(() => {
        if (animated && animationState.values) {
            pathSpring.set(animationState.values.pathLength);
            opacitySpring.set(animationState.values.opacity);
        }
    }, [animated, animationState, pathSpring, opacitySpring]);

    return (
        <>
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
                style={{
                    pathLength: pathSpring,
                    opacity: opacitySpring,
                }}
            />
        </>
    );
}

function ChartPoints({ points }: { points: Point[] }) {
    return (
        <>
            {points.map((point, index) => (
                <motion.circle
                    key={index}
                    cx={point.x}
                    cy={point.y}
                    r={4}
                    fill="rgb(0, 240, 255)"
                    initial={{ scale: 0 }}
                    animate={{ scale: 1 }}
                    transition={{ delay: index * 0.1 }}
                />
            ))}
        </>
    );
}

export default function MetricsChart({
    data,
    height = 100,
    gradient = true,
    animated = true,
}: MetricsChartProps) {
    const containerRef = useRef<HTMLDivElement>(null);
    const pathLength = useRef<number>(0);
    const [dimensions, setDimensions] = useState({ width: 0, height: 0 });
    const [points, setPoints] = useState<Point[]>([]);
    const [isLoading, setIsLoading] = useState(true);

    useEffect(() => {
        if (!containerRef.current) return;

        const resizeObserver = new ResizeObserver((entries) => {
            const { width } = entries[0].contentRect;
            setDimensions({ width, height });
        });

        resizeObserver.observe(containerRef.current);
        return () => resizeObserver.disconnect();
    }, [height]);

    useEffect(() => {
        const loadData = async () => {
            setIsLoading(true);
            try {
                const processedPoints = await processChartData(
                    data,
                    dimensions.width || 100,
                    height
                );
                setPoints(processedPoints);
            } catch (error) {
                console.error('Error processing chart data:', error);
            } finally {
                setIsLoading(false);
            }
        };

        loadData();
    }, [data, dimensions.width, height]);

    if (!data || data.length === 0) {
        return (
            <div className="w-full h-full flex items-center justify-center bg-secondary/30 rounded-lg">
                <div className="text-white/60 text-sm">No data available</div>
            </div>
        );
    }

    return (
        <div ref={containerRef} className="w-full relative" style={{ height }}>
            <svg
                viewBox="0 0 100 100"
                preserveAspectRatio="none"
                className="w-full h-full"
            >
                {isLoading ? (
                    <rect
                        width="100"
                        height="100"
                        fill="none"
                        stroke="rgb(0, 240, 255)"
                        strokeWidth="0.5"
                        strokeDasharray="2 2"
                    />
                ) : (
                    <>
                        <ChartContent
                            points={points}
                            gradient={gradient}
                            animated={animated}
                            pathLength={pathLength}
                        />
                        <ChartPoints points={points} />
                    </>
                )}
            </svg>

            {/* Y-axis labels */}
            <div className="absolute left-0 top-0 bottom-0 flex flex-col justify-between pointer-events-none">
                <span className="text-xs text-white/40">{Math.max(...data).toLocaleString()}</span>
                <span className="text-xs text-white/40">{Math.min(...data).toLocaleString()}</span>
            </div>
        </div>
    );
}