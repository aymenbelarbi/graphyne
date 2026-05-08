import { useState } from 'react'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Search, Brain, LayoutDashboard, Settings } from 'lucide-react'
import SearchPage from '@/pages/SearchPage'
import MemoryPage from '@/pages/MemoryPage'
import DashboardPage from '@/pages/DashboardPage'
import SettingsPage from '@/pages/SettingsPage'

function App() {
  const [activeTab, setActiveTab] = useState('search')

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="container flex h-14 items-center">
          <div className="mr-4 flex">
            <a className="mr-6 flex items-center space-x-2" href="/">
              <Brain className="h-6 w-6 text-primary" />
              <span className="hidden font-bold sm:inline-block">
                Graphyne
              </span>
            </a>
          </div>
          <div className="flex flex-1 items-center justify-between space-x-2 md:justify-end">
            <nav className="flex items-center space-x-6">
              <a
                href="#search"
                onClick={(e) => { e.preventDefault(); setActiveTab('search') }}
                className={`text-sm font-medium transition-colors hover:text-primary ${activeTab === 'search' ? 'text-primary' : 'text-muted-foreground'}`}
              >
                <Search className="mr-1 inline h-4 w-4" />
                Search
              </a>
              <a
                href="#memory"
                onClick={(e) => { e.preventDefault(); setActiveTab('memory') }}
                className={`text-sm font-medium transition-colors hover:text-primary ${activeTab === 'memory' ? 'text-primary' : 'text-muted-foreground'}`}
              >
                <Brain className="mr-1 inline h-4 w-4" />
                Memory
              </a>
              <a
                href="#dashboard"
                onClick={(e) => { e.preventDefault(); setActiveTab('dashboard') }}
                className={`text-sm font-medium transition-colors hover:text-primary ${activeTab === 'dashboard' ? 'text-primary' : 'text-muted-foreground'}`}
              >
                <LayoutDashboard className="mr-1 inline h-4 w-4" />
                Dashboard
              </a>
              <a
                href="#settings"
                onClick={(e) => { e.preventDefault(); setActiveTab('settings') }}
                className={`text-sm font-medium transition-colors hover:text-primary ${activeTab === 'settings' ? 'text-primary' : 'text-muted-foreground'}`}
              >
                <Settings className="mr-1 inline h-4 w-4" />
                Settings
              </a>
            </nav>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="container mx-auto py-6">
        <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
          <TabsList className="grid w-full grid-cols-4">
            <TabsTrigger value="search">Search</TabsTrigger>
            <TabsTrigger value="memory">Memory</TabsTrigger>
            <TabsTrigger value="dashboard">Dashboard</TabsTrigger>
            <TabsTrigger value="settings">Settings</TabsTrigger>
          </TabsList>
          
          <TabsContent value="search" className="mt-6">
            <Card>
              <CardHeader>
                <CardTitle>Search Interface</CardTitle>
                <CardDescription>
                  Search the knowledge base using hybrid, lexical, vector, or graph modes.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <SearchPage />
              </CardContent>
            </Card>
          </TabsContent>
          
          <TabsContent value="memory" className="mt-6">
            <Card>
              <CardHeader>
                <CardTitle>Memory Browser</CardTitle>
                <CardDescription>
                  Store and browse memories (working, episodic, semantic, procedural).
                </CardDescription>
              </CardHeader>
              <CardContent>
                <MemoryPage />
              </CardContent>
            </Card>
          </TabsContent>
          
          <TabsContent value="dashboard" className="mt-6">
            <Card>
              <CardHeader>
                <CardTitle>Dashboard</CardTitle>
                <CardDescription>
                  View system statistics and health information.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <DashboardPage />
              </CardContent>
            </Card>
          </TabsContent>
          
          <TabsContent value="settings" className="mt-6">
            <Card>
              <CardHeader>
                <CardTitle>Settings</CardTitle>
                <CardDescription>
                  Configure Graphyne server settings.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <SettingsPage />
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </main>
    </div>
  )
}

export default App
