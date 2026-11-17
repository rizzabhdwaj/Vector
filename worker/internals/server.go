// Recieve's HTTP requests , decodes JSON , returns reponse
package internals;
import (
	"context"
	"log"
	"net"
	"worker/proto"
	"google.golang.org/grpc"
	"google.golang.org/grpc/reflection"
)

// server implements pb.GreeterServer.
type server struct {
	pb.UnimplementedGreeterServer // Required for forward compatibility
}

// SayHello implements the RPC method defined in greet.proto.
func (s *server) SayHello(ctx context.Context, req *pb.HelloRequest) (*pb.HelloReply, error) {
	return &pb.HelloReply{
		Message: "Hello, " + req.GetName(),
	}, nil
}

func ProtoServer() {
	// 1. Open a TCP listener on port 50051
	lis, err := net.Listen("tcp", ":50051")
	if err != nil {
		log.Fatalf("Failed to listen: %v", err)
	}

	grpcServer := grpc.NewServer()

	pb.RegisterGreeterServer(grpcServer, &server{})

	reflection.Register(grpcServer)

	log.Println("gRPC server listening on :50051")

	if err := grpcServer.Serve(lis); err != nil {
		log.Fatalf("Failed to serve: %v", err)
	}
}

