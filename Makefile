service := chatservice
network := ${service}-network

.PHONY: default network backend frontend runBackend runFrontend

default: network backend frontend

network:
	docker network inspect ${network} >/dev/null 2>&1 || docker network create ${network}

backend: backend.Dockerfile
	docker build --tag ${service}-${@}:local -f $< .

frontend: frontend.Dockerfile
	docker build --tag ${service}-${@}:local -f $< .

runBackend: backend
	docker run --rm -ti --name ${service}-${<}:local --network=${network} -p 9000:9000 -p 9001:9001 ${service}-${<}

runFrontend: frontend
	docker run --rm -ti --name ${service}-${<}:local --network=${network} -p 8000:8000 -e CHAT_ENV='prod' ${service}-${<}

