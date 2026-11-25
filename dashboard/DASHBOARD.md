# Vector Dashboard

Run the coordinator, then start the dashboard:

```sh
npm install
npm run dev
```

Vite proxies `/vector-api` to `127.0.0.1:3030` (stripping the prefix) and
`/vector-ws` to `127.0.0.1:3031`. Run Vite and the coordinator on the same
host. Workers send heartbeats to the coordinator on gRPC port `50052`.

For a deployed dashboard, configure those same routes on your reverse proxy,
or set `VITE_API_BASE_URL` and `VITE_WS_BASE_URL` to browser-reachable endpoints
before building. The WebSocket listener is local to the coordinator host. An
HTTPS deployment should expose the WebSocket endpoint over `wss://`.

Workers and tasks update over the WebSocket. The worker table shows registered workers, their
reported availability (not verified liveness), and the IDs of tasks currently
assigned to them. Assigned tasks are removed from that list once they finish.
Tasks are submitted to the shared queue, not to a selected worker. A submission
confirmation means queued, not successfully executed.

Enter an executable in Command and one argument per line in Arguments. Leave
Arguments empty for a command with no arguments. Spaces within each line are
preserved; do not add shell quotes. Shell pipelines and redirects are not parsed.

Time limit sets how long the worker runs the command, in seconds. Leave it blank
to use the 15-second default.

The coordinator has no authentication and workers execute the submitted commands.
Keep both the dashboard and coordinator on a trusted network.

```sh
npm run build
npm run lint
```
