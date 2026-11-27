package internals

import (
	"context"
	"log"
	"net/http"
	"strconv"
	"sync"
	"time"

	pb "worker/proto"
)

const heartbeatInterval = 5 * time.Second

// HeartbeatControl coordinates a worker's heartbeat loop and chaos endpoint.
type HeartbeatControl struct {
	mu          sync.Mutex
	pausedUntil time.Time
	changed     chan struct{}
}

func NewHeartbeatControl() *HeartbeatControl {
	return &HeartbeatControl{changed: make(chan struct{}, 1)}
}

func (h *HeartbeatControl) pause(duration time.Duration) {
	h.mu.Lock()
	until := time.Now().Add(duration)
	if until.After(h.pausedUntil) {
		h.pausedUntil = until
	}
	h.mu.Unlock()
	select {
	case h.changed <- struct{}{}:
	default:
	}
}

// PauseHandler pauses new heartbeat attempts. An RPC already in flight may finish.
func (h *HeartbeatControl) PauseHandler(w http.ResponseWriter, r *http.Request) {
	seconds, err := strconv.Atoi(r.URL.Query().Get("seconds"))
	if err != nil || seconds < 1 || seconds > 300 {
		http.Error(w, "seconds must be an integer from 1 to 300", http.StatusBadRequest)
		return
	}
	h.pause(time.Duration(seconds) * time.Second)
	w.WriteHeader(http.StatusNoContent)
}

func (h *HeartbeatControl) Run(ctx context.Context, client pb.HeartbeatClient, workerID int32) {
	h.run(ctx, heartbeatInterval, func() { Heartbeat(ctx, client, workerID) })
}

func (h *HeartbeatControl) run(ctx context.Context, interval time.Duration, send func()) {
	next := time.Now().Add(interval)
	for ctx.Err() == nil {
		h.mu.Lock()
		due := next
		if !h.pausedUntil.IsZero() {
			due = h.pausedUntil
		}
		if !time.Now().Before(due) {
			h.pausedUntil = time.Time{}
			h.mu.Unlock()
			send()
			next = time.Now().Add(interval)
			continue
		}
		h.mu.Unlock()

		timer := time.NewTimer(time.Until(due))
		select {
		case <-ctx.Done():
		case <-h.changed:
		case <-timer.C:
		}
		timer.Stop()
	}
}

func Heartbeat(ctx context.Context, client pb.HeartbeatClient, workerID int32) {
	ctx, cancel := context.WithTimeout(ctx, 2*time.Second)
	defer cancel()

	_, err := client.SendHeartbeat(ctx, &pb.HeartbeatRequest{
		WorkerId:  workerID,
		Available: true,
	})
	recordHeartbeat(err)
	if err != nil {
		log.Printf("heartbeat failed for worker %d: %v", workerID, err)
	}
}
