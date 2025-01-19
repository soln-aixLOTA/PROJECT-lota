'use client';

import Navigation from '@/components/Navigation';
import { OrbitControls, Sphere } from '@react-three/drei';
import { Canvas } from '@react-three/fiber';
import { motion } from 'framer-motion';

function AnimatedSphere() {
    return (
        <Sphere args={[1, 32, 32]}>
            <meshStandardMaterial
                color="#00F0FF"
                wireframe
                transparent
                opacity={0.2}
            />
        </Sphere>
    );
}

export default function Home() {
    return (
        <main className="relative min-h-screen">
            <Navigation />

            {/* Hero Section */}
            <section className="relative h-screen flex items-center justify-center overflow-hidden">
                {/* 3D Background */}
                <div className="absolute inset-0 z-0">
                    <Canvas camera={{ position: [0, 0, 5] }}>
                        <ambientLight intensity={0.5} />
                        <pointLight position={[10, 10, 10]} />
                        <OrbitControls
                            enableZoom={false}
                            enablePan={false}
                            autoRotate
                            autoRotateSpeed={0.5}
                        />
                        <AnimatedSphere />
                    </Canvas>
                </div>

                {/* Content */}
                <div className="relative z-10 container mx-auto px-6 text-center">
                    <motion.h1
                        initial={{ opacity: 0, y: 20 }}
                        animate={{ opacity: 1, y: 0 }}
                        transition={{ duration: 0.8 }}
                        className="font-display text-6xl md:text-8xl mb-6"
                    >
                        <span className="gradient-text">LotaBots</span>
                    </motion.h1>

                    <motion.p
                        initial={{ opacity: 0, y: 20 }}
                        animate={{ opacity: 1, y: 0 }}
                        transition={{ duration: 0.8, delay: 0.2 }}
                        className="text-xl md:text-2xl text-white/80 mb-8 max-w-2xl mx-auto"
                    >
                        Enterprise-grade AI-powered conversational agents for the future of business
                    </motion.p>

                    <motion.div
                        initial={{ opacity: 0, y: 20 }}
                        animate={{ opacity: 1, y: 0 }}
                        transition={{ duration: 0.8, delay: 0.4 }}
                        className="flex flex-col md:flex-row items-center justify-center space-y-4 md:space-y-0 md:space-x-4"
                    >
                        <button className="button-primary">Get Started</button>
                        <button className="button-secondary">Learn More</button>
                    </motion.div>
                </div>

                {/* Scroll Indicator */}
                <motion.div
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    transition={{ delay: 1, duration: 1 }}
                    className="absolute bottom-8 left-1/2 transform -translate-x-1/2"
                >
                    <div className="w-6 h-10 rounded-full border-2 border-white/20 flex items-start justify-center p-2">
                        <motion.div
                            animate={{
                                y: [0, 12, 0],
                            }}
                            transition={{
                                duration: 1.5,
                                repeat: Infinity,
                                repeatType: 'loop',
                            }}
                            className="w-1 h-1 rounded-full bg-white"
                        />
                    </div>
                </motion.div>
            </section>
        </main>
    );
} 