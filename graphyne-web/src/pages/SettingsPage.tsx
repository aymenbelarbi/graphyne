import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useState } from 'react'

export default function SettingsPage() {
  const [serverUrl, setServerUrl] = useState('http://localhost:8080')
  const [saved, setSaved] = useState(false)

  const handleSave = () => {
    // In a real app, you'd save this to localStorage or a config file
    localStorage.setItem('graphyne-server-url', serverUrl)
    setSaved(true)
    setTimeout(() => setSaved(false), 3000)
  }

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Server Configuration</CardTitle>
          <CardDescription>
            Configure the Graphyne server connection settings.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="server-url">Server URL</Label>
            <Input
              id="server-url"
              type="url"
              value={serverUrl}
              onChange={(e) => setServerUrl(e.target.value)}
              placeholder="http://localhost:8080"
            />
            <p className="text-sm text-muted-foreground">
              The URL where the Graphyne server is running.
            </p>
          </div>
          
          <Button onClick={handleSave}>
            {saved ? 'Saved!' : 'Save Settings'}
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>About Graphyne</CardTitle>
          <CardDescription>
            Information about the Graphyne system.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <h3 className="text-lg font-medium">Graphyne</h3>
            <p className="text-sm text-muted-foreground">
              A high-performance knowledge retrieval system with hybrid search capabilities,
              memory management, and graph-based knowledge representation.
            </p>
          </div>
          
          <div className="space-y-2">
            <h4 className="text-sm font-medium">Features</h4>
            <ul className="list-disc list-inside text-sm text-muted-foreground space-y-1">
              <li>Hybrid search (lexical, vector, graph)</li>
              <li>Memory types (working, episodic, semantic, procedural)</li>
              <li>Knowledge graph with GraphRAG</li>
              <li>Admin operations and health monitoring</li>
              <li>Prometheus metrics and structured logging</li>
            </ul>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
