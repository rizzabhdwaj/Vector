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
  return { firstId: 1 + (random % (0x7fffffff - tasks + 1)) };
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

export default function ({ firstId }) {
  const id = firstId + exec.scenario.iterationInTest;
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
