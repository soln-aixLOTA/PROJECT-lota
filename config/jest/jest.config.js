const nextJest = require('next/jest');

const createJestConfig = nextJest({
    dir: './src/frontend',
});

const customJestConfig = {
    setupFilesAfterEnv: ['<rootDir>/config/jest/jest.setup.js'],
    testEnvironment: 'jest-environment-jsdom',
    moduleNameMapper: {
        '^@/(.*)$': '<rootDir>/src/$1',
        '^@/frontend/(.*)$': '<rootDir>/src/frontend/$1',
        '^@/backend/(.*)$': '<rootDir>/src/backend/$1',
        '^@/common/(.*)$': '<rootDir>/src/common/$1',
        '^@/config/(.*)$': '<rootDir>/config/$1',
    },
    testMatch: [
        '<rootDir>/tests/unit/**/*.test.{js,jsx,ts,tsx}',
        '<rootDir>/tests/integration/**/*.test.{js,jsx,ts,tsx}',
    ],
    collectCoverageFrom: [
        'src/**/*.{js,jsx,ts,tsx}',
        '!src/**/*.d.ts',
        '!src/**/*.stories.{js,jsx,ts,tsx}',
    ],
    coverageDirectory: 'coverage',
};

module.exports = createJestConfig(customJestConfig); 