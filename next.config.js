/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
  experimental: {
    appDir: true,
  },
  distDir: '.next',
  dir: 'src/frontend',
}

module.exports = nextConfig 