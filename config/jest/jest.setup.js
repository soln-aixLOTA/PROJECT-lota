// Learn more: https://github.com/testing-library/jest-dom
import '@testing-library/jest-dom';
import { v4 as uuidv4 } from 'uuid';

// Mock Next.js router
jest.mock('next/router', () => ({
    useRouter: () => ({
        push: jest.fn(),
        replace: jest.fn(),
        prefetch: jest.fn(),
        query: {},
    }),
}));

// Mock Next.js navigation hooks
jest.mock('next/navigation', () => ({
    useRouter: () => ({
        push: jest.fn(),
        replace: jest.fn(),
        prefetch: jest.fn(),
        back: jest.fn(),
        forward: jest.fn(),
    }),
    usePathname: () => '/',
    useSearchParams: () => new URLSearchParams(),
}));

// Mock Next.js Image component
jest.mock('next/image', () => ({
    __esModule: true,
    default: (props) => {
        // eslint-disable-next-line @next/next/no-img-element
        return <img {...props} alt={props.alt} />;
    },
}));

// Mock Next.js Head component
jest.mock('next/head', () => {
    return {
        __esModule: true,
        default: ({ children }) => {
            return <>{children}</>;
        },
    };
});

// Setup global fetch mock
global.fetch = jest.fn(() =>
    Promise.resolve({
        json: () => Promise.resolve({}),
        ok: true,
        status: 200,
    })
);

// Mock window.matchMedia
Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: jest.fn().mockImplementation(query => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: jest.fn(),
        removeListener: jest.fn(),
        addEventListener: jest.fn(),
        removeEventListener: jest.fn(),
        dispatchEvent: jest.fn(),
    })),
});

// Mock window.ResizeObserver
global.ResizeObserver = class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
};

// Mock window.IntersectionObserver
global.IntersectionObserver = class IntersectionObserver {
    constructor() {}
    observe() {}
    unobserve() {}
    disconnect() {}
};

// Setup for Next.js Request/Response
const { Request, Response, Headers, fetch } = require('node-fetch');
global.Request = Request;
global.Response = class {
  static json(data) {
    return { json: () => data };
  }
};
global.Headers = Headers;
global.fetch = fetch;

if (!global.crypto) {
  global.crypto = {
    randomUUID: () => 'test-uuid',
  };
} else if (!global.crypto.randomUUID) {
  global.crypto.randomUUID = () => 'test-uuid';
}

global.ResponseCookies = class {
  constructor() {}
  getSetCookie() {
    return [];
  }
};

global.crypto = {
    ...global.crypto,
    randomUUID: uuidv4,
};
