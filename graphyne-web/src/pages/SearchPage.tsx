import { useState } from "react"
import { Card, CardContent } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Search, Sparkles, FileText, ArrowRight } from "lucide-react"

interface SearchResult {
  id: string
  data: string
  score: number
}

export default function SearchPage() {
  const [query, setQuery] = useState("")
  const [mode, setMode] = useState("hybrid")
  const [results, setResults] = useState<SearchResult[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState("")
  const [searched, setSearched] = useState(false)

  const handleSearch = async () => {
    if (!query.trim()) return
    setLoading(true)
    setError("")
    setSearched(true)
    try {
      const response = await fetch(
        `http://localhost:8080/v1/search?query=${encodeURIComponent(query)}&mode=${mode}&collection=default&bucket=default`
      )
      if (!response.ok) throw new Error("Search failed")
      const data = await response.json()
      setResults(data.results || [])
    } catch {
      setError("Search failed. Make sure the Graphyne server is running.")
    } finally {
      setLoading(false)
    }
  }

  const modeDescriptions: Record<string, string> = {
    hybrid: "Combines lexical, vector, and graph search",
    lexical: "BM25 full-text search",
    vector: "HNSW similarity search",
    graph: "Knowledge graph traversal",
  }

  return (
    <div className="space-y-6">
      {/* Search Header */}
      <Card className="border-primary/20 bg-gradient-to-br from-primary/5 to-transparent">
        <CardContent className="p-6">
          <div className="flex items-center gap-3 mb-4">
            <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-primary/10">
              <Search className="h-5 w-5 text-primary" />
            </div>
            <div>
              <h2 className="text-lg font-semibold">Hybrid Search</h2>
              <p className="text-sm text-muted-foreground">Search across lexical, vector, and graph indices</p>
            </div>
          </div>
          <div className="flex gap-3">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                type="text"
                placeholder="Search the knowledge base..."
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleSearch()}
                className="pl-10 h-11"
              />
            </div>
            <Select value={mode} onValueChange={setMode}>
              <SelectTrigger className="w-[160px] h-11">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="hybrid">Hybrid</SelectItem>
                <SelectItem value="lexical">Lexical</SelectItem>
                <SelectItem value="vector">Vector</SelectItem>
                <SelectItem value="graph">Graph</SelectItem>
              </SelectContent>
            </Select>
            <Button onClick={handleSearch} disabled={loading || !query.trim()} className="h-11 px-6">
              {loading ? (
                <Sparkles className="h-4 w-4 animate-spin" />
              ) : (
                <>
                  Search
                  <ArrowRight className="h-4 w-4" />
                </>
              )}
            </Button>
          </div>
          <p className="mt-2 text-xs text-muted-foreground">{modeDescriptions[mode]}</p>
        </CardContent>
      </Card>

      {error && (
        <div className="rounded-lg border border-destructive/50 bg-destructive/10 px-4 py-3 text-sm text-destructive">
          {error}
        </div>
      )}

      {/* Results */}
      {searched && results.length === 0 && !loading && (
        <div className="flex flex-col items-center justify-center py-16 text-center">
          <Search className="h-12 w-12 text-muted-foreground/30 mb-4" />
          <p className="text-muted-foreground">No results found for "{query}"</p>
          <p className="text-sm text-muted-foreground/70 mt-1">Try a different query or search mode</p>
        </div>
      )}

      <div className="space-y-3">
        {results.map((result, index) => (
          <Card key={result.id || index} className="transition-colors hover:bg-muted/30">
            <CardContent className="p-4">
              <div className="flex items-start justify-between gap-4">
                <div className="flex items-start gap-3 min-w-0">
                  <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-primary/10">
                    <FileText className="h-4 w-4 text-primary" />
                  </div>
                  <div className="min-w-0">
                    <p className="text-sm font-medium truncate">{result.data}</p>
                    <p className="text-xs text-muted-foreground mt-1">ID: {result.id}</p>
                  </div>
                </div>
                <Badge variant="secondary" className="shrink-0">
                  {(result.score * 100).toFixed(1)}%
                </Badge>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  )
}
