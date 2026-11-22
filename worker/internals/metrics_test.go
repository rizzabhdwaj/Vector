package internals

import (
	"errors"
	"net/http/httptest"
	"strings"
	"testing"
	"time"
)

func TestMetricsRoutes(t *testing.T) {
	recordTaskResult(true, time.Second)
	recordTaskResult(false, time.Second)
	recordHeartbeat(nil)
	recordHeartbeat(errors.New("unreachable"))

	for _, path := range []string{"/metric", "/metrics"} {
		response := httptest.NewRecorder()
		metricsMux().ServeHTTP(response, httptest.NewRequest("GET", path, nil))
		if response.Code != 200 {
			t.Fatalf("%s: status %d", path, response.Code)
		}
		for _, sample := range []string{
			`worker_tasks_total{status="success"} 1`,
			`worker_tasks_total{status="failure"} 1`,
			`worker_heartbeats_total{status="success"} 1`,
			`worker_heartbeats_total{status="failure"} 1`,
			`worker_task_duration_seconds_count 2`,
		} {
			if !strings.Contains(response.Body.String(), sample) {
				t.Errorf("%s missing %s", path, sample)
			}
		}
	}
}
