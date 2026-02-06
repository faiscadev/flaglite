import { FlagIcon } from './FlagIcon';
import { ProjectSelector } from './ProjectSelector';
import { type ReactNode } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { Flag, Globe, LogOut, Menu, Settings, X } from 'lucide-react';
import { useState } from 'react';
import { useAuth } from '../context/AuthContext';

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  const { logout, user, selectedProjectId } = useAuth();
  const location = useLocation();
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  // Navigation items - only show when a project is selected
  const navItems = selectedProjectId
    ? [
        { path: `/projects/${selectedProjectId}/flags`, label: 'Flags', icon: Flag },
        { path: `/projects/${selectedProjectId}/environments`, label: 'Environments', icon: Globe },
        { path: `/projects/${selectedProjectId}/settings`, label: 'Settings', icon: Settings },
      ]
    : [];

  const isActive = (path: string) => 
    location.pathname === path || location.pathname.startsWith(path + '/');

  return (
    <div className="min-h-screen bg-zinc-50">
      {/* Mobile Header */}
      <div className="lg:hidden bg-white border-b border-zinc-200 px-4 py-3 flex items-center justify-between">
        <h1 className="text-xl font-bold text-green-600">
          <FlagIcon className="w-6 h-6 inline-block mr-1" />
          FlagLite
        </h1>
        <button
          onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
          className="p-2 text-zinc-600 hover:text-zinc-900 hover:bg-zinc-100 rounded-lg cursor-pointer"
        >
          {mobileMenuOpen ? <X className="w-6 h-6" /> : <Menu className="w-6 h-6" />}
        </button>
      </div>

      {/* Mobile Menu */}
      {mobileMenuOpen && (
        <div className="lg:hidden fixed inset-0 top-14 z-50 bg-white border-t border-zinc-200">
          <nav className="p-4 space-y-1">
            {navItems.map((item) => {
              const Icon = item.icon;
              return (
                <Link
                  key={item.path}
                  to={item.path}
                  onClick={() => setMobileMenuOpen(false)}
                  className={`flex items-center px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                    isActive(item.path)
                      ? 'bg-green-50 text-green-600'
                      : 'text-zinc-700 hover:bg-zinc-100'
                  }`}
                >
                  <Icon className="w-5 h-5 mr-3" />
                  {item.label}
                </Link>
              );
            })}
            <button
              onClick={() => {
                logout();
                setMobileMenuOpen(false);
              }}
              className="flex items-center w-full px-3 py-2 text-sm font-medium text-zinc-700 rounded-lg hover:bg-zinc-100 cursor-pointer"
            >
              <LogOut className="w-5 h-5 mr-3" />
              Logout
            </button>
          </nav>
        </div>
      )}

      {/* Desktop Sidebar */}
      <aside className="hidden lg:fixed lg:inset-y-0 lg:left-0 lg:flex lg:w-64 lg:flex-col bg-white border-r border-zinc-200">
        <div className="flex flex-col h-full">
          {/* Project Selector (Top) */}
          <ProjectSelector />

          {/* Navigation (Middle) - Scoped to project */}
          <nav className="flex-1 p-4">
            {selectedProjectId ? (
              <div className="space-y-1">
                {navItems.map((item) => {
                  const Icon = item.icon;
                  return (
                    <Link
                      key={item.path}
                      to={item.path}
                      className={`flex items-center gap-3 px-3 py-2 text-sm font-medium rounded-lg transition-colors cursor-pointer ${
                        isActive(item.path)
                          ? 'bg-green-50 text-green-600'
                          : 'text-zinc-600 hover:bg-zinc-50'
                      }`}
                    >
                      <Icon className="w-5 h-5" />
                      {item.label}
                    </Link>
                  );
                })}
              </div>
            ) : (
              <div className="text-sm text-zinc-400 px-3 py-2">
                Select a project to get started
              </div>
            )}
          </nav>

          {/* User Menu (Bottom) */}
          <div className="border-t border-zinc-200 p-4">
            <button
              onClick={logout}
              className="w-full flex items-center gap-3 px-3 py-2 text-sm rounded-lg hover:bg-zinc-50 cursor-pointer"
            >
              <div className="w-8 h-8 rounded-full bg-zinc-200 flex items-center justify-center text-zinc-600 font-medium text-sm">
                {user?.username?.slice(0, 2).toUpperCase() || '??'}
              </div>
              <div className="flex-1 text-left">
                <div className="text-sm font-medium text-zinc-900">
                  {user?.username || 'User'}
                </div>
                {user?.email && (
                  <div className="text-xs text-zinc-500 truncate">
                    {user.email}
                  </div>
                )}
              </div>
              <LogOut className="w-4 h-4 text-zinc-400" />
            </button>
          </div>
        </div>
      </aside>

      {/* Main content */}
      <main className="lg:ml-64 min-h-screen">
        <div className="p-6 lg:p-8">{children}</div>
      </main>
    </div>
  );
}
