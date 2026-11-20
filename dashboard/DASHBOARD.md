# Vector Dashboard

Run the coordinator, then start the dashboard:

```sh
npm install
npm run dev
```

The dashboard connects to port 3030 on the browser's current hostname by default.
To use another coordinator, set `VITE_API_BASE_URL` in `.env.local` (see
`.env.example`) and restart Vite. This value is embedded at build time.

Workers refresh every five seconds. The table shows registered workers and their
reported availability, not verified liveness. Tasks are submitted to the shared
queue, not to a selected worker. A submission confirmation means queued, not
successfully executed.

Enter an executable in Command and one argument per line in Arguments. Leave
Arguments empty for a command with no arguments. Spaces within each line are
preserved; do not add shell quotes. Shell pipelines and redirects are not parsed.

The coordinator has no authentication and workers execute the submitted commands.
Keep both the dashboard and coordinator on a trusted network.

```sh
npm run build
npm run lint
```
