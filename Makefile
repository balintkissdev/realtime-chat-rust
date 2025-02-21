imageName := "chatservice"

.PHONY: default buildDocker run

default: buildDocker

buildDocker:
	docker build --tag ${imageName} --file Dockerfile .

run: buildDocker
	docker run --rm -ti -p 8000:8000 -p 9001:9001 ${imageName}
