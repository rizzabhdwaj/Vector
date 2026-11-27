package internals

import (
	"context"
	"net/http"
	"net/http/httptest"
	"sync/atomic"
	"testing"
	"testing/synctest"
	"time"
)

func TestHeartbeatPauseRoute(t *testing.T) {
	for _, tc := range []struct {
		name    string
		enabled bool
		method  string
		query   string
		want    int
	}{
		{"disabled", false, "POST", "?seconds=20", 404},
		{"wrong method", true, "GET", "?seconds=20", 405},
		{"missing", true, "POST", "", 400},
		{"invalid", true, "POST", "?seconds=abc", 400},
		{"zero", true, "POST", "?seconds=0", 400},
		{"too long", true, "POST", "?seconds=301", 400},
		{"valid", true, "POST", "?seconds=20", 204},
	} {
		t.Run(tc.name, func(t *testing.T) {
			h := NewHeartbeatControl()
			w := httptest.NewRecorder()
			metricsMux(h, tc.enabled).ServeHTTP(w, httptest.NewRequest(tc.method, "/chaos/heartbeat/pause"+tc.query, nil))
			if w.Code != tc.want {
				t.Fatalf("status = %d, want %d", w.Code, tc.want)
			}
			if paused := !h.pausedUntil.IsZero(); paused != (tc.want == http.StatusNoContent) {
				t.Fatal("only an accepted request should change the pause deadline")
			}
		})
	}
}

func TestHeartbeatPauseAndResume(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		h := NewHeartbeatControl()
		var sent atomic.Int32
		go h.run(ctx, 5*time.Second, func() { sent.Add(1) })
		synctest.Wait()

		expectSent := func(want int32) {
			t.Helper()
			synctest.Wait()
			if got := sent.Load(); got != want {
				t.Fatalf("sent %d heartbeats, want %d", got, want)
			}
		}
		time.Sleep(5 * time.Second)
		expectSent(1)

		h.pause(8 * time.Second)
		time.Sleep(4 * time.Second)
		h.pause(10 * time.Second) // Extend the original deadline.
		h.pause(time.Second)      // A shorter request cannot shorten the pause.
		time.Sleep(9 * time.Second)
		expectSent(1)

		time.Sleep(time.Second)
		expectSent(2) // Resume at the deadline, not at the next regular tick.
		time.Sleep(5 * time.Second)
		expectSent(3)
	})
}
