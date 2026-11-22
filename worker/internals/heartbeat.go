package internals

import (
	"context"
	"log"
	"time"

	pb "worker/proto"
)

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
