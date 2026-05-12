import { useState, useEffect } from "react"
import { Card, CardContent } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Textarea } from "@/components/ui/textarea"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Badge } from "@/components/ui/badge"
import { Brain, Plus, Trash2, Clock, Hash, Loader2 } from "lucide-react"

interface Memory {
  id: string
  memory_type: string
  content: string
  importance: number
  created_at: string
  last_accessed: string
  access_count: number
}

const memoryTypeColors: Record<string, string> = {
  Working: "bg-blue-500/10 text-blue-600 border-blue-500/20",
  Episodic: "bg-purple-500/10 text-purple-600 border-purple-500/20",
  Semantic: "bg-emerald-500/10 text-emerald-600 border-emerald-500/20",
  Procedural: "bg-amber-500/10 text-amber-600 border-amber-500/20",
}

export default function MemoryPage() {
  const [memories, setMemories] = useState<Memory[]>([])
  const [content, setContent] = useState("")
  const [memoryType, setMemoryType] = useState("Working")
  const [importance, setImportance] = useState("0.5")
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState("")
  const [success, setSuccess] = useState("")
  const [fetching, setFetching] = useState(true)

  const fetchMemories = async () => {
    try {
      const response = await fetch("http://localhost:8080/v1/memory/recall?limit=50")
      if (!response.ok) throw new Error("Failed to fetch")
      const data = await response.json()
      setMemories(data.memories || [])
    } catch (err) {
      console.error("Failed to fetch memories:", err)
    } finally {
      setFetching(false)
    }
  }

  useEffect(() => {
    fetchMemories()
  }, [])

  const handleStore = async () => {
    if (!content.trim()) return
    setLoading(true)
    setError("")
    setSuccess("")
    try {
      const response = await fetch("http://localhost:8080/v1/memory/store", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          memory_type: memoryType,
          content,
          importance: parseFloat(importance),
        }),
      })
      if (!response.ok) throw new Error("Failed to store memory")
      setSuccess("Memory stored successfully!")
      setContent("")
      fetchMemories()
    } catch {
      setError("Failed to store memory. Make sure the Graphyne server is running.")
    } finally {
      setLoading(false)
    }
  }

  const handleDelete = async (id: string) => {
    try {
      await fetch(`http://localhost:8080/v1/memory/${id}`, { method: "DELETE" })
      fetchMemories()
    } catch {
      setError("Failed to delete memory.")
    }
  }

  return (
    <div className="space-y-6">
      {/* Store Memory Form */}
      <Card>
        <CardContent className="p-6">
          <div className="flex items-center gap-3 mb-5">
            <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-primary/10">
              <Plus className="h-5 w-5 text-primary" />
            </div>
            <div>
              <h2 className="text-lg font-semibold">Store New Memory</h2>
              <p className="text-sm text-muted-foreground">Add a new memory to the knowledge base</p>
            </div>
          </div>

          <div className="space-y-4">
            <div className="flex gap-3">
              <Select value={memoryType} onValueChange={setMemoryType}>
                <SelectTrigger className="w-[180px]">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="Working">Working</SelectItem>
                  <SelectItem value="Episodic">Episodic</SelectItem>
                  <SelectItem value="Semantic">Semantic</SelectItem>
                  <SelectItem value="Procedural">Procedural</SelectItem>
                </SelectContent>
              </Select>
              <Select value={importance} onValueChange={setImportance}>
                <SelectTrigger className="w-[140px]">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="0.2">Low (0.2)</SelectItem>
                  <SelectItem value="0.5">Medium (0.5)</SelectItem>
                  <SelectItem value="0.8">High (0.8)</SelectItem>
                  <SelectItem value="1.0">Critical (1.0)</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <Textarea
              placeholder="Enter memory content..."
              value={content}
              onChange={(e) => setContent(e.target.value)}
              rows={4}
            />

            <div className="flex items-center gap-3">
              <Button onClick={handleStore} disabled={loading || !content.trim()}>
                {loading ? <Loader2 className="h-4 w-4 animate-spin mr-2" /> : <Brain className="h-4 w-4 mr-2" />}
                {loading ? "Storing..." : "Store Memory"}
              </Button>
              {error && <p className="text-sm text-destructive">{error}</p>}
              {success && <p className="text-sm text-green-600">{success}</p>}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Memory List */}
      <div>
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold">Stored Memories</h2>
          <Badge variant="secondary">{memories.length} memories</Badge>
        </div>

        {fetching ? (
          <div className="flex items-center justify-center py-16">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : memories.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-16 text-center rounded-xl border border-dashed">
            <Brain className="h-12 w-12 text-muted-foreground/30 mb-3" />
            <p className="text-muted-foreground">No memories stored yet</p>
            <p className="text-sm text-muted-foreground/70 mt-1">Store your first memory using the form above</p>
          </div>
        ) : (
          <div className="space-y-3">
            {memories.map((memory) => (
              <Card key={memory.id} className="transition-colors hover:bg-muted/30">
                <CardContent className="p-4">
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-2">
                        <Badge variant="outline" className={memoryTypeColors[memory.memory_type] || ""}>
                          {memory.memory_type}
                        </Badge>
                        <div className="flex items-center gap-1 text-xs text-muted-foreground">
                          <Hash className="h-3 w-3" />
                          {memory.importance.toFixed(1)}
                        </div>
                      </div>
                      <p className="text-sm leading-relaxed">{memory.content}</p>
                      <div className="flex items-center gap-4 mt-2 text-xs text-muted-foreground">
                        <span className="flex items-center gap-1">
                          <Clock className="h-3 w-3" />
                          {new Date(memory.created_at).toLocaleDateString()}
                        </span>
                        <span>Accessed {memory.access_count}x</span>
                      </div>
                    </div>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8 text-muted-foreground hover:text-destructive"
                      onClick={() => handleDelete(memory.id)}
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
