import { NextApiRequest, NextApiResponse } from 'next';
import { logger } from '../utils/logger';

export interface ErrorResponse {
    error: string;
    message: string;
    code: string;
    timestamp: string;
    requestId?: string;
}

export class ApiError extends Error {
    constructor(
        public statusCode: number,
        public message: string,
        public code: string = 'INTERNAL_ERROR'
    ) {
        super(message);
        this.name = 'ApiError';
    }
}

export const errorHandler = (handler: any) => {
    return async (req: NextApiRequest, res: NextApiResponse) => {
        try {
            return await handler(req, res);
        } catch (error) {
            logger.error('API Error:', {
                error,
                path: req.url,
                method: req.method,
                requestId: req.headers['x-request-id'],
            });

            const statusCode = error instanceof ApiError ? error.statusCode : 500;
            const errorResponse: ErrorResponse = {
                error: error instanceof ApiError ? error.code : 'INTERNAL_ERROR',
                message: error instanceof ApiError ? error.message : 'An unexpected error occurred',
                code: error instanceof ApiError ? error.code : 'INTERNAL_ERROR',
                timestamp: new Date().toISOString(),
                requestId: req.headers['x-request-id'] as string,
            };

            // Don't expose internal error details in production
            if (process.env.NODE_ENV === 'development' && !(error instanceof ApiError)) {
                errorResponse.message = error.message || errorResponse.message;
            }

            res.status(statusCode).json(errorResponse);
        }
    };
};
