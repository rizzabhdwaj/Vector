// executes task and caputre results

package internals

import (
	"context"
	"errors"
	"os/exec"
	"time"
)

// Used only when a caller sends no (or an invalid) time limit; normal requests
// carry their own timelimit and that value is authoritative.
const fallbackTaskTimeout = 15 * time.Second

type Response struct {
	success bool
	id      int32
	content []byte
}

func execute_task(id int32, command string, args []string, timelimit uint32) *Response {
	limit := fallbackTaskTimeout
	if timelimit > 0 {
		limit = time.Duration(timelimit) * time.Second
	}
	return execute_task_with_timeout(id, command, args, limit)
}

func execute_task_with_timeout(id int32, command string, args []string, limit time.Duration) *Response {
	started := time.Now()
	ctx, cancel := context.WithTimeout(context.Background(), limit)
	defer cancel()

	cmd := exec.CommandContext(ctx, command, args...)
	out, err := cmd.Output()

	success := err == nil
	recordTaskResult(success, time.Since(started))

	if !success {
		content := err.Error()
		// Report the time limit to the coordinator instead of the kill signal.
		if errors.Is(ctx.Err(), context.DeadlineExceeded) || errors.Is(err, context.DeadlineExceeded) {
			content = "timelimit exceeded"
		}
		return &Response{content: []byte(content), id: id, success: false}
	}
	return &Response{content: out, id: id, success: true}
}
