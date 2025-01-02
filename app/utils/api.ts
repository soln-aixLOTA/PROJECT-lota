import axios from 'axios';

const api = axios.create({
  baseURL: process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:3000',
  headers: {
    'Content-Type': 'application/json',
  },
});

// Add request interceptor for authentication
api.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('auth_token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    return Promise.reject(new Error(error.message));
  }
);

// Add response interceptor for error handling
api.interceptors.response.use(
  (response) => response,
  async (error) => {
    const originalRequest = error.config;

    // Handle token expiration
    if (error.response?.status === 401 && !originalRequest._retry) {
      originalRequest._retry = true;

      try {
        const refreshToken = localStorage.getItem('refresh_token');
        const response = await api.post('/api/auth/refresh', {
          refresh_token: refreshToken,
        });

        const { token } = response.data;
        localStorage.setItem('auth_token', token);

        originalRequest.headers.Authorization = `Bearer ${token}`;
        return api(originalRequest);
      } catch (error) {
        // Refresh token failed, redirect to login
        localStorage.removeItem('auth_token');
        localStorage.removeItem('refresh_token');
        window.location.href = '/login';
        return Promise.reject(new Error('Authentication failed'));
      }
    }

    // Handle rate limiting
    if (error.response?.status === 429) {
      const retryAfter = error.response.headers['retry-after'] ?? 60;
      return new Promise((resolve) => {
        setTimeout(() => {
          resolve(api(originalRequest));
        }, retryAfter * 1000);
      });
    }

    return Promise.reject(new Error(error.message));
  }
);

// Error handling wrapper
const handleApiError = (error: any) => {
  if (error.response) {
    // Server responded with error
    const message = error.response.data.message ?? 'An error occurred';
    const status = error.response.status;
    return { message, status };
  } else if (error.request) {
    // Request made but no response
    return {
      message: 'Unable to connect to server',
      status: 0,
    };
  } else {
    // Request setup error
    return {
      message: error.message ?? 'An error occurred',
      status: 0,
    };
  }
};

export { api, handleApiError };
