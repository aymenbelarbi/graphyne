import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'

interface Memory {
  id: string
  content: string
  type: string
  timestamp: number
}

export default function MemoryPage() {
  const [memories, setMemories] = useState<Memory[]>([])
  const [content, setContent] = useState('')
  const [memoryType, setMemoryType] = useState('working')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [success, setSuccess] = useState('')

  const fetchMemories = async () => {
    try {
      const response = await fetch('http://localhost:8080/v1/memory/recall?type=${memoryType}')
      if (!response.ok) {
        throw new Error('Failed to fetch memories')
      }
      const data = await response.json()
      setMemories(data.memories || [])
    } catch (err) {
      console.error('Failed to fetch memories:', err)
    }
  }

  const handleStore = async () => {
    if (!content.trim()) return
    
    setLoading(true)
    setError('')
    setSuccess('')
    
    try {
      const response = await fetch('http://localhost:8080/v1/memory/store', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          content,
          memory_type: memoryType,
        }),
      })
      
      if (!response.ok) {
        throw new Error('Failed to store memory')
      }
      
      setSuccess('Memory stored successfully!')
      setContent('')
      fetchMemories()
    } catch (err) {
      setError('Failed to store memory. Make sure the Graphyne server is running.')
      console.error('Store error:', err)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchMemories()
  }, [memoryType])

  return (
    <div className="space-y-6">
      {/* Store Memory Form */}
      <Card>
        <CardHeader>
          <CardTitle>Store New Memory</CardTitle>
          <CardDescription>
            Add a new memory to the knowledge base.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-4">
            <Select value={memoryType} onValueChange={setMemoryType}>
              <SelectTrigger className="w-[200px]">
                <SelectValue placeholder="Select type" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="working">Working</SelectItem>
                <SelectItem value="episodic">Episodic</SelectItem>
                <SelectItem value="semantic">Semantic</SelectItem>
                <SelectItem value="procedural">Procedural</SelectItem>
              </SelectContent>
            </Select>
          </div>
          
          <Textarea
            placeholder="Enter memory content..."
            value={content}
            onChange={(e) => setContent(e.target.value)}
            rows={4}
          />
          
          <Button onClick={handleStore} disabled={loading || !content.trim()}>
            {loading ? 'Storing...' : 'Store Memory'}
          </Button>
          
          {error && (
            <p className="text-sm text-destructive">{error}</p>
          )}
          {success && (
            <p className="text-sm text-green-600">{success}</p>
          )}
        </CardContent>
      </Card>

      {/* Memory List */}
      <Card>
        <CardHeader>
          <CardTitle>Stored Memories</CardTitle>
          <CardDescription>
            Browse memories filtered by type.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Type</TableHead>
                <TableHead>Content</TableHead>
                <TableHead>Timestamp</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {memories.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={3} className="text-center text-muted-foreground">
                    No memories found.
                  </TableCell>
                </TableRow>
              ) : (
                memories.map((memory) => (
                  <TableRow key={memory.id}>
                    <TableCell>
                      <Badge variant="outline">{memory.type}</Badge>
                    </TableCell>
                    <TableCell className="max-w-md truncate">
                      {memory.content}
                    </TableCell>
                    <TableCell className="text-muted-foreground">
                      {new Date(memory.timestamp * 1000).toLocaleString()}
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}
