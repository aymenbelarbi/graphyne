import { useState, useEffect } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'

interface Stats {
  total_searches: number
  total_memories: number
  storage_size_bytes: number
  uptime_seconds: number
  status: string
}

export default function DashboardPage() {
  const [stats, setStats] = useState<Stats | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')

  const fetchStats = async () => {
    try {
      const response = await fetch('http://localhost:8080/admin/stats')
      if (!response.ok) {
        throw new Error('Failed to fetch stats')
      }
      const data = await response.json()
      setStats(data)
    } catch (err) {
      setError('Failed to load statistics. Make sure the Graphyne server is running.')
      console.error('Stats error:', err)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchStats()
    const interval = setInterval(fetchStats, 30000) // Refresh every 30 seconds
    return () => clearInterval(interval)
  }, [])

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 Bytes'
    const k = 1024
    const sizes = ['Bytes', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  const formatUptime = (seconds: number) => {
    const hours = Math.floor(seconds / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    const secs = Math.floor(seconds % 60)
    return `${hours}h ${minutes}m ${secs}s`
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-muted-foreground">Loading statistics...</p>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {error && (
        <div className="rounded-md bg-destructive/15 p-4 text-destructive">
          {error}
        </div>
      )}
      
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              Total Searches
            </CardTitle>
            <Badge variant="secondary">Search</Badge>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {stats?.total_searches?.toLocaleString() || '0'}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              Total Memories
            </CardTitle>
            <Badge variant="secondary">Memory</Badge>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {stats?.total_memories?.toLocaleString() || '0'}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              Storage Size
            </CardTitle>
            <Badge variant="secondary">Storage</Badge>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {stats ? formatBytes(stats.storage_size_bytes) : '0 Bytes'}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              Uptime
            </CardTitle>
            <Badge variant="secondary">System</Badge>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {stats ? formatUptime(stats.uptime_seconds) : '0s'}
            </div>
          </CardContent>
        </Card>
      </div>

      {stats && (
        <Card>
          <CardHeader>
            <CardTitle>System Status</CardTitle>
            <CardDescription>
              Current system health and status information.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex items-center space-x-4">
              <Badge variant={stats.status === 'healthy' ? 'default' : 'destructive'}>
                {stats.status || 'Unknown'}
              </Badge>
              <span className="text-sm text-muted-foreground">
                Server is {stats.status || 'unknown'}
              </span>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
