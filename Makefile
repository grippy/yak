RUST_VERSION=1.74
LLVM_VERSION=14
IMAGE_NAME="rust-llvm:rust-${RUST_VERSION}-llvm-${LLVM_VERSION}"

# docker build rust-llvm
docker-build:
	docker buildx build . \
		--build-arg RUST_VERSION=${RUST_VERSION} \
		--build-arg LLVM_VERSION=${LLVM_VERSION} \
		--file ./dev/Dockerfile \
		--tag ${IMAGE_NAME} \
		--load

# docker run rust-llvm (should skip adding local ./taget directory)
docker-run:
	# mount empty ./target folder
	docker run -it \
		-v ${PWD}:${PWD} \
		-v ${PWD}/target/ \
		-w ${PWD} \
		${IMAGE_NAME}