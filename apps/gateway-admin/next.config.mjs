/** @type {import('next').NextConfig} */
const allowedDevOrigins = ['127.0.0.1', 'localhost']

if (process.env.LAB_ALLOWED_DEV_ORIGINS) {
  for (const origin of process.env.LAB_ALLOWED_DEV_ORIGINS.split(',')) {
    const trimmed = origin.trim()
    if (trimmed.length > 0) {
      allowedDevOrigins.push(trimmed)
    }
  }
}

const nextConfig = {
  output: 'export',
  trailingSlash: true,
  allowedDevOrigins,
  typescript: {
    ignoreBuildErrors: true,
  },
  images: {
    unoptimized: true,
  },
}

export default nextConfig
