type LogLevel = 'info' | 'warn' | 'error' | 'debug';

interface LogMessage {
    level: LogLevel;
    message: string;
    timestamp: string;
    [key: string]: any;
}

class Logger {
    private isDevelopment = process.env.NODE_ENV === 'development';

    private formatMessage(level: LogLevel, message: string, meta: object = {}): LogMessage {
        return {
            level,
            message,
            timestamp: new Date().toISOString(),
            ...meta,
        };
    }

    private log(level: LogLevel, message: string, meta: object = {}) {
        const formattedMessage = this.formatMessage(level, message, meta);

        if (this.isDevelopment) {
            console[level](JSON.stringify(formattedMessage, null, 2));
        } else {
            // In production, you might want to send logs to a service like CloudWatch, Datadog, etc.
            console[level](JSON.stringify(formattedMessage));
        }
    }

    info(message: string, meta: object = {}) {
        this.log('info', message, meta);
    }

    warn(message: string, meta: object = {}) {
        this.log('warn', message, meta);
    }

    error(message: string, meta: object = {}) {
        this.log('error', message, meta);
    }

    debug(message: string, meta: object = {}) {
        if (this.isDevelopment) {
            this.log('debug', message, meta);
        }
    }
}

export const logger = new Logger();
