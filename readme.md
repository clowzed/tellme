# Tellme
![Docker Image Size (latest by date)](https://img.shields.io/docker/image-size/clowzed/tellme?color=u&label=docker%20image%20size)   ![GitHub](https://img.shields.io/github/license/clowzed/tellme?color=g)
[![build](https://github.com/clowzed/tellme/actions/workflows/build.yml/badge.svg)](https://github.com/clowzed/tellme/actions/workflows/build.yml)

Tiny service registry with notifications feature

## API wrapper crate (rust)
- [`tellme-client`](https://github.com/clowzed/tellme-client)


## Installation
1) From source
```sh
git clone https://github.com/clowzed/tellme
cd tellme
# Modify .env file
cargo run --release
```
2) With `docker`
```sh
git clone https://github.com/clowzed/tellme
cd tellme
# Create .env file
sudo docker run -it -d --env-file=./.env clowzed/tellme:latest
```
3) With `docker-compose`
```sh
git clone https://github.com/tellme
cd tellme
sudo docker-compose up -d
```
4) Use in docker-compose.yml
```
service-registry:
    image: clowzed/tellme:latest
    environment:
      - PORT=8080
      - HEALTHCHECK_INTERVAL=30
      - LOGIN=my-login
      - PASSWORD=my-password
```

### Usage
2) If you want to register service you need access token.
```python
import requests
response = requests.post("<server_ip>/newtoken", data = dict(login    = "login",
                                                             password = "pass"))

if response.status_code:
    print("Access token: ", response.json().get("token", None))
```
3) Then you need to send request to server with access token and params
Server will recive you your unique identifier

| param                | decription                                   |
| -------------------- | -------------------------------------------- |
| access_token         | Token given in last request                  |
| healthcheck_endpoint | Point for ping service to check availability |
| service_type         | Type of the service for query in /find       |
```python
import requests

response = requests.post("<server_ip>/me", data = dict(access_token        = access_token,
                                                      healthcheck_endpoint = "/health",
                                                      service_type         = "storage"))
if response.status_code:
    identifier = response.json().get("identifier", None)
```

4) Now your service is not shown to any other clients. You need to accept it.
```python
import requests

response = requests.post("{server_ip}/accept", data = dict(login     = login_from_file,
                                                          password   = password_from_file,
                                                          identifier = identifier))
if response.status_code:
    print("Service was successfully accepted!")

```
5) Now it is available and shown to others.
You can get information about services using /find

| param        | decription                                                              |
| ------------ | ----------------------------------------------------------------------- |
| limit        | limits the amount of services to be returned  `usize`                   |
| is_available | returns services which are now available. Default: returns all services |
| service_type | type of the services to filter on. Default: all types are returned      |

```python
import requests
response = requests.get("{server_ip}/find", params = dict())
if response.status_code:
    print("Services: ", response.json())
```
6) You can register hooks on registration and on acceptance of any service
service registry will notify you on provided endpoint with POST request and Service data as Json
```python
import requests

# Subscribing for all registration events
response = requests.post("<server_ip>/subscribe", params = dict(
    login           = "login",
    password        = "pass",
    on_registration = true,
    on_acceptance   = flase,
    identifier      = identifier, #Subscriber identifier
    endpoint        = "/hooks/on-registration"
))

# Subscribing for all registration events
response = requests.post("<server_ip>/subscribe", params = dict(
    login           = "login",
    password        = "pass",
    on_registration = false,
    on_acceptance   = true,
    identifier      = identifier, #Subscriber identifier
    endpoint        = "/hooks/on-acceptance"
))
```
