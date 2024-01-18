protoc:
	protoc \
		--go_out=./proto \
		--go_opt=paths=source_relative \
		--go-grpc_out=./proto \
		--go-grpc_opt=paths=source_relative \
		--proto_path external/yellowstone-grpc-proto/proto/ external/yellowstone-grpc-proto/proto/*.proto
