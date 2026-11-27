import { check, sleep } from 'k6';
import exec from 'k6/execution';
import http from 'k6/http';

function intEnv(name, fallback, min, max) {
  const raw = __ENV[name] ?? String(fallback);
  const value = Number(raw);
  if (!/^\d+$/.test(raw) || !Number.isInteger(value) || value < min || value > max) {
    throw new Error(`${name} must be an integer between ${min} and ${max}`);
  }
  return value;
}

const baseUrl = (__ENV.VECTOR_URL ?? 'http://127.0.0.1:3030').replace(/\/+$/, '');
const tasks = intEnv('TASKS', 40, 1, 10000);
const vus = intEnv('VUS', Math.min(3, tasks), 1, Math.min(tasks, 50));
const maxPauseMs = intEnv('MAX_PAUSE_MS', 1500, 0, 10000);
const failurePercent = intEnv('FAILURE_PERCENT', 15, 0, 100);
const pauseSeconds = intEnv('HEARTBEAT_PAUSE_SECONDS', 25, 1, 300);
const workerUrls = [...new Set((__ENV.WORKER_URLS ??
  'http://127.0.0.1:9101,http://127.0.0.1:9102,http://127.0.0.1:9103,http://127.0.0.1:9104,http://127.0.0.1:9105')
  .split(',')
  .map((url) => url.trim().replace(/\/+$/, ''))
  .filter(Boolean))];
const maxHeartbeatPauses = Math.min(2, tasks, workerUrls.length);
const heartbeatPauses = __ENV.HEARTBEAT_PAUSES === undefined
  ? null
  : intEnv('HEARTBEAT_PAUSES', 0, 0, maxHeartbeatPauses);

if (heartbeatPauses !== 0) {
  if (maxHeartbeatPauses === 0 || workerUrls.some((url) => !/^https?:\/\/[^/?#]+$/.test(url))) {
    throw new Error('WORKER_URLS must contain comma-separated worker HTTP base URLs');
  }
  if (!__ENV.WORKER_URLS && !/^https?:\/\/(?:localhost|127\.0\.0\.1|\[::1\])(?::\d+)?$/.test(baseUrl)) {
    throw new Error('Set WORKER_URLS for a non-local VECTOR_URL, or set HEARTBEAT_PAUSES=0');
  }
}

export const options = {
  scenarios: {
    chaos: {
      executor: 'shared-iterations',
      vus,
      iterations: tasks,
      maxDuration: '10m',
    },
  },
  thresholds: {
    checks: ['rate==1'],
    iterations: [`count==${tasks}`],
  },
};

export function setup() {
  // The coordinator converts IDs to signed int32. Pick a random contiguous
  // block so VUs don't submit duplicate IDs within this run.
  const random = crypto.getRandomValues(new Uint32Array(1))[0];
  const candidates = [...workerUrls];
  const count = heartbeatPauses ?? randomInt(1, maxHeartbeatPauses);
  const midpoint = Math.ceil(tasks / 2);
  const pauses = {};
  for (let i = 0; i < count; i++) {
    const iteration = i === 0 ? randomInt(0, midpoint - 1) : randomInt(midpoint, tasks - 1);
    const [workerUrl] = candidates.splice(randomInt(0, candidates.length - 1), 1);
    pauses[iteration] = workerUrl;
  }
  return { firstId: 1 + (random % (0x7fffffff - tasks + 1)), pauses };
}

function randomInt(min, max) {
  return min + Math.floor(Math.random() * (max - min + 1));
}

function workload(id) {
  const message = `vector-chaos:${id}`;
  if (randomInt(1, 100) <= failurePercent) {
    if (randomInt(0, 1) === 0) {
      return { kind: 'exit_error', command: 'false', args: [], timelimit: 5 };
    }
    return { kind: 'timeout', command: 'sleep', args: ['3'], timelimit: 1 };
  }

  switch (randomInt(0, 2)) {
    case 0:
      return { kind: 'echo', command: 'echo', args: [message], timelimit: 5 };
    case 1:
      return { kind: 'sleep', command: 'sleep', args: [String(randomInt(1, 2))], timelimit: 5 };
    default:
      return {
        kind: 'sleep_then_print',
        command: 'sh',
        args: ['-c', 'sleep "$1" && printf "%s\\n" "$2"', 'vector-chaos', String(randomInt(1, 2)), message],
        timelimit: 5,
      };
  }
}

function pauseWorker(workerUrl) {
  const response = http.post(`${workerUrl}/chaos/heartbeat/pause?seconds=${pauseSeconds}`, null, {
    tags: { name: 'worker heartbeat pause', workload: 'heartbeat_pause' },
    redirects: 0,
    timeout: '5s',
  });
  if (check(response, { 'worker accepted heartbeat pause (204)': (r) => r.status === 204 })) {
    console.info(`paused heartbeats at ${workerUrl} for ${pauseSeconds}s`);
  } else {
    console.warn(`heartbeat pause at ${workerUrl}: expected HTTP 204, got ${response.status}; check WORKER_URLS and WORKER_ENABLE_CHAOS=1`);
  }
}

export default function ({ firstId, pauses }) {
  const iteration = exec.scenario.iterationInTest;
  if (pauses[iteration]) {
    pauseWorker(pauses[iteration]);
  }
  const id = firstId + iteration;
  const { kind, ...task } = workload(id);
  const response = http.post(`${baseUrl}/tasks`, JSON.stringify({ id, status: 'Queued', ...task }), {
    headers: { 'Content-Type': 'application/json' },
    tags: { workload: kind },
    redirects: 0,
    timeout: '5s',
  });

  if (!check(response, { 'coordinator accepted task (201)': (r) => r.status === 201 })) {
    console.warn(`task ${id} (${kind}): expected HTTP 201, got ${response.status}`);
  }

  // Some iterations go straight into the next submission; others pause.
  if (maxPauseMs > 0 && Math.random() < 0.5) {
    sleep(randomInt(1, maxPauseMs) / 1000);
  }
}
