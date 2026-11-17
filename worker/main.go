//Initialize dependencies, start HTTP server, handle shutdown

package main

import (
	"time"
	"worker/internals"
)

func periodicHeartbeat() {
	for {
		time.Sleep(5)
		internals.Heartbeat();
	}
}

func main() {
	internals.ProtoServer()
	go periodicHeartbeat();
}
