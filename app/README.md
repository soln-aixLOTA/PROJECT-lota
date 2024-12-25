# LotaBots Front-End

A modern, responsive front-end for the LotaBots platform, built with Next.js and inspired by the Lusion design aesthetic.

## Features

- 🎨 Modern, minimalist design inspired by Lusion
- 🚀 Built with Next.js 14 and TypeScript
- 💫 Smooth animations with Framer Motion
- 🎮 Interactive 3D elements with Three.js
- 🎯 Responsive design for all devices
- 🎭 Dark mode by default
- ⚡ Optimized performance
- 🔒 Type-safe development
- 📱 Mobile-first approach

## Tech Stack

- **Framework:** Next.js 14
- **Language:** TypeScript
- **Styling:** Tailwind CSS
- **Animations:** Framer Motion
- **3D Graphics:** Three.js with React Three Fiber
- **State Management:** Zustand
- **Data Fetching:** SWR
- **HTTP Client:** Axios
- **Testing:** Jest + React Testing Library
- **E2E Testing:** Cypress

## Getting Started

1. **Clone the repository**

```bash
git clone <repository-url>
cd lotabots-frontend
```

2. **Install dependencies**

```bash
npm install
```

3. **Start the development server**

```bash
npm run dev
```

4. **Open your browser**

Visit [http://localhost:3000](http://localhost:3000) to see the application.

## Project Structure

```
app/
├── app/                    # Next.js app directory
│   ├── layout.tsx         # Root layout component
│   ├── page.tsx           # Landing page
│   └── globals.css        # Global styles
├── components/            # Reusable components
├── lib/                   # Utility functions and hooks
├── public/               # Static assets
│   └── fonts/            # Custom fonts
├── styles/               # Component styles
└── types/                # TypeScript type definitions
```

## Development

### Commands

- `npm run dev` - Start development server
- `npm run build` - Build for production
- `npm start` - Start production server
- `npm run lint` - Run ESLint
- `npm test` - Run tests
- `npm run cypress` - Open Cypress for E2E testing

### Code Style

- Follow the TypeScript best practices
- Use functional components with hooks
- Follow the Airbnb React/JSX Style Guide
- Write meaningful commit messages

## Design System

The design system is inspired by Lusion's aesthetic, featuring:

- **Typography:**
  - Display: Monument Extended
  - Body: Inter

- **Colors:**
  - Primary: #0A0A0A (Dark background)
  - Accent: #00F0FF (Cyan highlights)
  - Text: White with various opacity levels

- **Components:**
  - Glass panels with backdrop blur
  - Gradient text effects
  - Smooth animations
  - Interactive 3D elements
  - Custom buttons and form elements

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is proprietary and confidential.

## Contact

For any questions or concerns, please contact the development team. 