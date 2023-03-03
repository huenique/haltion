clean:
	docker rm -vf $$(docker ps -aq)
	docker rmi -f $$(docker images -aq)
	docker volume rm $$(docker volume ls -q)

run:
	docker-compose up
