import { useEffect, useState, type FormEvent } from 'react'

type WorkerStatus = 'free' | 'busy' | 'dead'
type Worker = { id: number; curr_status: WorkerStatus }
type TaskStatus = 'Assigned' | 'Queued' | 'Failed'
type Task = { id: number; command: string; args: string[]; status: TaskStatus }
type TaskOutput = { task_id: number; output: string }
type TaskQueueSnapshot = { content: Task[]; outputs: TaskOutput[] }

const workerStatus: Record<WorkerStatus, { label: string; dot: string }> = {
  free: { label: 'Available', dot: 'bg-emerald-600' },
  busy: { label: 'Busy', dot: 'bg-yellow-400' },
  dead: { label: 'Dead', dot: 'bg-black' },
}

const apiUrl = (import.meta.env.VITE_API_BASE_URL || `http://${window.location.hostname}:3030`).replace(/\/$/, '')
const inputClass = 'w-full rounded border border-zinc-300 bg-white px-3 py-2 font-mono text-sm outline-none focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600 disabled:opacity-60'

function App() {
  const [workers, setWorkers] = useState<Worker[] | null>(null)
  const [workerError, setWorkerError] = useState('')
  const [queue, setQueue] = useState<TaskQueueSnapshot | null>(null)
  const [queueError, setQueueError] = useState('')
  const [command, setCommand] = useState('echo')
  const [args, setArgs] = useState('hello world')
  const [submitting, setSubmitting] = useState(false)
  const [result, setResult] = useState<{ error: boolean; message: string } | null>(null)

  useEffect(() => {
    const controller = new AbortController()
    let timer: ReturnType<typeof setTimeout>

    async function refresh() {
      try {
        const response = await fetch(`${apiUrl}/workforce`, {
          signal: controller.signal,
          cache: 'no-store',
        })
        if (!response.ok) throw new Error(`HTTP ${response.status}`)
        const data: { workers: Worker[] } = await response.json()
        if (!controller.signal.aborted) {
          setWorkers(data.workers.sort((a, b) => a.id - b.id))
          setWorkerError('')
        }
      } catch {
        if (!controller.signal.aborted) {
          setWorkerError('Cannot reach the coordinator. Check that it is running and accessible.')
        }
      } finally {
        if (!controller.signal.aborted) timer = setTimeout(refresh, 50)
      }
    }

    void refresh()
    return () => {
      controller.abort()
      clearTimeout(timer)
    }
  }, [])

  useEffect(() => {
    const controller = new AbortController()
    let timer: ReturnType<typeof setTimeout>

    async function refresh() {
      try {
        const response = await fetch(`${apiUrl}/tasks`, {
          signal: AbortSignal.any([controller.signal, AbortSignal.timeout(15000)]),
          cache: 'no-store',
        })
        if (!response.ok) throw new Error(`HTTP ${response.status}`)
        const data: TaskQueueSnapshot = await response.json()
        if (!controller.signal.aborted) {
          setQueue(data)
          setQueueError('')
        }
      } catch {
        if (!controller.signal.aborted) {
          setQueueError('Could not load tasks. Retrying automatically.')
        }
      } finally {
        if (!controller.signal.aborted) timer = setTimeout(refresh, 5000)
      }
    }

    void refresh()
    return () => {
      controller.abort()
      clearTimeout(timer)
    }
  }, [])

  async function submitTask(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (!command.trim() || submitting) return
    setSubmitting(true)
    setResult(null)
    // The scheduler converts task IDs to signed 32-bit integers.
    const id = crypto.getRandomValues(new Uint32Array(1))[0] & 0x7fffffff
    try {
      const response = await fetch(`${apiUrl}/tasks`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id,
          command: command.trim(),
          args: args === '' ? [] : args.replace(/\r\n/g, '\n').split('\n'),
          status: 'Assigned',
        }),
        signal: AbortSignal.timeout(15000),
      })
      if (!response.ok) throw new Error(`Coordinator returned HTTP ${response.status}.`)
      setResult({ error: false, message: `Task ${id} queued. It will run when a worker is available.` })
    } catch (error) {
      setResult({
        error: true,
        message: error instanceof Error && error.message.startsWith('Coordinator returned')
          ? error.message
          : 'Could not confirm submission. Check the coordinator before retrying to avoid a duplicate task.',
      })
    } finally {
      setSubmitting(false)
    }
  }

  const outputs = queue?.outputs ? [...queue.outputs].reverse() : null
  const queuedTasks = queue?.content ?? null

  return (
    <main className="mx-auto max-w-5xl px-5 py-8 sm:px-8 sm:py-12">
      <header className="mb-8 border-b border-zinc-200 pb-5">
        <h1 className="text-xl font-semibold tracking-tight">Vector</h1>
        <p className="mt-1 text-sm text-zinc-500">Workers, tasks & outputs</p>
      </header>

      <div className="grid items-start gap-10 md:grid-cols-2 md:gap-12">
        <section aria-labelledby="workers-heading">
          <div className="mb-4 flex items-center justify-between gap-3">
            <h2 id="workers-heading" className="text-base font-semibold">
              Workers {workers && <span className="ml-1 font-normal text-zinc-500">({workers.length})</span>}
            </h2>
            <span className="text-xs text-zinc-500">Refreshes ever- VERY FAST!</span>
          </div>
          {workerError && <p role="alert" className="mb-3 text-sm text-red-700">{workerError}{workers && ' Showing last known data.'}</p>}
          <div className="overflow-hidden rounded border border-zinc-200">
            <table className="w-full text-left text-sm">
              <thead className="border-b border-zinc-200 bg-zinc-50 text-zinc-500">
                <tr>
                  <th scope="col" className="px-4 py-3 font-medium">Worker ID</th>
                  <th scope="col" className="px-4 py-3 font-medium">Status</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-100">
                {workers?.map((worker) => {
                  const status = workerStatus[worker.curr_status] ?? { label: 'Unknown', dot: 'bg-zinc-400' }
                  return (
                    <tr key={worker.id}>
                      <td className="px-4 py-3 font-mono">{worker.id}</td>
                      <td className="px-4 py-3">
                        <span className="inline-flex items-center gap-2">
                          <span aria-hidden="true" className={`size-1.5 rounded-full ${status.dot}`} />
                          {status.label}
                        </span>
                      </td>
                    </tr>
                  )
                })}
                {(!workers || workers.length === 0) && (
                  <tr><td colSpan={2} className="px-4 py-8 text-center text-zinc-500">
                    {workers ? 'No workers registered yet.' : workerError ? 'Worker list unavailable.' : 'Loading workers...'}
                  </td></tr>
                )}
              </tbody>
            </table>
          </div>
          <p className="mt-3 text-xs text-zinc-500">Reported by the coordinator; not a live health check.</p>
        </section>

        <section aria-labelledby="task-heading">
          <h2 id="task-heading" className="mb-4 text-base font-semibold">New task</h2>
          <form onSubmit={submitTask} className="space-y-5">
            <div>
              <label htmlFor="command" className="mb-2 block text-sm font-medium">Command</label>
              <input id="command" className={inputClass} placeholder="echo" required value={command} onChange={(event) => setCommand(event.target.value)} disabled={submitting} autoComplete="off" spellCheck={false} />
            </div>
            <div>
              <label htmlFor="arguments" className="mb-2 block text-sm font-medium">Arguments <span className="font-normal text-zinc-500">(optional)</span></label>
              <textarea id="arguments" className={`${inputClass} min-h-32 resize-y`} rows={5} placeholder="Hello from Vector" value={args} onChange={(event) => setArgs(event.target.value)} disabled={submitting} spellCheck={false} aria-describedby="arguments-help" />
              <p id="arguments-help" className="mt-2 text-xs leading-5 text-zinc-500">One argument per line. Spaces are preserved; no shell quoting needed. Commands run directly, without a shell.</p>
            </div>
            <button type="submit" disabled={submitting || !command.trim()} className="rounded bg-zinc-900 px-4 py-2 text-sm font-medium text-white hover:bg-zinc-700 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-zinc-900 disabled:cursor-not-allowed disabled:opacity-40">
              {submitting ? 'Submitting...' : 'Submit task'}
            </button>
            {result && <p role={result.error ? 'alert' : 'status'} className={`text-sm leading-6 ${result.error ? 'text-red-700' : 'text-emerald-700'}`}>{result.message}</p>}
          </form>
        </section>
      </div>
      <section aria-labelledby="tasks-heading" className="mt-10 border-t border-zinc-200 pt-6">
        <div className="mb-4 flex items-center justify-between gap-3">
          <h2 id="tasks-heading" className="text-base font-semibold">
            Tasks {queue && <span className="ml-1 font-normal text-zinc-500">({queuedTasks?.length ?? 0} queued, {outputs?.length ?? 0} with output)</span>}
          </h2>
          <span className="text-xs text-zinc-500">Refreshes every 5s</span>
        </div>
        {queueError && <p role="alert" className="mb-3 text-sm text-red-700">{queueError}{queue && ' Showing last known data.'}</p>}

        <div className="space-y-8">
          <div>
            <h3 className="mb-3 text-sm font-medium text-zinc-700">Queued / in-memory tasks</h3>
            <div className="overflow-hidden rounded border border-zinc-200">
              <table className="w-full text-left text-sm">
                <thead className="border-b border-zinc-200 bg-zinc-50 text-zinc-500">
                  <tr>
                    <th scope="col" className="px-4 py-3 font-medium">Task ID</th>
                    <th scope="col" className="px-4 py-3 font-medium">Command</th>
                    <th scope="col" className="px-4 py-3 font-medium">Args</th>
                    <th scope="col" className="px-4 py-3 font-medium">Status</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-zinc-100">
                  {queuedTasks?.map((task) => (
                    <tr key={task.id}>
                      <td className="px-4 py-3 font-mono">{task.id}</td>
                      <td className="px-4 py-3 font-mono">{task.command}</td>
                      <td className="px-4 py-3 font-mono text-zinc-600">{task.args.length ? task.args.join(' ') : '(none)'}</td>
                      <td className="px-4 py-3">{task.status}</td>
                    </tr>
                  ))}
                  {(!queuedTasks || queuedTasks.length === 0) && (
                    <tr><td colSpan={4} className="px-4 py-8 text-center text-zinc-500">
                      {queue ? 'No queued tasks in memory.' : queueError ? 'Queued tasks unavailable.' : 'Loading tasks...'}
                    </td></tr>
                  )}
                </tbody>
              </table>
            </div>
          </div>

          <div>
            <h3 className="mb-3 text-sm font-medium text-zinc-700">Task outputs</h3>
            {outputs?.length ? (
              <div className="space-y-3">
                {outputs.map((output, index) => (
                  <article key={`${output.task_id}-${outputs.length - index}`} className="min-w-0 overflow-hidden rounded border border-zinc-200">
                    <h3 className="border-b border-zinc-200 bg-zinc-50 px-4 py-2 text-sm font-medium">
                      Task <span className="font-mono">{output.task_id}</span>
                    </h3>
                    <pre tabIndex={0} aria-label={`Output for task ${output.task_id}`} className="max-h-80 overflow-auto whitespace-pre-wrap break-words p-4 font-mono text-sm leading-6">{output.output || '(No output returned)'}</pre>
                  </article>
                ))}
              </div>
            ) : (
              <p role="status" className="rounded border border-zinc-200 px-4 py-8 text-center text-sm text-zinc-500">
                {outputs ? 'No outputs yet. Results will appear here after a worker runs a task.' : queueError ? 'Task outputs unavailable.' : 'Loading outputs...'}
              </p>
            )}
            <p className="mt-3 text-xs text-zinc-500">Newest results first. Output does not indicate execution success; the coordinator does not report exit status.</p>
          </div>
        </div>
      </section>
      <footer className="mt-12 border-t border-zinc-200 pt-4 text-xs leading-5 text-zinc-500">
        Trusted environments only. Submitted commands execute on your workers.
      </footer>
    </main>
  )
}

export default App
