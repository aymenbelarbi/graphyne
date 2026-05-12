import { useState } from "react"
import { Search, Brain, LayoutDashboard, Settings, Zap, Database, Activity, ChevronRight, Menu, X } from "lucide-react"
import { cn } from "@/lib/utils"
import DashboardPage from "@/pages/DashboardPage"
import SearchPage from "@/pages/SearchPage"
import MemoryPage from "@/pages/MemoryPage"
import SettingsPage from "@/pages/SettingsPage"

const navItems = [
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
  { id: "search", label: "Search", icon: Search },
  { id: "memory", label: "Memory", icon: Brain },
  { id: "settings", label: "Settings", icon: Settings },
]

function App() {
  const [activeTab, setActiveTab] = useState("dashboard")
  const [sidebarOpen, setSidebarOpen] = useState(false)

  return (
    <div className="flex h-screen bg-background">
      {/* Mobile overlay */}
      {sidebarOpen && (
        <div className="fixed inset-0 z-40 bg-black/50 lg:hidden" onClick={() => setSidebarOpen(false)} />
      )}

      {/* Sidebar */}
      <aside
        className={cn(
          "fixed inset-y-0 left-0 z-50 w-64 bg-card border-r transform transition-transform duration-200 ease-in-out lg:relative lg:translate-x-0",
          sidebarOpen ? "translate-x-0" : "-translate-x-full"
        )}
      >
        <div className="flex h-full flex-col">
          {/* Logo */}
          <div className="flex h-16 items-center gap-3 px-6 border-b">
            <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-primary">
              <Zap className="h-5 w-5 text-primary-foreground" />
            </div>
            <div>
              <h1 className="font-bold text-lg leading-none">Graphyne</h1>
              <p className="text-xs text-muted-foreground">Knowledge Engine</p>
            </div>
            <button className="ml-auto lg:hidden" onClick={() => setSidebarOpen(false)}>
              <X className="h-5 w-5" />
            </button>
          </div>

          {/* Navigation */}
          <nav className="flex-1 p-4 space-y-1">
            {navItems.map((item) => {
              const Icon = item.icon
              const isActive = activeTab === item.id
              return (
                <button
                  key={item.id}
                  onClick={() => { setActiveTab(item.id); setSidebarOpen(false) }}
                  className={cn(
                    "flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-all",
                    isActive
                      ? "bg-primary text-primary-foreground shadow-sm"
                      : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                  )}
                >
                  <Icon className="h-5 w-5 shrink-0" />
                  <span>{item.label}</span>
                  {isActive && <ChevronRight className="h-4 w-4 ml-auto" />}
                </button>
              )
            })}
          </nav>

          {/* Status */}
          <div className="p-4 border-t">
            <div className="flex items-center gap-3 rounded-lg bg-muted/50 px-3 py-2.5">
              <Activity className="h-4 w-4 text-green-500" />
              <div className="flex-1 min-w-0">
                <p className="text-xs font-medium">Server Online</p>
                <p className="text-xs text-muted-foreground">localhost:8080</p>
              </div>
            </div>
          </div>
        </div>
      </aside>

      {/* Main content */}
      <div className="flex flex-1 flex-col min-w-0">
        {/* Top bar */}
        <header className="flex h-16 items-center gap-4 border-b bg-card px-6">
          <button className="lg:hidden" onClick={() => setSidebarOpen(true)}>
            <Menu className="h-5 w-5" />
          </button>
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <span>Graphyne</span>
            <ChevronRight className="h-3 w-3" />
            <span className="font-medium text-foreground capitalize">{activeTab}</span>
          </div>
          <div className="ml-auto flex items-center gap-3">
            <div className="hidden sm:flex items-center gap-2 text-xs text-muted-foreground bg-muted/50 rounded-full px-3 py-1.5">
              <Database className="h-3 w-3" />
              <span>gRPC: 50051</span>
              <span className="text-border">|</span>
              <span>HTTP: 8080</span>
            </div>
          </div>
        </header>

        {/* Page content */}
        <main className="flex-1 overflow-auto">
          <div className="mx-auto max-w-6xl p-6">
            {activeTab === "dashboard" && <DashboardPage />}
            {activeTab === "search" && <SearchPage />}
            {activeTab === "memory" && <MemoryPage />}
            {activeTab === "settings" && <SettingsPage />}
          </div>
        </main>
      </div>
    </div>
  )
}

export default App
