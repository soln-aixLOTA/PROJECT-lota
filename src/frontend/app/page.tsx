'use client';

import { Environment, MeshDistortMaterial, OrbitControls, Sphere } from '@react-three/drei';
import { Canvas } from '@react-three/fiber';
import { motion } from 'framer-motion';
import Link from 'next/link';
import React from 'react';

function Scene() {
    return (
        <>
            <OrbitControls enableZoom={false} />
            <Environment preset="city" />
            <Sphere args={[1, 32, 32]}>
                <MeshDistortMaterial
                    color="#00F0FF"
                    wireframe
                    transparent
                    opacity={0.2}
                    distort={0.3}
                    speed={2}
                />
            </Sphere>
        </>
    );
}

export default function Home() {
    return (
        <div className="min-h-screen flex flex-col">
            <div className="flex-1 relative">
                {/* Background Canvas */}
                <div className="absolute inset-0 -z-10">
                    <Canvas camera={{ position: [0, 0, 5] }}>
                        <Scene />
                    </Canvas>
                </div>

                {/* Content */}
                <div className="container mx-auto px-4 py-20">
                    <motion.div
                        initial={{ opacity: 0, y: 20 }}
                        animate={{ opacity: 1, y: 0 }}
                        transition={{ duration: 0.8 }}
                        className="max-w-3xl mx-auto text-center"
                    >
                        <h1 className="text-6xl font-bold mb-6 bg-gradient-to-r from-accent to-accent/50 text-transparent bg-clip-text">
                            LotaBots
                        </h1>
                        <p className="text-xl text-white/60 mb-8">
                            Build, deploy, and manage AI agents with ease
                        </p>
                        <Link
                            href="/dashboard"
                            className="inline-block bg-accent hover:bg-accent/90 text-accent-foreground px-8 py-3 rounded-lg font-medium transition-colors"
                        >
                            Get Started
                        </Link>
                    </motion.div>

                    {/* Features */}
                    <div className="grid md:grid-cols-3 gap-8 mt-20">
                        <motion.div
                            initial={{ opacity: 0, y: 20 }}
                            animate={{ opacity: 1, y: 0 }}
                            transition={{ duration: 0.8, delay: 0.2 }}
                            className="bg-secondary/30 backdrop-blur-xl p-6 rounded-lg"
                        >
                            <h3 className="text-xl font-semibold mb-4">Easy Integration</h3>
                            <p className="text-white/60">
                                Seamlessly integrate AI agents into your existing workflows
                            </p>
                        </motion.div>
                        <motion.div
                            initial={{ opacity: 0, y: 20 }}
                            animate={{ opacity: 1, y: 0 }}
                            transition={{ duration: 0.8, delay: 0.4 }}
                            className="bg-secondary/30 backdrop-blur-xl p-6 rounded-lg"
                        >
                            <h3 className="text-xl font-semibold mb-4">Powerful Analytics</h3>
                            <p className="text-white/60">
                                Monitor and optimize your AI agents&apos; performance
                            </p>
                        </motion.div>
                        <motion.div
                            initial={{ opacity: 0, y: 20 }}
                            animate={{ opacity: 1, y: 0 }}
                            transition={{ duration: 0.8, delay: 0.6 }}
                            className="bg-secondary/30 backdrop-blur-xl p-6 rounded-lg"
                        >
                            <h3 className="text-xl font-semibold mb-4">Secure & Scalable</h3>
                            <p className="text-white/60">
                                Enterprise-grade security and scalability built-in
                            </p>
                        </motion.div>
                    </div>
                </div>
            </div>
        </div>
    );
} 