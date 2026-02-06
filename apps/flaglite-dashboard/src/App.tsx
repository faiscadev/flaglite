import { Routes, Route, Navigate } from 'react-router-dom';
import { AuthProvider, useAuth } from './context/AuthContext';
import { LoginPage } from './pages/LoginPage';
import { SignupPage } from './pages/SignupPage';
import { ProjectsPage } from './pages/ProjectsPage';
import { ProjectFlagsPage } from './pages/ProjectFlagsPage';
import { ProjectEnvironmentsPage } from './pages/ProjectEnvironmentsPage';
import { ProjectSettingsPage } from './pages/ProjectSettingsPage';
import { FlagDetailPage } from './pages/FlagDetailPage';

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated } = useAuth();

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  return <>{children}</>;
}

function PublicRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated } = useAuth();

  if (isAuthenticated) {
    return <Navigate to="/projects" replace />;
  }

  return <>{children}</>;
}

function DefaultRedirect() {
  const { selectedProjectId } = useAuth();
  
  // If there's a selected project, go to its flags page
  if (selectedProjectId) {
    return <Navigate to={`/projects/${selectedProjectId}/flags`} replace />;
  }
  
  // Otherwise, go to projects list
  return <Navigate to="/projects" replace />;
}

function AppRoutes() {
  return (
    <Routes>
      <Route
        path="/login"
        element={
          <PublicRoute>
            <LoginPage />
          </PublicRoute>
        }
      />
      <Route
        path="/signup"
        element={
          <PublicRoute>
            <SignupPage />
          </PublicRoute>
        }
      />
      <Route
        path="/projects"
        element={
          <ProtectedRoute>
            <ProjectsPage />
          </ProtectedRoute>
        }
      />
      {/* Project-scoped routes */}
      <Route
        path="/projects/:projectId/flags"
        element={
          <ProtectedRoute>
            <ProjectFlagsPage />
          </ProtectedRoute>
        }
      />
      <Route
        path="/projects/:projectId/flags/:flagKey"
        element={
          <ProtectedRoute>
            <FlagDetailPage />
          </ProtectedRoute>
        }
      />
      <Route
        path="/projects/:projectId/environments"
        element={
          <ProtectedRoute>
            <ProjectEnvironmentsPage />
          </ProtectedRoute>
        }
      />
      <Route
        path="/projects/:projectId/settings"
        element={
          <ProtectedRoute>
            <ProjectSettingsPage />
          </ProtectedRoute>
        }
      />
      {/* Legacy route - redirect to new structure */}
      <Route
        path="/projects/:projectId"
        element={
          <ProtectedRoute>
            <ProjectDetailRedirect />
          </ProtectedRoute>
        }
      />
      <Route path="/" element={<ProtectedRoute><DefaultRedirect /></ProtectedRoute>} />
      <Route path="*" element={<ProtectedRoute><DefaultRedirect /></ProtectedRoute>} />
    </Routes>
  );
}

// Redirect legacy /projects/:projectId to /projects/:projectId/flags
function ProjectDetailRedirect() {
  const projectId = window.location.pathname.split('/')[2];
  return <Navigate to={`/projects/${projectId}/flags`} replace />;
}

export default function App() {
  return (
    <AuthProvider>
      <AppRoutes />
    </AuthProvider>
  );
}
