package internals

import (
	"strings"
	"testing"
	"time"
)

func TestExecuteTaskReportsTimeLimitExceeded(t *testing.T) {
	response := execute_task_with_timeout(42, "sleep", []string{"5"}, 50*time.Millisecond)

	if response.success {
		t.Fatal("timed out task reported success")
	}
	if response.id != 42 {
		t.Errorf("id = %d, want 42", response.id)
	}
	if got := string(response.content); got != "timelimit exceeded" {
		t.Errorf("content = %q, want %q", got, "timelimit exceeded")
	}
}

func TestExecuteTaskReturnsOutput(t *testing.T) {
	response := execute_task_with_timeout(7, "echo", []string{"hello"}, 2*time.Second)

	if !response.success {
		t.Fatalf("task failed: %s", response.content)
	}
	if got := string(response.content); !strings.Contains(got, "hello") {
		t.Errorf("content = %q, want it to contain hello", got)
	}
}
