// executes task and caputre results

package internals

import (
	"os/exec"
	"time"
)

type Response struct {
	success bool
	id      int32
	content []byte
}

func execute_task(id int32, command string, args []string) *Response {
	started := time.Now()
	cmd := exec.Command(command, args...)
	out, err := cmd.Output()

	success := err == nil
	recordTaskResult(success, time.Since(started))

	if !success {
		return &Response{content: []byte(err.Error()), id: id, success: false}
	}
	return &Response{content: out, id: id, success: true}

}
