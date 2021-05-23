VERSION:=	0.2.3
build:
	docker build -t pizzamig/breathe:${VERSION} -t pizzamig/breathe:latest .

push:
	docker push pizzamig/breathe:latest
	docker push pizzamig/breathe:${VERSION}
