import { useState, useEffect } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

interface SearchResult {
  id: string
  content: string
  score: number
  mode: string
}

export default function SearchPage() {
  const [query, setQuery] = useState('')
  const [mode, setMode] = useState('hybrid')
  const [results, setResults] = useState<SearchResult[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  const handleSearch = async () => {
    if (!query.trim()) return
    
    setLoading(true)
    setError('')
    
    try {
      const response = await fetch(`http://localhost:8080/v1/search?q=${encodeURIComponent(query)}&mode=${mode}`)
      if (!response.ok) {
        throw new Error('Search failed')
      }
      const data = await response.json()
      setResults(data.results || [])
    } catch (err) {
      setError('Failed to search. Make sure the Graphyne server is running.')
      console.error('Search error:', err)
    } finally {
      setLoading(false)
    }
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSearch()
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex gap-4">
        <Input
          type="text"
          placeholder="Search the knowledge base..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyPress={handleKeyPress}
          className="flex-1"
        />
        <Select value={mode} onValueChange={setMode}>
          <SelectTrigger className="w-[180px]">
            <SelectValue placeholder="Select mode" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="hybrid">Hybrid</SelectItem>
            <SelectItem value="lexical">Lexical</SelectItem>
            <SelectItem value="vector">Vector</SelectItem>
            <SelectItem value="graph">Graph</SelectItem>
          </SelectContent>
        </Select>
        <Button onClick={handleSearch} disabled={loading}>
          {loading ? 'Searching...' : 'Search'}
        </Button>
      </div>

      {error && (
        <div className="rounded-md bg-destructive/15 p-4 text-destructive">
          {error}
        </div>
      )}

      <div className="space-y-4">
        {results.length === 0 && !loading && query && (
          <p className="text-center text-muted-foreground py-8">
            No results found. Try a different query or mode.
          </p>
        )}
        
        {results.map((result, index) => (
          <Card key={result.id || index}>
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="text-lg">Result {index + 1}</CardTitle>
                <Badge variant="secondary">{result.mode || mode}</Badge>
              </div>
              <CardDescription>
                Score: {(result.score * 100).toFixed(2)}%
              </CardDescription>
            </CardHeader>
            <CardContent>
              <p className="text-sm text-muted-foreground">{result.content}</p>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  )
}
