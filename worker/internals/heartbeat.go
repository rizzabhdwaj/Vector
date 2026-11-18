package internals

import (
	"github.com/joho/godotenv"
	"log"
	"os"
	"strconv"

	pb "worker/proto"

	"context"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"time"
)

func Heartbeat() {

	err := godotenv.Load()
	if err != nil {
		log.Fatal("Error loading .env file")
	}

	grpc_addr := os.Getenv("COORDINATOR_GRPC")
	workerID := int64(1)
	if value := os.Getenv("WORKER_ID"); value != "" {
		workerID, err = strconv.ParseInt(value, 10, 32)
		if err != nil {
			log.Fatalf("Invalid WORKER_ID: %v", err)
		}
	}

	conn, err := grpc.NewClient(
		grpc_addr,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		log.Fatal(err)
	}
	defer conn.Close()

	client := pb.NewHeartbeatClient(conn)

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	client.SendHeartbeat(ctx, &pb.HeartbeatRequest{
		WorkerId:  int32(workerID),
		Available: true,
	})

}
