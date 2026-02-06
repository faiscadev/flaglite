# FlagLite Dashboard

Web-based dashboard for managing feature flags with FlagLite.

## Features

- **Authentication**: Login and Signup with JWT-based auth
- **Projects**: Create and manage multiple projects
- **Environments**: Switch between development, staging, and production
- **Feature Flags**: Create, toggle, and delete flags
- **SDK Integration**: Copy-ready code snippets for JavaScript, Python, Go, and Rust

## Tech Stack

- React 18 with TypeScript
- Vite for build tooling
- TailwindCSS for styling
- React Router for navigation
- React Query for API state management
- Axios for HTTP requests

## Development

```bash
# Install dependencies
npm install

# Start dev server (runs on port 3001)
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview

# Lint
npm run lint
```

## Configuration

Create a `.env` file (or use `.env.example` as template):

```env
VITE_API_URL=https://api.flaglite.dev
```

## Docker

Build and run with Docker:

```bash
# Build image
docker build -t flaglite-dashboard .

# Run container
docker run -p 3001:80 flaglite-dashboard
```

Or use docker-compose from the project root:

```bash
docker compose up dashboard
```

## Project Structure

```
src/
├── components/     # Reusable UI components
│   ├── Alert.tsx
│   ├── Button.tsx
│   ├── Input.tsx
│   ├── Layout.tsx
│   ├── Modal.tsx
│   └── Toggle.tsx
├── context/        # React contexts
│   └── AuthContext.tsx
├── lib/            # API client and utilities
│   └── api.ts
├── pages/          # Page components
│   ├── FlagDetailPage.tsx
│   ├── LoginPage.tsx
│   ├── ProjectDetailPage.tsx
│   ├── ProjectsPage.tsx
│   └── SignupPage.tsx
├── types.ts        # TypeScript types
├── App.tsx         # App routes
└── main.tsx        # Entry point
```

## API Endpoints Used

- `POST /v1/auth/signup` - Create account
- `POST /v1/auth/login` - Login
- `GET /v1/projects` - List projects
- `POST /v1/projects` - Create project
- `GET /v1/projects/:id/environments` - List environments
- `GET /v1/flags` - List flags (with project API key)
- `POST /v1/flags` - Create flag
- `POST /v1/flags/:key/toggle` - Toggle flag
- `DELETE /v1/flags/:key` - Delete flag
# Dashboard CI/CD
