// Prometheus metrics for the worker.
package internals

import (
	"log"
	"net/http"
	"os"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
)

var (
	tasksTotal = prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Namespace: "worker",
			Name:      "tasks_total",
			Help:      "Total number of tasks executed by the worker.",
		},
		[]string{"status"},
	)

	taskDuration = prometheus.NewHistogram(
		prometheus.HistogramOpts{
			Namespace: "worker",
			Name:      "task_duration_seconds",
			Help:      "Time spent executing tasks.",
			Buckets:   prometheus.DefBuckets,
		},
	)

	heartbeatsTotal = prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Namespace: "worker",
			Name:      "heartbeats_total",
			Help:      "Total number of heartbeats sent to the coordinator.",
		},
		[]string{"status"},
	)

	lastHeartbeat = prometheus.NewGauge(
		prometheus.GaugeOpts{
			Namespace: "worker",
			Name:      "last_heartbeat_timestamp_seconds",
			Help:      "Unix timestamp of the last successfully sent heartbeat.",
		},
	)

	workerUp = prometheus.NewGauge(
		prometheus.GaugeOpts{
			Namespace: "worker",
			Name:      "up",
			Help:      "Whether the worker process is running.",
		},
	)
)

func init() {
	prometheus.MustRegister(tasksTotal, taskDuration, heartbeatsTotal, lastHeartbeat, workerUp)
	workerUp.Set(1)
	// Pre-create the label values so the series are visible before the first event.
	tasksTotal.WithLabelValues("success")
	tasksTotal.WithLabelValues("failure")
	heartbeatsTotal.WithLabelValues("success")
	heartbeatsTotal.WithLabelValues("failure")
}

func recordTaskResult(success bool, duration time.Duration) {
	status := "success"
	if !success {
		status = "failure"
	}
	tasksTotal.WithLabelValues(status).Inc()
	taskDuration.Observe(duration.Seconds())
}

func recordHeartbeat(err error) {
	status := "success"
	if err != nil {
		status = "failure"
	} else {
		lastHeartbeat.SetToCurrentTime()
	}
	heartbeatsTotal.WithLabelValues(status).Inc()
}

// metricsMux registers the worker's metrics and opt-in chaos routes.
func metricsMux(heartbeat *HeartbeatControl, enableChaos bool) *http.ServeMux {
	mux := http.NewServeMux()
	mux.Handle("/metrics", promhttp.Handler())
	mux.Handle("/metric", promhttp.Handler())
	if enableChaos {
		mux.HandleFunc("POST /chaos/heartbeat/pause", heartbeat.PauseHandler)
	}
	return mux
}

// StartMetricsServer serves Prometheus metrics over HTTP. The listen address
// can be configured with WORKER_METRICS (default ":9100").
func StartMetricsServer(heartbeat *HeartbeatControl) {
	addr := os.Getenv("WORKER_METRICS")
	if addr == "" {
		addr = ":9100"
	}

	log.Printf("metrics server listening on %s", addr)
	if err := http.ListenAndServe(addr, metricsMux(heartbeat, os.Getenv("WORKER_ENABLE_CHAOS") == "1")); err != nil {
		log.Printf("metrics server stopped: %v", err)
	}
}
