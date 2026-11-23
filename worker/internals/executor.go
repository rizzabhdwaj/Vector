// executes task and caputre results

package internals

import (
	"context"
	"errors"
	"os/exec"
	"time"
)

const taskTimeout = 10 * time.Second

type Response struct {
	success bool
	id      int32
	content []byte
}

func execute_task(id int32, command string, args []string) *Response {
	return execute_task_with_timeout(id, command, args, taskTimeout)
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
