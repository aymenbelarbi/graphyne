import { useState, useEffect } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Search, Brain, Database, Clock, Activity, TrendingUp } from "lucide-react"

interface Stats {
  total_searches: number
  total_memories: number
  storage_size_bytes: number
  uptime_seconds: number
  status: string
}

const statCards = [
  { key: "total_searches", label: "Total Searches", icon: Search, color: "text-blue-500", bg: "bg-blue-500/10" },
  { key: "total_memories", label: "Total Memories", icon: Brain, color: "text-purple-500", bg: "bg-purple-500/10" },
  { key: "storage_size_bytes", label: "Storage", icon: Database, color: "text-emerald-500", bg: "bg-emerald-500/10" },
  { key: "uptime_seconds", label: "Uptime", icon: Clock, color: "text-amber-500", bg: "bg-amber-500/10" },
]

export default function DashboardPage() {
  const [stats, setStats] = useState<Stats | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState("")

  const fetchStats = async () => {
    try {
      const response = await fetch("http://localhost:8080/admin/stats")
      if (!response.ok) throw new Error("Failed to fetch stats")
      const data = await response.json()
      setStats(data)
    } catch {
      setError("Failed to load statistics. Make sure the Graphyne server is running.")
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchStats()
    const interval = setInterval(fetchStats, 30000)
    return () => clearInterval(interval)
  }, [])

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 B"
    const k = 1024
    const sizes = ["B", "KB", "MB", "GB"]
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i]
  }

  const formatUptime = (seconds: number) => {
    const h = Math.floor(seconds / 3600)
    const m = Math.floor((seconds % 3600) / 60)
    const s = Math.floor(seconds % 60)
    if (h > 0) return `${h}h ${m}m`
    if (m > 0) return `${m}m ${s}s`
    return `${s}s`
  }

  const getStatValue = (key: string) => {
    if (!stats) return "—"
    const val = stats[key as keyof Stats]
    if (key === "storage_size_bytes") return formatBytes(val as number)
    if (key === "uptime_seconds") return formatUptime(val as number)
    return (val as number)?.toLocaleString() || "0"
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center py-24">
        <div className="flex items-center gap-3 text-muted-foreground">
          <Activity className="h-5 w-5 animate-pulse" />
          <span>Loading dashboard...</span>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {error && (
        <div className="rounded-lg border border-destructive/50 bg-destructive/10 px-4 py-3 text-sm text-destructive">
          {error}
        </div>
      )}

      {/* Stats Grid */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {statCards.map((card) => {
          const Icon = card.icon
          return (
            <Card key={card.key} className="overflow-hidden">
              <CardContent className="p-5">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-muted-foreground">{card.label}</p>
                    <p className="mt-1 text-2xl font-bold tracking-tight">{getStatValue(card.key)}</p>
                  </div>
                  <div className={`flex h-11 w-11 items-center justify-center rounded-xl ${card.bg}`}>
                    <Icon className={`h-5 w-5 ${card.color}`} />
                  </div>
                </div>
              </CardContent>
            </Card>
          )
        })}
      </div>

      {/* System Status */}
      <div className="grid gap-4 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Activity className="h-5 w-5 text-primary" />
              System Status
            </CardTitle>
            <CardDescription>Current health and performance metrics</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between rounded-lg bg-muted/50 px-4 py-3">
              <div className="flex items-center gap-3">
                <div className="h-2.5 w-2.5 rounded-full bg-green-500 animate-pulse" />
                <span className="text-sm font-medium">Server Status</span>
              </div>
              <Badge variant="secondary" className="bg-green-500/10 text-green-600 hover:bg-green-500/20">
                Healthy
              </Badge>
            </div>
            <div className="flex items-center justify-between rounded-lg bg-muted/50 px-4 py-3">
              <div className="flex items-center gap-3">
                <TrendingUp className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm font-medium">API Endpoints</span>
              </div>
              <span className="text-sm text-muted-foreground">gRPC + HTTP</span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Database className="h-5 w-5 text-primary" />
              Knowledge Graph
            </CardTitle>
            <CardDescription>Graph database and vector index status</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between rounded-lg bg-muted/50 px-4 py-3">
              <span className="text-sm font-medium">Memory Types</span>
              <div className="flex gap-1.5">
                {["Working", "Episodic", "Semantic", "Procedural"].map((t) => (
                  <Badge key={t} variant="outline" className="text-xs">
                    {t}
                  </Badge>
                ))}
              </div>
            </div>
            <div className="flex items-center justify-between rounded-lg bg-muted/50 px-4 py-3">
              <span className="text-sm font-medium">Search Modes</span>
              <div className="flex gap-1.5">
                {["Lexical", "Vector", "Graph", "Hybrid"].map((m) => (
                  <Badge key={m} variant="outline" className="text-xs">
                    {m}
                  </Badge>
                ))}
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
