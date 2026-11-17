package internals

import (
	"github.com/joho/godotenv"
	"log"
	"os"

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
		WorkerId:      1,
		Available : true,
	})

}
